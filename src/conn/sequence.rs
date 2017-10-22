// sequence provides sequenced reliable data segments.

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

const MAX_TRIES: usize = 5;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    seq: u32,
    payload: Vec<u8>,
}

impl Segment {
    fn bytes(&self) -> usize { 4 + self.payload.len() }
}

#[derive(Debug, PartialEq)]
pub enum Cmd {
    Segments(Vec<Segment>),
    DroppedReliable,
}

// Manages sequenced reliable delivery and reception of data segments. Outgoint segments will be
// retried up to MAX_TRIES and incoming segments can only be dequeued when the next in order is
// available.
#[derive(Debug, PartialEq)]
pub struct Sequence {
    incoming: Window<Segment>,
    outgoing: Window<State>,
}

#[derive(Debug, PartialEq)]
struct Window<T> {
    next_seq: u32,
    slots: BTreeMap<u32, T>,
}

// Represents the state of a transmitted segment.
#[derive(Debug, PartialEq)]
enum State {
    Staged(Segment),
    Online { transmissions: usize, last: Instant, segment: Segment },
    Failed,
}

impl State {
    // Returns whether this segment should be transmitted this turn.
    fn should_send(&self, round_trip: Duration, bandwidth_allowed: usize) -> bool {
        match *self {
            State::Staged(ref segment) => segment.bytes() <= bandwidth_allowed,
            State::Online { last, ref segment, .. } => {
                last.elapsed() >= round_trip && segment.bytes() <= bandwidth_allowed
            }
            _ => false,
        }
    }

    // Steps the state forward to represent its upcoming transmission. If this transmission would
    // exceed the try limit, the state is transitionsed to failure and the segment should not be
    // sent.
    fn step_for_transmission(self) -> Self {
        match self {
            State::Staged(segment) => State::Online {
                transmissions: 1,
                last: Instant::now(),
                segment,
            },
            State::Online { transmissions, segment, .. } => {
                if transmissions == MAX_TRIES {
                    State::Failed
                } else {
                    State::Online {
                        transmissions: transmissions + 1,
                        segment,
                        last: Instant::now(),
                    }
                }
            }
            s => s,
        }
    }

    fn segment(&self) -> Option<&Segment> {
        match *self {
            State::Staged(ref segment) => Some(&segment),
            State::Online { ref segment, .. } => Some(&segment),
            _ => None,
        }
    }
}

impl Sequence {
    // Returns a new sequence that starts counting from the provided sequence numbers.
    pub fn new(in_seq: u32, out_seq: u32) -> Sequence {
        Sequence {
            incoming: Window { next_seq: in_seq, slots: BTreeMap::new() },
            outgoing: Window { next_seq: out_seq, slots: BTreeMap::new() },
        }
    }

    // Queues data to be sent.
    pub fn send(&mut self, payload: Vec<u8>) {
        let seq = self.outgoing.next_seq;
        self.outgoing.next_seq += 1;
        self.outgoing.slots.insert(
            seq,
            State::Staged(Segment { seq, payload }),
        );
    }

    // Queues a segment to be recieved when it is in order. If segment 1 is received but no 0, 1 is
    // buffered until 0 arrives.
    pub fn receive(&mut self, segment: Segment) {
        self.incoming.slots.insert(segment.seq, segment);
        while self.incoming.slots.contains_key(&self.incoming.next_seq) {
            self.incoming.next_seq += 1
        }
    }

    // Acknowledge receipt of a segment from the peer; this segment should never be retransmitted.
    pub fn acknowledge(&mut self, seq: u32) { self.outgoing.slots.remove(&seq); }

    // Gets any commands to send data segments this turn based on which ones are live and fit in
    // available bandwidth. 
    pub fn cmds(&mut self, round_trip: Duration, mut bandwidth_allowed: usize) -> (usize, Cmd) {
        match self.outgoing.slots.iter().next().map(|(seq, _)| *seq) {
            Some(lowest_unfinished) => {
                let mut to_send = Vec::new();
                for i in lowest_unfinished..(self.outgoing.next_seq) {
                    if self.outgoing.slots[&i].should_send(round_trip, bandwidth_allowed) {
                        let next_state = self.outgoing
                            .slots
                            .remove(&i)
                            .unwrap()
                            .step_for_transmission();
                        match next_state.segment().is_some() {
                            true => {
                                to_send.push(next_state.segment().unwrap().clone());
                                bandwidth_allowed -= next_state.segment().unwrap().bytes();
                                self.outgoing.slots.insert(i, next_state);
                            }
                            false => return (bandwidth_allowed, Cmd::DroppedReliable),
                        }
                    }
                }
                (bandwidth_allowed, Cmd::Segments(to_send))
            }
            None => (bandwidth_allowed, Cmd::Segments(Vec::new())),
        }
    }

    // Dequeue any segments received that fall in order. This may not return all buffered recevied
    // segments.
    pub fn dequeue(&mut self) -> (usize, Vec<Vec<u8>>) {
        match self.incoming.slots.iter().next().map(|(seq, _)| *seq) {
            Some(lowest_buffered) => {
                let mut ready = Vec::new();
                for i in lowest_buffered..(self.incoming.next_seq) {
                    match self.incoming.slots.remove(&i) {
                        Some(segment) => ready.push(segment.payload),
                        None => {}
                    }
                }

                (ready.iter().map(|v| v.len()).sum(), ready)
            }
            None => (0, Vec::new()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;

    #[test]
    fn stages_outgoing_grams() {
        let mut sequence = Sequence::new(0, 100);
        let expected_gram = Segment { seq: 100, payload: vec![0, 1, 2] };
        sequence.send(expected_gram.payload.clone());

        // Request for outgoing grams that fit in 0 bytes should yield none.
        assert_eq!(sequence.cmds(Duration::new(0, 0), 0), (0, Cmd::Segments(vec![])));
        // Request for outgoing grams that fit in the staged segment's bytes should yield our staged
        // segment.
        assert_eq!(sequence.cmds(Duration::new(0, 0), expected_gram.bytes()), (
            0,
            Cmd::Segments(vec![expected_gram]),
        ));
    }

    #[test]
    fn retries_outgoing_grams() {
        let mut sequence = Sequence::new(0, 100);
        let expected_gram1 = Segment { seq: 100, payload: vec![0, 1, 2] };
        let expected_gram2 = Segment { seq: 101, payload: vec![0, 1, 2, 4] };
        sequence.send(expected_gram1.payload.clone());
        sequence.send(expected_gram2.payload.clone());

        // Request for cmds with bandwidth to fit first staged segment should return that segment
        // and return the extra bandwidth.
        assert_eq!(sequence.cmds(Duration::new(0, 0), expected_gram1.bytes() + 1), (
            1,
            Cmd::Segments(
                vec![expected_gram1.clone()],
            ),
        ));
        // Request for cmds with too little bandwidth for the second staged segment should yield
        // nothing.
        assert_eq!(sequence.cmds(Duration::new(0, 0), 3), (3, Cmd::Segments(vec![])));
        // Request for cmds where a round trip hasn't elapsed since the first segment's transmission
        // should yield the second segment.
        assert_eq!(sequence.cmds(Duration::new(100, 0), expected_gram2.bytes()), (
            0,
            Cmd::Segments(
                vec![expected_gram2],
            ),
        ));


        // Request for cmds with bandwidth to fit first staged segment after elapsed round trip
        // should resend staged segment until all tries are used up, then fail.
        for _ in 1..MAX_TRIES {
            assert_eq!(sequence.cmds(Duration::new(0, 0), expected_gram1.bytes()), (
                0,
                Cmd::Segments(
                    vec![expected_gram1.clone()],
                ),
            ));
        }
        assert_eq!(sequence.cmds(Duration::new(0, 0), 1000), (1000, Cmd::DroppedReliable));
    }

    #[test]
    fn acknowlegdes_delivered_grams() {
        let mut sequence = Sequence::new(0, 100);
        let expected_gram = Segment { seq: 100, payload: vec![0, 1, 2] };
        sequence.send(expected_gram.payload.clone());

        // Request for outgoing grams that fit in the staged segment's bytes should yield our staged
        // segment.
        assert_eq!(sequence.cmds(Duration::new(0, 0), expected_gram.bytes()), (
            0,
            Cmd::Segments(
                vec![expected_gram.clone()],
            ),
        ));

        // After acknowledgement the segment should be removed and not returned in later flushes.
        sequence.acknowledge(100);
        assert_eq!(sequence.cmds(Duration::new(0, 0), expected_gram.bytes()), (
            expected_gram.bytes(),
            Cmd::Segments(vec![]),
        ));
    }

    #[test]
    fn buffers_incoming_grams() {
        let mut sequence = Sequence::new(0, 100);
        let gram1 = Segment { seq: 1, payload: vec![1, 2, 3] };
        sequence.receive(gram1.clone());

        // Request for dequeue of incoming grams should return nothing because we still need
        // segment 0.
        assert_eq!(sequence.dequeue(), (0, Vec::<Vec<u8>>::new()));

        let gram0 = Segment { seq: 0, payload: vec![0, 1, 2] };
        sequence.receive(gram0.clone());

        // Request for dequeue should now return all buffered grams since the sequence is complete.
        assert_eq!(sequence.dequeue(), (
            gram0.payload.len() + gram1.payload.len(),
            vec![gram0.payload, gram1.payload],
        ));
    }
}

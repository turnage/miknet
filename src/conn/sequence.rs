// sequence provides sequenced reliable data segments.

use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::mem::size_of;
use std::time::{Duration, Instant};

const MAX_TRIES: usize = 5;

/// A segment is the online representation of a gram with a sequence number.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    seq:     u32,
    payload: Vec<u8>,
}

impl Segment {
    fn size(&self) -> usize { size_of::<u32>() + self.payload.len() }
}

/// Cmd enumerates instructions the Sequence state machine will output when
/// processing events.
#[derive(Debug, PartialEq)]
pub enum Cmd {
    TxSegment(Segment),
    RxSegment(Segment),
    HandleDroppedReliable,
}

/// Sequence does accounting for sequenced reliable exchange of data. There are
/// two windows where online and staged grams are accounted for.
#[derive(Debug, PartialEq)]
pub struct Sequence {
    /// The incoming window stages grams until an order can be provided to
    /// clients above this layer. For example, if we recevie over the wire grams
    /// 3, 4, and 2, but not 1, we stage 3, 4, and 2 until 1 comes in.
    ///
    /// The `next_seq` always holds the next sequence number we expect. In the
    /// example above `next_seq = 1`.
    incoming: Window<Segment>,
    /// The outgoing window stages grams until we receive receipt of their delivery,
    /// so we can retry them and know when to give up.
    ///
    /// The `next_seq` always holds the next sequence number we will label a gram to
    /// be sent with. If we have 2, and 3 on the line, `next_seq = 4` and it will be
    /// assigned to the next queued gram.
    outgoing: Window<State>,
}

/// Window holds a sliding window over the connection's gram sequence.
#[derive(Debug, PartialEq)]
struct Window<T> {
    next_seq: u32,
    slots:    BTreeMap<u32, T>,
}

// Represents the state of a transmitted segment.
#[derive(Debug, PartialEq)]
enum State {
    Staged(Segment),
    Online {
        txs:     usize,
        last:    Instant,
        segment: Segment,
    },
    Failed,
}

impl State {
    // Returns whether this segment should be transmitted this turn.
    fn should_tx(&self, round_trip: Duration, data_allowed: usize) -> bool {
        match *self {
            State::Staged(ref segment) => segment.size() <= data_allowed,
            State::Online {
                last, ref segment, ..
            } => last.elapsed() >= round_trip && segment.size() <= data_allowed,
            _ => false,
        }
    }

    // Steps the state forward to represent its upcoming transmission. If this transmission would
    // exceed the try limit, the state is transitionsed to failure and the segment should not be
    // sent.
    fn step_for_tx(self) -> Self {
        match self {
            State::Staged(segment) => State::Online {
                txs: 1,
                last: Instant::now(),
                segment,
            },
            State::Online { txs, segment, .. } => {
                if txs == MAX_TRIES {
                    State::Failed
                } else {
                    State::Online {
                        txs: txs + 1,
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
            incoming: Window {
                next_seq: in_seq,
                slots:    BTreeMap::new(),
            },
            outgoing: Window {
                next_seq: out_seq,
                slots:    BTreeMap::new(),
            },
        }
    }

    // Queues data to be sent.
    pub fn tx(&mut self, payload: Vec<u8>) {
        let seq = self.outgoing.next_seq;
        self.outgoing.next_seq += 1;
        self.outgoing
            .slots
            .insert(seq, State::Staged(Segment { seq, payload }));
    }

    // Queues a segment to be recieved when it is in order. If segment 1 is received but no 0, 1 is
    // buffered until 0 arrives.
    pub fn rx(&mut self, segment: Segment) {
        self.incoming.slots.insert(segment.seq, segment);
        while self.incoming.slots.contains_key(&self.incoming.next_seq) {
            self.incoming.next_seq += 1
        }
    }

    // Acknowledge receipt of a segment from the peer; this segment should never be retransmitted.
    pub fn acknowledge(&mut self, seq: u32) {
        self.outgoing.slots.remove(&seq);
    }

    // Gets any commands to send data segments this turn based on which ones are live and fit in
    // available bandwidth.
    pub fn cmds(
        &mut self,
        round_trip: Duration,
        data_allowed: usize,
    ) -> (usize, Vec<Cmd>) {
        let (data_allowed, mut tx_cmds) =
            self.enqueue_tx(round_trip, data_allowed);
        let mut rx_segs = self
            .dequeue_rx()
            .into_iter()
            .map(|s| Cmd::RxSegment(s))
            .collect();
        tx_cmds.append(&mut rx_segs);
        (data_allowed, tx_cmds)
    }

    fn enqueue_tx(
        &mut self,
        round_trip: Duration,
        mut data_allowed: usize,
    ) -> (usize, Vec<Cmd>) {
        match self.outgoing.slots.iter().next().map(|(seq, _)| *seq) {
            Some(lowest_unfinished) => {
                let mut to_send = Vec::new();
                for i in lowest_unfinished..(self.outgoing.next_seq) {
                    if self.outgoing.slots[&i]
                        .should_tx(round_trip, data_allowed)
                    {
                        let next_state = self
                            .outgoing
                            .slots
                            .remove(&i)
                            .unwrap()
                            .step_for_tx();
                        match next_state.segment() {
                            Some(ref segment) => {
                                to_send
                                    .push(Cmd::TxSegment((*segment).clone()));
                                data_allowed -= segment.size();
                                self.outgoing.slots.insert(i, next_state);
                            }
                            None => to_send.push(Cmd::HandleDroppedReliable),
                        }
                    }
                }
                (data_allowed, to_send)
            }
            None => (data_allowed, Vec::new()),
        }
    }

    // Dequeue any segments received that fall in order. This may not return all buffered recevied
    // segments.
    fn dequeue_rx(&mut self) -> Vec<Segment> {
        match self.incoming.slots.iter().next().map(|(seq, _)| *seq) {
            Some(lowest_buffered) => {
                let mut ready = Vec::new();
                for i in lowest_buffered..(self.incoming.next_seq) {
                    match self.incoming.slots.remove(&i) {
                        Some(segment) => ready.push(segment),
                        None => {}
                    }
                }

                ready
            }
            None => Vec::new(),
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
        let expected_seg = Segment {
            seq:     100,
            payload: vec![0, 1, 2],
        };
        sequence.tx(expected_seg.payload.clone());

        // Request for outgoing grams that fit in 0 bytes should yield none.
        assert_eq!(sequence.cmds(Duration::new(0, 0), 0), (0, vec![]));
        // Request for outgoing grams that fit in the staged segment's bytes should yield our staged
        // segment.
        assert_eq!(
            sequence.cmds(Duration::new(0, 0), expected_seg.size()),
            (0, vec![Cmd::TxSegment(expected_seg)],)
        );
    }

    #[test]
    fn retries_outgoing_grams() {
        let mut sequence = Sequence::new(0, 100);
        let expected_seg1 = Segment {
            seq:     100,
            payload: vec![0, 1, 2],
        };
        let expected_seg2 = Segment {
            seq:     101,
            payload: vec![0, 1, 2, 4],
        };
        sequence.tx(expected_seg1.payload.clone());
        sequence.tx(expected_seg2.payload.clone());

        // Request for cmds with bandwidth to fit first staged segment should return that segment
        // and return the extra bandwidth.
        assert_eq!(
            sequence.cmds(Duration::new(0, 0), expected_seg1.size() + 1),
            (1, vec![Cmd::TxSegment(expected_seg1.clone())],)
        );
        // Request for cmds with too little bandwidth for the second staged segment should yield
        // nothing.
        assert_eq!(sequence.cmds(Duration::new(0, 0), 3), (3, vec![]));
        // Request for cmds where a round trip hasn't elapsed since the first segment's transmission
        // should yield the second segment.
        assert_eq!(
            sequence.cmds(Duration::new(100, 0), expected_seg2.size()),
            (0, vec![Cmd::TxSegment(expected_seg2.clone())],)
        );

        // Request for cmds with bandwidth to fit first staged segment after elapsed round trip
        // should resend staged segment until all tries are used up, then fail.
        for _ in 1..MAX_TRIES {
            assert_eq!(
                sequence.cmds(Duration::new(0, 0), expected_seg1.size()),
                (0, vec![Cmd::TxSegment(expected_seg1.clone())],)
            );
        }
        assert_eq!(
            sequence.cmds(Duration::new(0, 0), 1000),
            (
                992,
                vec![Cmd::HandleDroppedReliable, Cmd::TxSegment(expected_seg2)]
            )
        );
    }

    #[test]
    fn acknowlegdes_delivered_grams() {
        let mut sequence = Sequence::new(0, 100);
        let expected_seg = Segment {
            seq:     100,
            payload: vec![0, 1, 2],
        };
        sequence.tx(expected_seg.payload.clone());

        // Request for outgoing grams that fit in the staged segment's bytes should yield our staged
        // segment.
        assert_eq!(
            sequence.cmds(Duration::new(0, 0), expected_seg.size()),
            (0, vec![Cmd::TxSegment(expected_seg.clone())],)
        );

        // After acknowledgement the segment should be removed and not returned in later flushes.
        sequence.acknowledge(100);
        assert_eq!(
            sequence.cmds(Duration::new(0, 0), expected_seg.size()),
            (expected_seg.size(), vec![],)
        );
    }

    #[test]
    fn buffers_incoming_grams() {
        let mut sequence = Sequence::new(0, 100);
        let seg1 = Segment {
            seq:     1,
            payload: vec![1, 2, 3],
        };
        sequence.rx(seg1.clone());

        // Request for dequeue of incoming grams should return nothing because we still need
        // segment 0.
        assert_eq!(sequence.cmds(Duration::new(0, 0), 0), (0, Vec::new()));

        let seg0 = Segment {
            seq:     0,
            payload: vec![0, 1, 2],
        };
        sequence.rx(seg0.clone());

        // Request for dequeue should now return all buffered grams since the sequence is complete.
        assert_eq!(
            sequence.cmds(Duration::new(0, 0), 0),
            (0, vec![Cmd::RxSegment(seg0), Cmd::RxSegment(seg1)],)
        );
    }
}

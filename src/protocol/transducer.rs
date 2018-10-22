pub trait Transducer: Sized {
    type Event;
    type Action;

    fn transition(
        self,
        event: Self::Event,
    ) -> (Option<Self>, Vec<Self::Event>, Vec<Self::Action>);

    fn transduce(
        mut self,
        mut events: Vec<Self::Event>,
    ) -> (Option<Self>, Vec<Self::Action>) {
        let mut queue = Vec::new();
        let mut actions = Vec::new();
        let mut state = Some(self);
        for event in events.drain(0..) {
            if let Some(transducer) = state {
                let (next_state, mut repipe_events, mut new_actions) =
                    transducer.transition(event);
                queue.append(&mut repipe_events);
                actions.append(&mut new_actions);
                state = next_state;
            } else {
                return (None, actions);
            }
        }

        (state, actions)
    }
}

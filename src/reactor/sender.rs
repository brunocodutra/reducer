use crate::reactor::*;
use std::sync::mpsc::{SendError, Sender};

impl<S> Reactor<S> for Sender<S>
where
    S: Clone,
{
    type Output = Result<(), SendError<S>>;

    fn react(&self, state: &S) -> Self::Output {
        self.send(state.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::*;
    use std::sync::mpsc::channel;

    proptest! {
        #[test]
        fn react(states: Vec<u8>) {
            let (tx, rx) = channel();

            for state in &states {
                assert!(tx.react(state).is_ok());
            }

            // hang up tx
            drop(tx);

            assert_eq!(rx.iter().collect::<Vec<_>>(), states);
        }
    }

    proptest! {
        #[test]
        fn err(states: Vec<u8>) {
            let (tx, _) = channel();

            for state in states {
                assert_eq!(tx.react(&state), Err(SendError(state)));
            }
        }
    }
}

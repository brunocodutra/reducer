use crate::reactor::*;
use std::sync::mpsc::{SendError, Sender};

/// Turns [`std::sync::mpsc::Sender`] into a [`Reactor`].
impl<S> Reactor<S> for Sender<S>
where
    S: Clone,
{
    type Error = SendError<S>;

    fn react(&mut self, state: &S) -> Result<(), Self::Error> {
        self.send(state.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn ok(states: Vec<u8>) {
            let (mut tx, rx) = std::sync::mpsc::channel();

            for state in &states {
                assert_eq!(react(&mut tx, state), Ok(()));
            }

            assert_eq!(rx.iter().take(states.len()).collect::<Vec<_>>(), states);
        }

        #[test]
        fn err(states: Vec<u8>) {
            let (mut tx, rx) = std::sync::mpsc::channel();

            // hang up tx
            drop(rx);

            for state in states {
                assert_eq!(react(&mut tx, &state), Err(SendError(state)));
            }
        }
    }
}

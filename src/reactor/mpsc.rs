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

#[cfg(feature = "async")]
use futures::{executor::block_on, sink::SinkExt};

#[cfg(feature = "async")]
use futures::channel::mpsc::{SendError as AsyncSendError, Sender as AsyncSender};

/// Turns [`futures::channel::mpsc::Sender`] into a [`Reactor`] (requires [`async`]).
///
/// [`async`]: index.html#experimental-features
#[cfg(feature = "async")]
impl<S> Reactor<S> for AsyncSender<S>
where
    S: Clone,
{
    type Error = AsyncSendError;

    fn react(&mut self, state: &S) -> Result<(), Self::Error> {
        block_on(self.send(state.clone()))
    }
}

#[cfg(feature = "async")]
use futures::channel::mpsc::{TrySendError, UnboundedSender};

/// Turns [`futures::channel::mpsc::UnboundedSender`] into a [`Reactor`] (requires [`async`]).
///
/// [`async`]: index.html#experimental-features
#[cfg(feature = "async")]
impl<S> Reactor<S> for UnboundedSender<S>
where
    S: Clone,
{
    type Error = TrySendError<S>;

    fn react(&mut self, state: &S) -> Result<(), Self::Error> {
        self.unbounded_send(state.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::*;
    use proptest::*;

    #[cfg(feature = "async")]
    use futures::stream::StreamExt;

    proptest! {
        #[test]
        fn std(states: Vec<u8>) {
            let (mut tx, rx) = std::sync::mpsc::channel();

            for state in &states {
                assert_eq!(react(&mut tx, state), Ok(()));
            }

            assert_eq!(rx.iter().take(states.len()).collect::<Vec<_>>(), states);

            // hang up tx
            drop(rx);

            assert_eq!(react(&mut tx, &0), Err(SendError(0)));
        }
    }

    proptest! {
        #[cfg(feature = "async")]
        #[test]
        fn sink(mut states: Vec<u8>) {
            let (mut tx, mut rx) = futures::channel::mpsc::channel(states.len());

            for state in &states {
                assert_eq!(react(&mut tx, state), Ok(()));
            }

            for state in states {
                assert_eq!(block_on(rx.next()), Some(state));
            }

            // hang up tx
            drop(rx);

            assert_ne!(react(&mut tx, &0), Ok(()));
        }
    }

    proptest! {
        #[cfg(feature = "async")]
        #[test]
        fn unbounded(mut states: Vec<u8>) {
            let (mut tx, mut rx) = futures::channel::mpsc::unbounded();

            for state in &states {
                assert_eq!(react(&mut tx, state), Ok(()));
            }

            for state in states {
                assert_eq!(block_on(rx.next()), Some(state));
            }

            // hang up tx
            drop(rx);

            assert_ne!(react(&mut tx, &0), Ok(()));
        }
    }
}

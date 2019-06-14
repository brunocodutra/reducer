use crate::reactor::*;
use std::sync::mpsc::{SendError, Sender};

/// Turns [`std::sync::mpsc::Sender`] into a [`Reactor`].
impl<S> Reactor<S> for Sender<S>
where
    S: Clone,
{
    type Output = Result<(), SendError<S>>;

    fn react(&self, state: &S) -> Self::Output {
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
    type Output = Result<(), AsyncSendError>;

    fn react(&self, state: &S) -> Self::Output {
        block_on(self.clone().send(state.clone()))
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
    type Output = Result<(), TrySendError<S>>;

    fn react(&self, state: &S) -> Self::Output {
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
            let (tx, rx) = std::sync::mpsc::channel();

            for state in &states {
                assert_eq!(react(&tx, state), Ok(()));
            }

            assert_eq!(rx.iter().take(states.len()).collect::<Vec<_>>(), states);

            // hang up tx
            drop(rx);

            assert_eq!(react(&tx, &0), Err(SendError(0)));
        }
    }

    proptest! {
        #[cfg(feature = "async")]
        #[test]
        fn sink(mut states: Vec<u8>) {
            let (tx, mut rx) = futures::channel::mpsc::channel(0);

            for state in &states {
                assert_eq!(react(&tx, state), Ok(()));
            }

            for state in states {
                assert_eq!(block_on(rx.next()), Some(state));
            }

            // hang up tx
            drop(rx);

            assert_ne!(react(&tx, &0), Ok(()));
        }
    }

    proptest! {
        #[cfg(feature = "async")]
        #[test]
        fn unbounded(mut states: Vec<u8>) {
            let (tx, mut rx) = futures::channel::mpsc::unbounded();

            for state in &states {
                assert_eq!(react(&tx, state), Ok(()));
            }

            for state in states {
                assert_eq!(block_on(rx.next()), Some(state));
            }

            // hang up tx
            drop(rx);

            assert_ne!(react(&tx, &0), Ok(()));
        }
    }
}

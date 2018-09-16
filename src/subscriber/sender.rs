use std::sync::mpsc::{SendError, Sender};
use subscriber::*;

impl<S> Subscriber<S> for Sender<S>
where
    S: Clone,
{
    type Error = SendError<S>;

    fn notify(&self, state: &S) -> Result<(), Self::Error> {
        self.send(state.clone())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::sync::mpsc::channel;

    #[test]
    fn sender() {
        let (tx, rx) = channel();

        assert!(tx.notify(&5).is_ok());
        assert!(tx.notify(&1).is_ok());
        assert!(tx.notify(&3).is_ok());

        // hang up tx
        drop(tx);

        assert_eq!(rx.iter().collect::<Vec<_>>(), vec![5, 1, 3]);
    }
}

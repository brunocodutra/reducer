use reactor::*;
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
pub(crate) mod tests {
    use super::*;
    use std::sync::mpsc::channel;

    #[test]
    fn react() {
        let (tx, rx) = channel();

        assert!(tx.react(&5).is_ok());
        assert!(tx.react(&1).is_ok());
        assert!(tx.react(&3).is_ok());

        // hang up tx
        drop(tx);

        assert_eq!(rx.iter().collect::<Vec<_>>(), vec![5, 1, 3]);
    }
}

#![cfg(test)]

use std::cell::RefCell;
use subscriber::Subscriber;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MockSubscriber<R> {
    states: RefCell<Vec<R>>,
    result: Result<(), ()>,
}

impl<R> MockSubscriber<R> {
    pub fn new(states: Vec<R>) -> Self {
        Self {
            states: RefCell::new(states),
            result: Ok(()),
        }
    }

    pub fn set_result(&mut self, result: impl FnOnce(()) -> Result<(), ()>) {
        self.result = result(());
    }
}

impl<R> Default for MockSubscriber<R> {
    fn default() -> Self {
        MockSubscriber::new(vec![])
    }
}

impl<R: Clone> Subscriber<R> for MockSubscriber<R> {
    type Error = ();

    fn notify(&self, state: &R) -> Result<(), Self::Error> {
        self.result?;
        self.states.borrow_mut().push(state.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notify() {
        let mut sbc = MockSubscriber::default();
        assert!(sbc.notify(&5).is_ok());

        sbc.set_result(Err);
        assert!(sbc.notify(&1).is_err());

        sbc.set_result(Ok);
        assert!(sbc.notify(&3).is_ok());

        assert_eq!(sbc, MockSubscriber::new(vec![5, 3]));
    }
}

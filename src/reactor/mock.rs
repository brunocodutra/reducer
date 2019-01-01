#![cfg(test)]

use reactor::Reactor;
use std::cell::RefCell;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MockReactor<R> {
    states: RefCell<Vec<R>>,
    result: Result<(), ()>,
}

impl<R> MockReactor<R> {
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

impl<R> Default for MockReactor<R> {
    fn default() -> Self {
        MockReactor::new(vec![])
    }
}

impl<R: Clone> Reactor<R> for MockReactor<R> {
    type Error = ();

    fn react(&self, state: &R) -> Result<(), Self::Error> {
        self.result?;
        self.states.borrow_mut().push(state.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn react() {
        let mut sbc = MockReactor::default();
        assert!(sbc.react(&5).is_ok());

        sbc.set_result(Err);
        assert!(sbc.react(&1).is_err());

        sbc.set_result(Ok);
        assert!(sbc.react(&3).is_ok());

        assert_eq!(sbc, MockReactor::new(vec![5, 3]));
    }
}

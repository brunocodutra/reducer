#![cfg(test)]

use reducer::Reducer;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct MockReducer<A: 'static> {
    actions: Vec<A>,
}

impl<A> MockReducer<A> {
    pub fn new(actions: Vec<A>) -> Self {
        Self { actions }
    }
}

impl<A> Reducer<A> for MockReducer<A> {
    fn reduce(&mut self, action: A) {
        self.actions.push(action);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduce() {
        let mut state = MockReducer::default();

        state.reduce(5);
        state.reduce(1);
        state.reduce(3);

        assert_eq!(state, MockReducer::new(vec![5, 1, 3]));
    }
}

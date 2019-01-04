#![cfg(test)]

use reactor::Reactor;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct MockReactor;

impl<S: Clone> Reactor<S> for MockReactor {
    type Output = S;

    fn react(&self, state: &S) -> Self::Output {
        state.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn react() {
        let reactor = MockReactor;

        assert_eq!(reactor.react(&5), 5);
        assert_eq!(reactor.react(&1), 1);
        assert_eq!(reactor.react(&3), 3);
    }
}

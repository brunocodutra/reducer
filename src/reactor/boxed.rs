use crate::reactor::*;

/// Forwards the event to the potentially _unsized_ nested reactor.
impl<S, T> Reactor<S> for Box<T>
where
    T: Reactor<S> + ?Sized,
{
    type Output = T::Output;

    fn react(&self, state: &S) -> Self::Output {
        (**self).react(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::*;
    use proptest::*;

    proptest! {
        #[test]
        fn react(states: Vec<char>) {
            let reactor = Box::new(MockReactor::default());

            for (i, state) in states.iter().enumerate() {
                assert_eq!(reactor.react(state), ());
                assert_eq!(reactor, Box::new(MockReactor::new(&states[0..=i])))
            }
        }
    }
}

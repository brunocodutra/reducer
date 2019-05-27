use crate::reactor::*;

/// Forwards the event to a potentially stack allocated reactor.
impl<'a, S, T> Reactor<S> for &'a T
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
            let reactor = &&MockReactor::default();

            for (i, state) in states.iter().enumerate() {
                assert_eq!(reactor.react(state), ());
                assert_eq!(reactor, &&MockReactor::new(&states[0..=i]))
            }
        }
    }
}

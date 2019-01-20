use crate::reactor::*;

impl<'a, S, T> Reactor<S> for &'a T
where
    T: Reactor<S> + ?Sized,
{
    type Output = T::Output;

    fn react(&self, state: &S) -> Self::Output {
        (*self).react(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::*;
    use proptest::*;

    proptest! {
        #[test]
        fn react(states: Vec<u8>) {
            let reactor = &&MockReactor::default();

            for state in states {
                assert_eq!(reactor.react(&state), state);
            }
        }
    }
}

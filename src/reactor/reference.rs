use crate::reactor::*;

/// Forwards the event to a potentially stack allocated [`Reactor`].
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
    use crate::mock::*;
    use proptest::*;

    proptest! {
        #[test]
        fn reference(states: Vec<u8>) {
            let reactor = &Mock::default();

            for (i, state) in states.iter().enumerate() {
                assert_eq!(react(&reactor, state), Ok(()));
                assert_eq!(reactor, &Mock::new(&states[0..=i]))
            }
        }
    }
}

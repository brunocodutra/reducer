use crate::reactor::*;

/// Forwards the event to a potentially stack allocated [`Reactor`].
impl<'a, S, T> Reactor<S> for &'a mut T
where
    T: Reactor<S> + ?Sized,
{
    type Error = T::Error;

    fn react(&mut self, state: &S) -> Result<(), Self::Error> {
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
            let mut reactor = &mut Mock::default();

            for (i, state) in states.iter().enumerate() {
                assert_eq!(react(&mut reactor, state), Ok(()));
                assert_eq!(reactor, &Mock::new(&states[0..=i]))
            }
        }
    }
}

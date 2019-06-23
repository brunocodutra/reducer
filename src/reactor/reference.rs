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
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn ok(states: Vec<u8>) {
            let mut reactor = &mut Mock::<_>::default();

            for (i, state) in states.iter().enumerate() {
                assert_eq!(react(&mut reactor, state), Ok(()));
                assert_eq!(reactor.calls(), &states[0..=i])
            }
        }
    }

    proptest! {
        #[test]
        fn err(state: u8, error: String) {
            let mut reactor = &mut Mock::default();
            reactor.fail_if(state, &error[..]);

            assert_eq!(react(&mut reactor, &state), Err(&error[..]));
            assert_eq!(reactor.calls(), &[state]);
        }
    }
}

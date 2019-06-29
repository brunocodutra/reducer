use crate::reactor::*;

/// Forwards the event to the potentially _unsized_ nested [`Reactor`] (requires [`std`]).
///
/// [`std`]: index.html#optional-features
impl<S, T> Reactor<S> for Box<T>
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
            let mut reactor = Box::new(Mock::<_>::default());

            for (i, state) in states.iter().enumerate() {
                assert_eq!(react(&mut reactor, state), Ok(()));
                assert_eq!(reactor.calls(), &states[0..=i])
            }
        }
    }

    proptest! {
        #[test]
        fn err(state: u8, error: String) {
            let mut reactor = Box::new(Mock::default());
            reactor.fail_if(state, &error[..]);

            assert_eq!(react(&mut reactor, &state), Err(&error[..]));
            assert_eq!(reactor.calls(), &[state]);
        }
    }
}

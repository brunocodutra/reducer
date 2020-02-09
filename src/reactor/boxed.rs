use crate::reactor::*;
use alloc::boxed::Box;

/// Forwards the event to the potentially _unsized_ nested [`Reactor`] (requires [`alloc`]).
///
/// [`alloc`]: index.html#optional-features
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
    use super::*;
    use mockall::predicate::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn react(state: u8, result: Result<(), u8>) {
            let mut mock = MockReactor::new();

            mock.expect_react()
                .with(eq(state))
                .times(1)
                .return_const(result);

            let mut reactor = Box::new(mock);
            assert_eq!(Reactor::react(&mut reactor, &state), result);
        }
    }
}

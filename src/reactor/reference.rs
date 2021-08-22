use crate::reactor::*;

/// Forwards the event to a potentially stack allocated [`Reactor`].
impl<'a, S, T> Reactor<S> for &'a mut T
where
    S: ?Sized,
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
    use test_strategy::proptest;

    #[proptest]
    fn react(state: u8, result: Result<(), u8>) {
        let mut mock = MockReactor::new();

        mock.expect_react()
            .with(eq(state))
            .once()
            .return_const(result);

        let mut reactor = &mut mock;
        assert_eq!(Reactor::react(&mut reactor, &state), result);
    }
}

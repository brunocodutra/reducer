use crate::reactor::*;

/// Notifies all [`Reactor`]s in the array in order.
///
/// <small>Currently implemented for arrays of up to 32 elements.</small>
///
/// # Example
///
/// ```rust
/// use reducer::*;
///
/// struct State { /* ... */ }
/// struct Action { /* ... */ }
///
/// impl Reducer<Action> for State {
///     fn reduce(&mut self, action: Action) {
///         // ...
///     }
/// }
///
/// struct Actor { /* ... */ }
/// struct ActorError(/*...*/);
///
/// impl Reactor<State> for Actor {
///     type Error = ActorError;
///     fn react(&mut self, state: &State) -> Result<(), Self::Error> {
///         // ...
///         Ok(())
///     }
/// }
///
/// let a = Actor { /* ... */ };
/// let b = Actor { /* ... */ };
/// // ...
/// let z = Actor { /* ... */ };
///
/// let mut store = Store::new(State { /* ... */ }, [a, b, /* ..., */ z]);
///
/// // All actors get notified of state changes.
/// store.dispatch(Action { /* ... */ });
/// ```
impl<S, T, const N: usize> Reactor<S> for [T; N]
where
    S: ?Sized,
    T: Reactor<S>,
{
    type Error = T::Error;

    fn react(&mut self, state: &S) -> Result<(), Self::Error> {
        self[..].react(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use test_strategy::proptest;

    #[proptest]
    fn react(state: u8, results: [Result<(), u8>; 32]) {
        let (idx, result) = results
            .iter()
            .enumerate()
            .find(|(_, r)| r.is_err())
            .map_or((results.len(), Ok(())), |(i, &r)| (i, r));

        let mut reactor: [MockReactor<_, _>; 32] = Default::default();

        for (i, (mock, result)) in reactor.iter_mut().zip(results).enumerate() {
            mock.expect_react()
                .with(eq(state))
                .times(if i > idx { 0 } else { 1 })
                .return_const(result);
        }

        assert_eq!(Reactor::react(&mut reactor, &state), result);
    }
}

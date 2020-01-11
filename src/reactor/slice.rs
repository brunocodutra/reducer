use crate::reactor::*;

/// Notifies all [`Reactor`]s in the slice in order.
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
/// # #[cfg(feature = "std")] {
/// let mut actors = vec![];
///
/// actors.push(Actor { /* ... */ });
/// actors.push(Actor { /* ... */ });
/// // ...
/// actors.push(Actor { /* ... */ });
///
/// let mut store = Store::new(State { /* ... */ }, actors.into_boxed_slice());
///
/// // All actors get notified of state changes.
/// store.dispatch(Action { /* ... */ });
/// # }
/// ```
impl<S, T> Reactor<S> for [T]
where
    T: Reactor<S>,
{
    type Error = T::Error;

    fn react(&mut self, state: &S) -> Result<(), Self::Error> {
        for reducer in self {
            reducer.react(state)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use proptest::prelude::*;
    use std::vec::Vec;

    proptest! {
        #[test]
        fn react(state: u8, mut results: Vec<Result<(), u8>>) {
            let (idx, result) = results
                .iter()
                .enumerate()
                .find(|(_, r)| r.is_err())
                .map_or((results.len(), Ok(())), |(i, &r)| (i, r));

            let mut mocks: Vec<_> = results
                .drain(..)
                .enumerate()
                .map(move |(i, r)| {
                    let mut mock = MockReactor::new();

                    mock.expect_react()
                        .with(eq(state))
                        .times(if i > idx { 0 } else { 1 })
                        .return_const(r);

                    mock
                })
                .collect();

            let reactor = mocks.as_mut_slice();
            assert_eq!(Reactor::react(reactor, &state), result);
        }
    }
}

use crate::reactor::*;

macro_rules! impl_reactor_for_arrays {
    () => {};

    ( $head:ident $(, $tail:ident)* $(,)? ) => {
        impl_reactor_for_arrays!($($tail,)*);

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
        impl<S, T> Reactor<S> for [T; count!($($tail,)*)]
        where
            S: ?Sized,
            T: Reactor<S>,
        {
            type Error = T::Error;

            fn react(&mut self, state: &S) -> Result<(), Self::Error> {
                self[..].react(state)
            }
        }
    };
}

impl_reactor_for_arrays!(
    _32, _31, _30, _29, _28, _27, _26, _25, _24, _23, _22, _21, _20, _19, _18, _17, _16, _15, _14,
    _13, _12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01, _00
);

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use proptest::prelude::*;

    macro_rules! test_reactor_for_arrays {
        () => {};

        ( $head:ident $(, $tail:ident)* $(,)? ) => {
            proptest! {
                #[test]
                fn $head(state: u8, results in vec![any::<Result<(), u8>>(); count!($($tail,)*)]) {
                    let (idx, result) = results
                        .iter()
                        .enumerate()
                        .find(|(_, r)| r.is_err())
                        .map_or((results.len(), Ok(())), |(i, &r)| (i, r));

                    let mut reactor: [MockReactor<_, _>; count!($($tail,)*)] = Default::default();

                    for (i, (mock, result)) in reactor.iter_mut().zip(results).enumerate() {
                        mock.expect_react()
                            .with(eq(state))
                            .times(if i > idx { 0 } else { 1 })
                            .return_const(result);
                    }

                    assert_eq!(Reactor::react(&mut reactor, &state), result);
                }
            }

            test_reactor_for_arrays!($($tail,)*);
        };
    }

    test_reactor_for_arrays!(
        _32, _31, _30, _29, _28, _27, _26, _25, _24, _23, _22, _21, _20, _19, _18, _17, _16, _15,
        _14, _13, _12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01, _00
    );
}

use crate::reactor::*;

macro_rules! impl_reactor_for_tuple {
    ( $($args:ident,)+ ) => {
        /// Notifies all [`Reactor`]s in the tuple in order.
        ///
        /// <small>Currently implemented for tuples of up to 12 elements.</small>
        ///
        /// # Example
        ///
        /// ```rust
        /// use reducer::*;
        /// use std::error::Error;
        ///
        /// #[derive(Debug)]
        /// struct State { /* ... */ }
        /// struct Action { /* ... */ }
        ///
        /// impl Reducer<Action> for State {
        ///     fn reduce(&mut self, action: Action) {
        ///         // ...
        ///     }
        /// }
        ///
        /// struct GUI { /* ... */ }
        /// struct DebugLogger { /* ... */ }
        ///
        /// impl Reactor<State> for GUI {
        ///     type Error = Box<dyn Error>;
        ///     fn react(&mut self, state: &State) -> Result<(), Self::Error> {
        ///         // ...
        ///         Ok(())
        ///     }
        /// }
        ///
        /// impl Reactor<State> for DebugLogger {
        ///     type Error = Box<dyn Error>;
        ///     fn react(&mut self, state: &State) -> Result<(), Self::Error> {
        ///         println!("[DEBUG] {:#?}", state);
        ///         Ok(())
        ///     }
        /// }
        ///
        /// let gui = GUI { /* ... */ };
        /// let logger = DebugLogger { /* ... */ };
        ///
        /// let mut store = Store::new(State { /* ... */ }, (gui, logger));
        ///
        /// // Both `gui` and `logger` get notified of state changes.
        /// store.dispatch(Action { /* ... */ });
        /// ```
        impl<S, X, $($args,)+> Reactor<S> for ($($args,)+)
        where
            $($args: Reactor<S, Error = X>,)+
        {
            type Error = X;

            fn react(&mut self, state: &S) -> Result<(), Self::Error> {
                #[allow(non_snake_case)]
                let ($($args,)+) = self;
                $($args.react(state)?;)+
                Ok(())
            }
        }
    };
}

macro_rules! impl_reactor_for_tuples {
    () => {};

    ( $head:ident $(, $tail:ident)* $(,)? ) => {
        impl_reactor_for_tuples!($($tail,)*);
        reverse!(impl_reactor_for_tuple!($head $(, $tail)*));
    };
}

impl_reactor_for_tuples!(L, K, J, I, H, G, F, E, D, C, B, A);

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use proptest::prelude::*;

    macro_rules! test_reactor_for_tuples {
        () => {};

        ( $head:ident $(, $tail:ident)* $(,)? ) => {
            proptest! {
                #[test]
                fn $head(state: u8, results: [Result<(), u8>; count!($($tail,)*) + 1]) {
                    let (idx, result) = results
                        .iter()
                        .enumerate()
                        .find(|(_, r)| r.is_err())
                        .map_or((results.len(), Ok(())), |(i, &r)| (i, r));

                    let mut mocks: [MockReactor<_, _>; count!($($tail,)*) + 1] = Default::default();

                    for (i, (mock, &result)) in mocks.iter_mut().zip(&results).enumerate() {
                        mock.expect_react()
                            .with(eq(state))
                            .times(if i > idx { 0 } else { 1 })
                            .return_const(result);
                    }

                    let [$head, $($tail,)*] = mocks;
                    let mut reactor = ($head, $($tail,)*);
                    assert_eq!(Reactor::react(&mut reactor, &state), result);
                }
            }

            test_reactor_for_tuples!($($tail,)*);
        };
    }

    test_reactor_for_tuples!(_12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01);
}

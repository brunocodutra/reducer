use crate::reducer::*;
use alloc::rc::Rc;

/// Enhances a potentially _unsized_ [`Reducer`] with copy-on-write semantics (requires [`alloc`]).
///
/// Helps avoiding cloning the entire state when it needs to be sent to other parts of the
/// application.
///
/// [`alloc`]: index.html#optional-features
///
/// # Example
///
/// ```rust
/// use reducer::*;
/// use std::rc::Rc;
///
/// #[derive(Clone)]
/// struct State { /* ... */ }
/// struct Action { /* ... */ }
/// struct Actor<T> { states: Vec<T>, /* ... */ }
///
/// impl Reducer<Action> for State {
///     fn reduce(&mut self, action: Action) {
///         // ...
///     }
/// }
///
/// impl<T: Clone> Reactor<T> for Actor<T> {
///     type Error = std::convert::Infallible; // TODO: use `!` once it's stable.
///     fn react(&mut self, state: &T) -> Result<(), Self::Error> {
///         self.states.push(state.clone());
///         Ok(())
///     }
/// }
///
/// let state = Rc::new(State { /* ... */ });
/// let reactor = Actor { states: vec![], /* ... */ };
/// let mut store = Store::new(state, reactor);
///
/// store.dispatch(Action { /* ... */ }); // `state` is not cloned yet.
///
/// // `reactor` now holds a reference to `state`.
///
/// store.dispatch(Action { /* ... */ }); // `state` is cloned through `Rc::make_mut`.
/// ```
impl<A, T> Reducer<A> for Rc<T>
where
    T: Reducer<A> + Clone + ?Sized,
{
    fn reduce(&mut self, action: A) {
        Rc::make_mut(self).reduce(action);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn reduce(action: u8) {
            let mut mock = MockReducer::new();

            mock.expect_reduce()
                .with(eq(action))
                .times(1)
                .return_const(());

            let mut reducer = Rc::new(mock);
            Reducer::reduce(&mut reducer, action);
        }

        #[test]
        fn cow(action: u8) {
            let mut mock = MockReducer::new();
            mock.expect_reduce().never();
            mock.expect_clone().times(1).returning(move || {
                let mut mock = MockReducer::new();
                mock.expect_reduce().with(eq(action)).times(1).return_const(());
                mock.expect_clone().never();
                mock
            });

            let mut reducer = Rc::new(mock);
            let other = reducer.clone();
            Reducer::reduce(&mut reducer, action);
            drop(other);
        }
    }
}

use crate::reducer::*;
use std::rc::Rc;

/// Enhances a potentially _unsized_ [`Reducer`] with copy-on-write semantics (requires [`std`]).
///
/// Helps avoiding cloning the entire state when it needs to be sent to other parts of the
/// application.
///
/// [`std`]: index.html#optional-features
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
///
/// impl Reducer<Action> for State {
///     fn reduce(&mut self, action: Action) {
///         // ...
///     }
/// }
///
/// let state = Rc::new(State { /* ... */ });
/// let reactor = Reactor::<Error = _>::from_sink(vec![]);
///
/// let mut store = Store::new(state, reactor);
///
/// store.dispatch(Action { /* ... */ }); // State is not cloned.
///
/// // The reactor now holds a reference to the current state.
///
/// store.dispatch(Action { /* ... */ }); // State is cloned.
///
/// // Replace the reactor by an empty one.
/// let mut reactor = store.subscribe(Reactor::<Error = _>::from_sink(vec![]));
///
/// // Consume all references to the state.
/// while let Some(s) = reactor.pop() {
///     // Consume `s`.
/// }
///
/// store.dispatch(Action { /* ... */ }); // State is not cloned.
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

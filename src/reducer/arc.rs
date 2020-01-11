use crate::reducer::*;
use std::sync::Arc;

/// Enhances a potentially _unsized_ [`Reducer`] with copy-on-write semantics (requires [`std`]).
///
/// Helps avoiding cloning the entire state when it needs to be sent to other threads,
/// e.g to the rendering thread of a GUI.
///
/// [`std`]: index.html#optional-features
///
/// # Example
///
/// ```rust
/// use reducer::*;
/// use std::sync::Arc;
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
/// let (tx, mut rx) = futures::channel::mpsc::channel(10);
///
/// let state = Arc::new(State { /* ... */ });
/// let reactor = Reactor::<Error = _>::from_sink(tx);
///
/// let mut store = Store::new(state, reactor);
///
/// store.dispatch(Action { /* ... */ }); // State is not cloned.
///
/// // The channel now holds a reference to the current state.
///
/// store.dispatch(Action { /* ... */ }); // State is cloned.
///
/// // Drain the channel so that it doesn't hold any references to the current state.
/// while let Ok(Some(s)) = rx.try_next() {
///     // Consume `s`.
/// }
///
/// store.dispatch(Action { /* ... */ }); // State is not cloned.
/// ```
impl<A, T> Reducer<A> for Arc<T>
where
    T: Reducer<A> + Clone + ?Sized,
{
    fn reduce(&mut self, action: A) {
        Arc::make_mut(self).reduce(action);
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

            let mut reducer = Arc::new(mock);
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

            let mut reducer = Arc::new(mock);
            let other = reducer.clone();
            Reducer::reduce(&mut reducer, action);
            drop(other);
        }
    }
}

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
    use crate::mock::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn ok(actions: Vec<u8>) {
            let mut reducer = Rc::new(Mock::<_>::default());

            for (i, &action) in actions.iter().enumerate() {
                reduce(&mut reducer, action);
                assert_eq!(reducer.calls(), &actions[0..=i]);
            }
        }
    }

    proptest! {
        #[test]
        fn cow([a, b, c]: [u8; 3]) {
            let mut reducer = Rc::new(Mock::<_>::default());

            reduce(&mut reducer, a);
            assert_eq!(reducer.calls(), &[a]);
            assert_eq!(reducer.generation(), 0);

            let other = reducer.clone();

            assert_eq!(other.generation(), 0);
            assert_eq!(reducer.generation(), 0);

            reduce(&mut reducer, b);
            assert_eq!(reducer.calls(), &[a, b]);
            assert_eq!(reducer.generation(), 1);

            assert_eq!(other.calls(), &[a]);
            assert_eq!(other.generation(), 0);

            reduce(&mut reducer, c);
            assert_eq!(reducer.calls(), &[a, b, c]);
            assert_eq!(reducer.generation(), 1);

            assert_eq!(other.calls(), &[a]);
            assert_eq!(other.generation(), 0);
        }
    }
}

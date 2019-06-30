use crate::reducer::*;
use std::sync::Arc;

/// Enhances a potentially _unsized_ [`Reducer`] with copy-on-write semantics (requires [`std`]).
///
/// Helps avoiding cloning the entire state when it needs to be sent to other threads,
/// e.g to the rendering thread of a GUI.
///
/// [`std`]: index.html#optional-features
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
    use crate::mock::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn ok(actions: Vec<u8>) {
            let mut reducer = Arc::new(Mock::<_>::default());

            for (i, &action) in actions.iter().enumerate() {
                reduce(&mut reducer, action);
                assert_eq!(reducer.calls(), &actions[0..=i]);
            }
        }
    }

    proptest! {
        #[test]
        fn cow([a, b, c]: [u8; 3]) {
            let mut reducer = Arc::new(Mock::<_>::default());

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

use crate::reducer::*;
use std::sync::Arc;

/// Enhances a potentially _unsized_ [`Reducer`] with copy-on-write semantics.
///
/// Helps avoiding cloning the entire state when it needs to be sent to other threads,
/// e.g to the rendering thread of a GUI.
impl<A, R> Reducer<A> for Arc<R>
where
    R: Reducer<A> + Clone + ?Sized,
{
    fn reduce(&mut self, action: A) {
        Arc::make_mut(self).reduce(action);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::*;
    use proptest::*;

    proptest! {
        #[test]
        fn arc(actions: Vec<u8>) {
            let mut reducer = Arc::new(Mock::default());

            for (i, &action) in actions.iter().enumerate() {
                reduce(&mut reducer, action);
                assert_eq!(reducer.calls(), &actions[0..=i]);
            }
        }
    }
}

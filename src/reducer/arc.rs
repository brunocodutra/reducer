use crate::reducer::*;
use std::sync::Arc;

/// Lazy copy-on-write for multi-threaded applications.
///
/// Helps avoiding cloning the entire state when it needs to be sent to a different thread
/// (e.g to the rendering thread of a GUI).
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
        fn reduce(actions: Vec<u8>) {
            let mut reducer = Arc::new(MockReducer::default());

            for (i, &action) in actions.iter().enumerate() {
                reducer.reduce(action);
                assert_eq!(reducer, Arc::new(MockReducer::new(actions[0..=i].into())));
            }
        }
    }
}

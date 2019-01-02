use reducer::*;
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
pub(crate) mod tests {
    use super::*;

    #[test]
    fn reduce() {
        let mut state = Arc::new(MockReducer::default());

        state.reduce(5);
        state.reduce(1);
        state.reduce(3);

        assert_eq!(state, Arc::new(MockReducer::new(vec![5, 1, 3])));
    }
}

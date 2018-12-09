use reducer::*;
use std::sync::Arc;

impl<R> Reducer for Arc<R>
where
    R: Reducer + Clone + ?Sized,
{
    type Action = R::Action;

    fn reduce(&mut self, action: Self::Action) {
        Arc::make_mut(self).reduce(action);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn arc() {
        let mut state = Arc::new(MockReducer::default());

        state.reduce(5);
        state.reduce(1);
        state.reduce(3);

        assert_eq!(state, Arc::new(MockReducer::new(vec![5, 1, 3])));
    }
}

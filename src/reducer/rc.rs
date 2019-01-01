use reducer::*;
use std::rc::Rc;

/// Lazy copy-on-write for single-threaded applications.
impl<R> Reducer for Rc<R>
where
    R: Reducer + Clone + ?Sized,
{
    type Action = R::Action;

    fn reduce(&mut self, action: Self::Action) {
        Rc::make_mut(self).reduce(action);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn arc() {
        let mut state = Rc::new(MockReducer::default());

        state.reduce(5);
        state.reduce(1);
        state.reduce(3);

        assert_eq!(state, Rc::new(MockReducer::new(vec![5, 1, 3])));
    }
}

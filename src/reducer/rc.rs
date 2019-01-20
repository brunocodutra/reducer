use reducer::*;
use std::rc::Rc;

/// Lazy copy-on-write for single-threaded applications.
impl<A, R> Reducer<A> for Rc<R>
where
    R: Reducer<A> + Clone + ?Sized,
{
    fn reduce(&mut self, action: A) {
        Rc::make_mut(self).reduce(action);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mock::*;

    #[test]
    fn reduce() {
        let mut state = Rc::new(MockReducer::default());

        state.reduce(5);
        state.reduce(1);
        state.reduce(3);

        assert_eq!(state, Rc::new(MockReducer::new(vec![5, 1, 3])));
    }
}

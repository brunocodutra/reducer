use crate::reducer::*;
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
    use crate::mock::*;
    use proptest::*;

    proptest! {
        #[test]
        fn reduce(actions: Vec<u8>) {
            let mut reducer = Rc::new(MockReducer::default());

            for (i, &action) in actions.iter().enumerate() {
                reducer.reduce(action);
                assert_eq!(reducer, Rc::new(MockReducer::new(actions[0..=i].into())));
            }
        }
    }
}

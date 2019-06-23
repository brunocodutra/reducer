use crate::reducer::*;
use std::rc::Rc;

/// Enhances a potentially _unsized_ [`Reducer`] with copy-on-write semantics.
///
/// Helps avoiding cloning the entire state when it needs to be sent to other parts of the
/// application.
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
        fn rc(actions: Vec<u8>) {
            let mut reducer = Rc::new(Mock::default());

            for (i, &action) in actions.iter().enumerate() {
                reduce(&mut reducer, action);
                assert_eq!(reducer.calls(), &actions[0..=i]);
            }
        }
    }
}

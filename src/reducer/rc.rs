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
        fn reduce(actions: Vec<u8>) {
            let mut reducer = Rc::new(Mock::default());

            for (i, &action) in actions.iter().enumerate() {
                reducer.reduce(action);
                assert_eq!(reducer, Rc::new(Mock::new(&actions[0..=i])));
            }
        }
    }
}

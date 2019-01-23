use crate::reducer::*;

/// Updates the potentially _unsized_ nested reducer.
impl<A, R> Reducer<A> for Box<R>
where
    R: Reducer<A> + Clone + ?Sized,
{
    fn reduce(&mut self, action: A) {
        (**self).reduce(action);
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
            let mut reducer = Box::new(MockReducer::default());

            for (i, &action) in actions.iter().enumerate() {
                reducer.reduce(action);
                assert_eq!(reducer, Box::new(MockReducer::new(actions[0..=i].into())));
            }
        }
    }
}

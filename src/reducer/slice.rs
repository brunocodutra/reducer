use crate::reducer::*;

/// Updates all reducers in the slice in order.
impl<A, R> Reducer<A> for [R]
where
    A: Clone,
    R: Reducer<A>,
{
    fn reduce(&mut self, action: A) {
        for reducer in self {
            reducer.reduce(action.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::*;
    use proptest::*;

    proptest! {
        #[test]
        fn reduce(actions: Vec<u8>, len in 0..100usize) {
            let reducer: &mut [MockReducer<_>] = &mut vec![MockReducer::default(); len];

            for (i, &action) in actions.iter().enumerate() {
                reducer.reduce(action);
                assert_eq!(reducer, &*vec![MockReducer::new(actions[0..=i].into()); len]);
            }
        }
    }
}

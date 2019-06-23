use crate::reducer::*;

/// Updates all [`Reducer`]s in the slice in order.
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
    use crate::mock::*;
    use proptest::*;

    proptest! {
        #[test]
        fn slice(actions: Vec<u8>, len in 0..100usize) {
            let reducers: &mut [Mock<_>] = &mut vec![Mock::default(); len];

            for (i, &action) in actions.iter().enumerate() {
                reduce(reducers, action);

                for reducer in reducers.iter() {
                    assert_eq!(reducer.calls(), &actions[0..=i])
                }
            }
        }
    }
}

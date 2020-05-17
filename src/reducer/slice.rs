#![cfg(feature = "deprecated")]

use crate::reducer::*;

/// Updates all [`Reducer`]s in the slice in order.
///
/// **Warning: this implementation is deprecated and will be removed in a future release.**
#[deprecated]
impl<A, T> Reducer<A> for [T]
where
    A: Clone,
    T: Reducer<A>,
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
    use mockall::predicate::*;
    use proptest::prelude::*;
    use std::vec::Vec;

    proptest! {
        #[test]
        fn reduce(action: u8, mut results: Vec<()>) {
            let mut mocks: Vec<_> = results
                .drain(..)
                .map(|r| {
                    let mut mock = MockReducer::new();

                    #[allow(clippy::unit_arg)]
                    mock.expect_reduce()
                        .with(eq(action))
                        .times(1)
                        .return_const(r);

                    mock
                })
                .collect();

            let reducer = mocks.as_mut_slice();
            Reducer::reduce(reducer, action);
        }
    }
}

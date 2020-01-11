use crate::reducer::*;

/// Updates the potentially _unsized_ nested [`Reducer`] (requires [`std`]).
///
/// [`std`]: index.html#optional-features
impl<A, T> Reducer<A> for Box<T>
where
    T: Reducer<A> + ?Sized,
{
    fn reduce(&mut self, action: A) {
        (**self).reduce(action);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn reduce(action: u8) {
            let mut mock = MockReducer::new();

            mock.expect_reduce()
                .with(eq(action))
                .times(1)
                .return_const(());

            let mut reducer = Box::new(mock);
            Reducer::reduce(&mut reducer, action);
        }
    }
}

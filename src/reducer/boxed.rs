use crate::reducer::*;
use alloc::boxed::Box;

/// Updates the potentially _unsized_ nested [`Reducer`] (requires [`alloc`]).
///
/// [`alloc`]: index.html#optional-features
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
    use test_strategy::proptest;

    #[proptest]
    fn reduce(action: u8) {
        let mut mock = MockReducer::new();

        mock.expect_reduce()
            .with(eq(action))
            .once()
            .return_const(());

        let mut reducer = Box::new(mock);
        Reducer::reduce(&mut reducer, action);
    }
}

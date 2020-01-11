use crate::reducer::*;

/// Forwards the event to a potentially stack allocated [`Reducer`].
impl<'a, A, T> Reducer<A> for &'a mut T
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

            let mut reducer = &mut mock;
            Reducer::reduce(&mut reducer, action);
        }
    }
}

use crate::reducer::*;

/// Updates the potentially _unsized_ nested [`Reducer`].
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
    use crate::mock::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn ok(actions: Vec<u8>) {
            let mut reducer = Box::new(Mock::<_>::default());

            for (i, &action) in actions.iter().enumerate() {
                reduce(&mut reducer, action);
                assert_eq!(reducer.calls(), &actions[0..=i]);
            }
        }
    }
}

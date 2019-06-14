use crate::reducer::*;

/// Updates the potentially _unsized_ nested [`Reducer`].
impl<A, R> Reducer<A> for Box<R>
where
    R: Reducer<A> + ?Sized,
{
    fn reduce(&mut self, action: A) {
        (**self).reduce(action);
    }
}

#[cfg(test)]
mod tests {
    use crate::mock::*;
    use proptest::*;

    proptest! {
        #[test]
        fn boxed(actions: Vec<u8>) {
            let mut reducer = Box::new(Mock::default());

            for (i, &action) in actions.iter().enumerate() {
                reduce(&mut reducer, action);
                assert_eq!(reducer, Box::new(Mock::new(&actions[0..=i])));
            }
        }
    }
}

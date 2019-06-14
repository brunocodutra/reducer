use crate::reducer::*;

/// Forwards the event to a potentially stack allocated [`Reducer`].
impl<'a, A, R> Reducer<A> for &'a mut R
where
    R: Reducer<A> + ?Sized,
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
            let reducer = &mut &mut Mock::default();

            for (i, &action) in actions.iter().enumerate() {
                reducer.reduce(action);
                assert_eq!(reducer, &&Mock::new(&actions[0..=i]));
            }
        }
    }
}

use crate::reactor::*;

/// Notifies all [`Reactor`]s in the slice in order.
impl<S, T> Reactor<S> for [T]
where
    T: Reactor<S>,
{
    type Output = Box<[T::Output]>;

    fn react(&self, state: &S) -> Self::Output {
        self.iter()
            .map(|r| r.react(state))
            .collect::<Vec<T::Output>>()
            .into_boxed_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::*;
    use proptest::*;

    proptest! {
        #[test]
        fn react(states: Vec<u8>, len in 0..100usize) {
            let reactor: &[Mock<_>] = &vec![Mock::default(); len];

            for (i, state) in states.iter().enumerate() {
                assert_eq!(reactor.react(state), vec![Ok(()); len].into());
                assert_eq!(reactor, &*vec![Mock::new(&states[0..=i]); len])
            }
        }
    }
}

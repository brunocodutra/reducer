use crate::reactor::*;

/// Notifies all reactors in the slice in order.
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
            let reactor: &[MockReactor<_>] = &vec![MockReactor::default(); len];

            for state in states {
                assert_eq!(reactor.react(&state), vec![state; len].into());
            }
        }
    }
}

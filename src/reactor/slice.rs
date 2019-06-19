use crate::reactor::*;

/// Notifies all [`Reactor`]s in the slice in order.
impl<S, T> Reactor<S> for [T]
where
    T: Reactor<S>,
{
    type Output = Box<[T::Output]>;

    fn react(&mut self, state: &S) -> Self::Output {
        self.iter_mut()
            .map(|r| r.react(state))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::mock::*;
    use proptest::*;

    proptest! {
        #[test]
        fn slice(states: Vec<u8>, len in 0..100usize) {
            let reactor: &mut [Mock<_>] = &mut vec![Mock::default(); len];

            for (i, state) in states.iter().enumerate() {
                assert_eq!(react(reactor, state), vec![Ok(()); len].into());
                assert_eq!(reactor, &*vec![Mock::new(&states[0..=i]); len])
            }
        }
    }
}

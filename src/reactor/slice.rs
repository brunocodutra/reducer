use crate::reactor::*;

/// Notifies all [`Reactor`]s in the slice in order.
impl<S, T> Reactor<S> for [T]
where
    T: Reactor<S>,
{
    type Error = T::Error;

    fn react(&mut self, state: &S) -> Result<(), Self::Error> {
        for reducer in self {
            reducer.react(state)?;
        }

        Ok(())
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
                assert_eq!(react(reactor, state), Ok(()));
                assert_eq!(reactor, &*vec![Mock::new(&states[0..=i]); len])
            }
        }
    }
}

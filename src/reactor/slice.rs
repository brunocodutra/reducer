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
    use proptest::prelude::*;

    prop_compose! {
        pub(crate) fn length_and_index(max: usize)
            (length in 1..=max)
            (index in 0..length, length in Just(length))
        -> (usize, usize) {
            (length, index)
        }
    }

    proptest! {
        #[test]
        fn ok(states: Vec<u8>, len in 0..=100usize) {
            let reactors: &mut [Mock<_>] = &mut vec![Mock::default(); len];

            for (i, state) in states.iter().enumerate() {
                assert_eq!(react(reactors, state), Ok(()));

                for reactor in reactors.iter() {
                    assert_eq!(reactor.calls(), &states[0..=i])
                }
            }
        }
    }

    proptest! {
        #[test]
        fn err(state: u8, error: String, (len, at) in length_and_index(100)) {
            let reactors: &mut [Mock<_, _>] = &mut vec![Mock::default(); len];
            reactors[at].fail_if(state, &error[..]);

            assert_eq!(react(reactors, &state), Err(&error[..]));

            for reactor in reactors.iter().take(at + 1) {
                assert_eq!(reactor.calls(), &[state])
            }

            for reactor in reactors.iter().skip(at + 1) {
                assert_eq!(reactor.calls(), &[])
            }
        }
    }
}

use crate::reactor::*;

/// Forwards the event if [`Some`], ignores if [`None`].
impl<S, T> Reactor<S> for Option<T>
where
    T: Reactor<S>,
{
    type Output = Option<T::Output>;

    fn react(&self, state: &S) -> Self::Output {
        match self {
            Some(r) => Some(r.react(state)),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::*;
    use proptest::*;

    proptest! {
        #[test]
        fn some(states: Vec<u8>) {
            let reactor = Some(Mock::default());

            for (i, state) in states.iter().enumerate() {
                assert_eq!(reactor.react(state), Some(Ok(())));
                assert_eq!(reactor, Some(Mock::new(&states[0..=i])))
            }
        }
    }

    proptest! {
        #[test]
        fn none(states: Vec<u8>) {
            let reactor: Option<Mock<_>> = None;

            for state in states {
                assert_eq!(reactor.react(&state), None);
                assert_eq!(reactor, None);
            }
        }
    }
}

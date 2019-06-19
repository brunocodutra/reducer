use crate::reactor::*;

/// Forwards the event to the potentially _unsized_ nested [`Reactor`].
impl<S, T> Reactor<S> for Box<T>
where
    T: Reactor<S> + ?Sized,
{
    type Output = T::Output;

    fn react(&mut self, state: &S) -> Self::Output {
        (**self).react(state)
    }
}

#[cfg(test)]
mod tests {
    use crate::mock::*;
    use proptest::*;

    proptest! {
        #[test]
        fn boxed(states: Vec<u8>) {
            let mut reactor = Box::new(Mock::default());

            for (i, state) in states.iter().enumerate() {
                assert_eq!(react(&mut reactor, state), Ok(()));
                assert_eq!(reactor, Box::new(Mock::new(&states[0..=i])))
            }
        }
    }
}

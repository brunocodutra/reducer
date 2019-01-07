use reactor::*;

impl<'a, S, T> Reactor<S> for &'a T
where
    T: Reactor<S> + ?Sized,
{
    type Output = T::Output;

    fn react(&self, state: &S) -> Self::Output {
        (*self).react(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mock::*;

    #[test]
    fn react() {
        let reactor = &MockReactor;
        let reactor = &reactor;

        assert_eq!(reactor.react(&5), 5);
        assert_eq!(reactor.react(&1), 1);
        assert_eq!(reactor.react(&3), 3);
    }
}

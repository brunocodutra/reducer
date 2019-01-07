use reactor::*;

/// Forwards the event if `Some`, ignores if `None`.
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
    use mock::*;

    #[test]
    fn some() {
        let reactor = Some(MockReactor);

        assert_eq!(reactor.react(&5), Some(5));
        assert_eq!(reactor.react(&1), Some(1));
        assert_eq!(reactor.react(&3), Some(3));
    }

    #[test]
    fn none() {
        let reactor: Option<MockReactor> = None;

        assert_eq!(reactor.react(&5), None);
        assert_eq!(reactor.react(&1), None);
        assert_eq!(reactor.react(&3), None);
    }
}

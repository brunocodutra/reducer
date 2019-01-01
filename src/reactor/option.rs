use reactor::*;

/// Forwards the event if `Some`, ignores if `None`.
impl<R, T> Reactor<R> for Option<T>
where
    T: Reactor<R>,
{
    type Error = T::Error;

    fn react(&self, state: &R) -> Result<(), Self::Error> {
        if let Some(reactor) = self {
            reactor.react(state)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn some() {
        let sbc = Some(MockReactor::default());

        assert!(sbc.react(&5).is_ok());
        assert!(sbc.react(&1).is_ok());
        assert!(sbc.react(&3).is_ok());

        assert_eq!(sbc, Some(MockReactor::new(vec![5, 1, 3])));
    }

    #[test]
    fn none() {
        let sbc: Option<MockReactor<_>> = None;

        assert!(sbc.react(&5).is_ok());
        assert!(sbc.react(&1).is_ok());
        assert!(sbc.react(&3).is_ok());

        assert_eq!(sbc, None);
    }
}

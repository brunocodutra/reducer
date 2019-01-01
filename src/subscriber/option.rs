use subscriber::*;

/// Forwards the event if `Some`, ignores if `None`.
impl<R, T> Subscriber<R> for Option<T>
where
    T: Subscriber<R>,
{
    type Error = T::Error;

    fn notify(&self, state: &R) -> Result<(), Self::Error> {
        if let Some(subscriber) = self {
            subscriber.notify(state)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn some() {
        let sbc = Some(MockSubscriber::default());

        assert!(sbc.notify(&5).is_ok());
        assert!(sbc.notify(&1).is_ok());
        assert!(sbc.notify(&3).is_ok());

        assert_eq!(sbc, Some(MockSubscriber::new(vec![5, 1, 3])));
    }

    #[test]
    fn none() {
        let sbc: Option<MockSubscriber<_>> = None;

        assert!(sbc.notify(&5).is_ok());
        assert!(sbc.notify(&1).is_ok());
        assert!(sbc.notify(&3).is_ok());

        assert_eq!(sbc, None);
    }
}

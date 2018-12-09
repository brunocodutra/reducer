use subscriber::*;

impl<R, T> Subscriber<R> for [T]
where
    T: Subscriber<R>,
{
    type Error = T::Error;

    fn notify(&self, state: &R) -> Result<(), Self::Error> {
        for sbc in self {
            sbc.notify(state)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice() {
        let sbc: &[MockSubscriber<_>] = &[Default::default(), Default::default()];

        assert!(sbc.notify(&5).is_ok());
        assert!(sbc.notify(&1).is_ok());
        assert!(sbc.notify(&3).is_ok());

        assert_eq!(
            sbc,
            [
                MockSubscriber::new(vec![5, 1, 3]),
                MockSubscriber::new(vec![5, 1, 3])
            ]
        );
    }
}

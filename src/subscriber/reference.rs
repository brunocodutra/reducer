use subscriber::*;

impl<'a, R, T> Subscriber<R> for &'a T
where
    T: Subscriber<R> + ?Sized,
{
    type Error = T::Error;

    fn notify(&self, state: &R) -> Result<(), Self::Error> {
        (*self).notify(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference() {
        let mock = &MockSubscriber::default();

        {
            let sbc: &Subscriber<_, Error = _> = &mock;

            assert!(sbc.notify(&5).is_ok());
            assert!(sbc.notify(&1).is_ok());
            assert!(sbc.notify(&3).is_ok());
        }

        assert_eq!(mock, &MockSubscriber::new(vec![5, 1, 3]));
    }
}

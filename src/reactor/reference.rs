use reactor::*;

impl<'a, R, T> Reactor<R> for &'a T
where
    T: Reactor<R> + ?Sized,
{
    type Error = T::Error;

    fn react(&self, state: &R) -> Result<(), Self::Error> {
        (*self).react(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn react() {
        let mock = &MockReactor::default();

        {
            let sbc: &Reactor<_, Error = _> = &mock;

            assert!(sbc.react(&5).is_ok());
            assert!(sbc.react(&1).is_ok());
            assert!(sbc.react(&3).is_ok());
        }

        assert_eq!(mock, &MockReactor::new(vec![5, 1, 3]));
    }
}

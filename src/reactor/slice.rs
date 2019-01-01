use reactor::*;

/// Notifies all reactors in the slice in order.
impl<R, T> Reactor<R> for [T]
where
    T: Reactor<R>,
{
    type Error = T::Error;

    fn react(&self, state: &R) -> Result<(), Self::Error> {
        for sbc in self {
            sbc.react(state)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn react() {
        let sbc: &[MockReactor<_>] = &[Default::default(), Default::default()];

        assert!(sbc.react(&5).is_ok());
        assert!(sbc.react(&1).is_ok());
        assert!(sbc.react(&3).is_ok());

        assert_eq!(
            sbc,
            [
                MockReactor::new(vec![5, 1, 3]),
                MockReactor::new(vec![5, 1, 3])
            ]
        );
    }
}

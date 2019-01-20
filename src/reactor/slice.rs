use reactor::*;

/// Notifies all reactors in the slice in order.
impl<S, T> Reactor<S> for [T]
where
    T: Reactor<S>,
{
    type Output = Box<[T::Output]>;

    fn react(&self, state: &S) -> Self::Output {
        self.iter()
            .map(|r| r.react(state))
            .collect::<Vec<T::Output>>()
            .into_boxed_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mock::*;

    #[test]
    fn react() {
        let reactor: &[MockReactor<_>] = &[MockReactor::default(); 42];

        assert_eq!(reactor.react(&5), vec![5; 42].into());
        assert_eq!(reactor.react(&1), vec![1; 42].into());
        assert_eq!(reactor.react(&3), vec![3; 42].into());
    }
}

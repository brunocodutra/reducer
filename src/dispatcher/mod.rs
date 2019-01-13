mod store;

pub use self::store::*;

/// Trait for types that allow dispatching actions.
pub trait Dispatcher<A> {
    type Output;
    fn dispatch(&mut self, action: A) -> Self::Output;
}

#[cfg(test)]
mod tests {
    use super::*;
    use mock::*;

    #[test]
    fn dispatch() {
        let dispatcher: &mut Dispatcher<_, Output = _> = &mut MockDispatcher::default();

        assert_eq!(dispatcher.dispatch(5), 5);
        assert_eq!(dispatcher.dispatch(1), 1);
        assert_eq!(dispatcher.dispatch(3), 3);
    }
}

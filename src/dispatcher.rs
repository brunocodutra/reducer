#[cfg(feature = "async")]
mod r#async;
mod store;

#[cfg(feature = "async")]
pub use self::r#async::*;
pub use self::store::*;

/// Trait for types that allow dispatching actions.
pub trait Dispatcher<A> {
    type Output;
    fn dispatch(&mut self, action: A) -> Self::Output;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::*;
    use proptest::*;

    proptest! {
        #[test]
        fn dispatch(actions: Vec<u8>) {
            let dispatcher: &mut Dispatcher<_, Output = _> = &mut MockDispatcher::default();

            for action in actions {
                assert_eq!(dispatcher.dispatch(action), action);
            }
        }
    }
}

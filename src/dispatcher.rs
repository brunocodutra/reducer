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
            let mut mock = Mock::default();

            for (i, &action) in actions.iter().enumerate() {
                let dispatcher: &mut dyn Dispatcher<_, Output = _> = &mut mock;
                assert_eq!(dispatcher.dispatch(action), Ok(()));
                assert_eq!(mock.calls(), &actions[0..=i]);
            }
        }
    }
}

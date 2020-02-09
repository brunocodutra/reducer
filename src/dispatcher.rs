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
    use mockall::{predicate::*, *};
    use proptest::prelude::*;
    use std::{boxed::Box, vec::Vec};

    mock! {
        pub(crate) Dispatcher<A: 'static, O: 'static> {}
        trait Dispatcher<A> {
            type Output = O;
            fn dispatch(&mut self, action: A) -> O;
        }
    }

    proptest! {
        #[test]
        fn dispatch(action: u8, result: u8) {
            let mut mock = MockDispatcher::<_, u8>::new();

            mock.expect_dispatch()
                .with(eq(action))
                .times(1)
                .return_const(result);

            let dispatcher: &mut dyn Dispatcher<_, Output = _> = &mut mock;
            assert_eq!(dispatcher.dispatch(action), result);
        }
    }
}

#[cfg(test)]
#[cfg(feature = "async")]
pub(crate) use self::tests::MockDispatcher;

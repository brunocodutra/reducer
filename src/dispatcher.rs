#[cfg(feature = "async")]
mod sink;
mod store;

#[cfg(feature = "async")]
pub use self::sink::*;
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
        pub Dispatcher<A: 'static, O: 'static> {}
        impl<A: 'static, O: 'static> Dispatcher<A> for Dispatcher<A, O> {
            type Output = O;
            fn dispatch(&mut self, action: A) -> O;
        }
    }

    #[cfg(feature = "async")]
    use futures::Sink;

    #[cfg(feature = "async")]
    use std::{pin::Pin, task::Context, task::Poll};

    #[cfg(feature = "async")]
    impl<A: Unpin, E: Unpin> Sink<A> for MockDispatcher<A, Result<(), E>> {
        type Error = E;

        fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn start_send(self: Pin<&mut Self>, action: A) -> Result<(), Self::Error> {
            self.get_mut().dispatch(action)
        }

        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
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

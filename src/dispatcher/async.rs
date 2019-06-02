use crate::dispatcher::Dispatcher;
use futures::channel::{mpsc, oneshot};
use futures::future::{BoxFuture, FutureObj};
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use futures::task::{Spawn, SpawnError};

/// Trait for types that can spawn dispatchers as an asynchronous task (requires [`async`]).
///
/// [`async`]: index.html#experimental-features
pub trait SpawnDispatcher {
    /// Spawns a [Dispatcher] as a task that will listen to actions dispatched through the
    /// [AsyncDispatcher] returned.
    ///
    /// The task completes once [AsyncDispatcher] (or the last of its clones) is dropped.
    ///
    /// [Dispatcher]: trait.Dispatcher.html
    /// [AsyncDispatcher]: struct.AsyncDispatcher.html
    fn spawn_dispatcher<D, A>(
        &mut self,
        dispatcher: D,
    ) -> Result<AsyncDispatcher<A, D::Output>, SpawnError>
    where
        D: Dispatcher<A> + Send + 'static,
        D::Output: Send + 'static,
        A: Send + 'static;
}

impl<S> SpawnDispatcher for S
where
    S: Spawn + ?Sized,
{
    fn spawn_dispatcher<D, A>(
        &mut self,
        mut dispatcher: D,
    ) -> Result<AsyncDispatcher<A, D::Output>, SpawnError>
    where
        D: Dispatcher<A> + Send + 'static,
        D::Output: Send + 'static,
        A: Send + 'static,
    {
        let (tx, mut rx) = mpsc::channel::<(A, oneshot::Sender<D::Output>)>(0);

        self.spawn_obj(FutureObj::new(Box::new(async move {
            while let Some((action, tx)) = await!(rx.next()) {
                tx.send(dispatcher.dispatch(action)).ok();
            }
        })))?;

        Ok(AsyncDispatcher { tx })
    }
}

/// A handle that allows dispatching actions on a [spawned] [Dispatcher] (requires [`async`]).
///
/// AsyncDispatcher requires all actions to be of the same type `A`.
/// An effective way to fulfill this requirement is to define actions as `enum` variants.
///
/// This type is a just lightweight handle that may be cloned and sent to other threads.
///
/// [spawned]: trait.SpawnDispatcher.html
/// [Dispatcher]: trait.Dispatcher.html
/// [`async`]: index.html#experimental-features
///
/// ## Example
///
/// ```rust
/// use futures::executor::{block_on, ThreadPool};
/// use reducer::*;
/// use std::error::Error;
/// use std::io::{self, Write};
///
/// // The state of your app.
/// struct Calculator(i32);
///
/// // Actions the user can trigger.
/// enum Action {
///     Add(i32),
///     Sub(i32),
///     Mul(i32),
///     Div(i32),
/// }
///
/// impl Reducer<Action> for Calculator {
///     fn reduce(&mut self, action: Action) {
///         match action {
///             Action::Add(x) => self.0 += x,
///             Action::Sub(x) => self.0 -= x,
///             Action::Mul(x) => self.0 *= x,
///             Action::Div(x) => self.0 /= x,
///         }
///     }
/// }
///
/// // The user interface.
/// struct Display;
///
/// impl Reactor<Calculator> for Display {
///     type Output = io::Result<()>;
///     fn react(&self, state: &Calculator) -> Self::Output {
///         io::stdout().write_fmt(format_args!("{}\n", state.0))
///     }
/// }
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     let store = Store::new(Calculator(0), Display);
///
///     // Spin up a thread-pool.
///     let mut executor = ThreadPool::new()?;
///
///     // Process incoming actions on a background thread.
///     let mut dispatcher = executor.spawn_dispatcher(store).unwrap();
///
///     block_on(dispatcher.dispatch(Action::Add(5)))?; // displays "5"
///     block_on(dispatcher.dispatch(Action::Mul(3)))?; // displays "15"
///     block_on(dispatcher.dispatch(Action::Sub(1)))?; // displays "14"
///     block_on(dispatcher.dispatch(Action::Div(7)))?; // displays "2"
///
///     drop(dispatcher.dispatch(Action::Div(0))); // never delivered
///
///     // Allow the background thread to catch up.
///     std::thread::sleep(std::time::Duration::from_millis(500));
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AsyncDispatcher<A, O> {
    tx: mpsc::Sender<(A, oneshot::Sender<O>)>,
}

impl<A, O> Dispatcher<A> for AsyncDispatcher<A, O>
where
    A: Send + 'static,
    O: Send + 'static,
{
    /// A Future that represents asynchronously dispatched actions.
    ///
    /// _**Important:** do not rely on the actual type,_
    /// _this will become an existential type once issues with rustdoc are solved._
    type Output = BoxFuture<'static, O>;

    /// Asynchronously sends an action to the associated [spawned] [Dispatcher]
    /// and returns a future to the result.
    ///
    /// _**Important:** The action is only guaranteed to be delivered
    /// if the future is polled to completion._
    ///
    /// [spawned]: trait.SpawnDispatcher.html
    /// [Dispatcher]: trait.Dispatcher.html
    fn dispatch(&mut self, action: A) -> Self::Output {
        let mut sender = self.tx.clone();

        Box::pin(async move {
            let (tx, rx) = oneshot::channel();
            await!(sender.send((action, tx))).unwrap();
            await!(rx).unwrap()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::*;
    use futures::executor::{block_on, ThreadPool};
    use futures::future::join_all;
    use futures::task::SpawnExt;
    use proptest::*;

    proptest! {
        #[test]
        fn dispatch(actions: Vec<char>) {
            let dispatcher = MockDispatcher::default();
            let mut executor = ThreadPool::new()?;
            let mut handle = executor.spawn_dispatcher(dispatcher).unwrap();

            let futures = join_all(
                actions
                    .clone()
                    .drain(..)
                    .map(|action| handle.dispatch(action))
                    .map(|f| executor.spawn_with_handle(f))
                    .map(Result::unwrap),
            );

            drop(handle);

            assert_eq!(block_on(futures), actions);
        }
    }
}

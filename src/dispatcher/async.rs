use crate::dispatcher::Dispatcher;
use futures::channel::{mpsc, oneshot};
use futures::future::{FutureObj, TryFutureExt, UnwrapOrElse};
use futures::stream::StreamExt;
use futures::task::{Spawn, SpawnError};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// An asynchronous adapter for dispatchers
/// (requires [`async`](index.html#experimental-features)).
///
/// Once Async is [spawned](struct.Async.html#method.spawn) actions may be
/// [dispatched](trait.Dispatcher.html) on it through its [AsyncHandle](struct.AsyncHandle.html).
///
/// Async requires all actions to be of the same type `A`.
/// An effective way to fulfill this requirement, is to use an `enum` to represent actions.
///
/// ## Example
/// ```rust
/// use futures::executor::ThreadPoolBuilder;
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
///     let store = Async::new(Store::new(Calculator(0), Display));
///
///     // Spin up a thread-pool.
///     let mut executor = ThreadPoolBuilder::new().create()?;
///
///     // Process incoming actions on a background thread.
///     let mut dispatcher = store.spawn(&mut executor).unwrap();
///
///     dispatcher.dispatch(Action::Add(5)); // displays "5"
///     dispatcher.dispatch(Action::Mul(3)); // displays "15"
///     dispatcher.dispatch(Action::Sub(1)); // displays "14"
///     dispatcher.dispatch(Action::Div(7)); // displays "2"
///
///     // Allow the background thread to catch up.
///     std::thread::sleep(std::time::Duration::from_millis(500));
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Async<D: Dispatcher<A>, A> {
    inner: D,
    marker: PhantomData<A>,
}

impl<D, A> From<D> for Async<D, A>
where
    D: Dispatcher<A>,
{
    fn from(dispatcher: D) -> Self {
        Async::new(dispatcher)
    }
}

impl<D, A> Deref for Async<D, A>
where
    D: Dispatcher<A>,
{
    type Target = D;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D, A> DerefMut for Async<D, A>
where
    D: Dispatcher<A>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<D: Dispatcher<A>, A> Async<D, A> {
    /// Constructs Async given any dispatcher.
    pub fn new(inner: D) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }
}

impl<D, A> Async<D, A>
where
    D: Dispatcher<A> + Send + 'static,
    D::Output: Send + 'static,
    A: Send + 'static,
{
    /// Spawns Async onto an Executor and returns an [AsyncHandle](struct.AsyncHandle.html)
    /// that may be used to dispatch actions.
    ///
    /// The spawned Async will live as long as the handle (or one of its clones) lives.
    pub fn spawn(mut self, executor: &mut impl Spawn) -> Result<AsyncHandle<D, A>, SpawnError> {
        let (tx, mut rx) = mpsc::unbounded::<(A, oneshot::Sender<D::Output>)>();

        executor.spawn_obj(FutureObj::new(Box::new(
            async move {
                while let Some((action, tx)) = await!(rx.next()) {
                    tx.send(self.inner.dispatch(action)).ok();
                }
            },
        )))?;

        Ok(AsyncHandle { tx })
    }
}

/// A handle that allows dispatching actions on a spawned [Async](struct.Async.html)
/// (requires [`async`](index.html#experimental-features)).
///
/// As the name suggests, this is just a lightweight handle that may be cloned and passed around.
#[derive(Debug, Clone)]
pub struct AsyncHandle<D: Dispatcher<A>, A> {
    tx: mpsc::UnboundedSender<(A, oneshot::Sender<D::Output>)>,
}

impl<D, A> Dispatcher<A> for AsyncHandle<D, A>
where
    D: Dispatcher<A>,
{
    /// A type that implements `FutureExt<Output = D::Output>`.
    ///
    /// _**Important:** do not rely on the actual type,_
    /// _this will become an existential type once issues with rustdoc are solved._
    type Output = Promise<D::Output>;

    /// Asynchronously sends an action through the dispatcher managed by [Async](struct.Async.html)
    /// and returns a *promise* to its output.
    ///
    /// After this call returns, the action is guaranteed to eventually be delivered and to trigger
    /// a state transition, even if the *promise* is dropped or otherwise not polled.
    ///
    /// Once the action is received by [Async](struct.Async.html),
    /// the *promise* is fulfilled with the result of calling
    /// [`<D as Dispatcher<A>>::dispatch`](trait.Dispatcher.html#tymethod.dispatch).
    fn dispatch(&mut self, action: A) -> Self::Output {
        let (tx, rx) = oneshot::channel();
        self.tx.unbounded_send((action, tx)).unwrap();
        rx.unwrap_or_else(|_| panic!())
    }
}

type Promise<T> = UnwrapOrElse<oneshot::Receiver<T>, fn(oneshot::Canceled) -> T>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::*;
    use futures::executor::{block_on, ThreadPoolBuilder};
    use futures::future::join_all;
    use proptest::*;
    use std::error::Error;

    #[test]
    fn default() {
        let dispatcher = Async::<MockDispatcher<_>, ()>::default();
        assert_eq!(dispatcher.inner, MockDispatcher::default());
    }

    #[test]
    fn new() {
        let dispatcher = Async::new(MockDispatcher::<()>::default());
        assert_eq!(dispatcher.inner, MockDispatcher::default());
    }

    #[test]
    fn from() {
        let dispatcher = Async::from(MockDispatcher::<()>::default());
        assert_eq!(dispatcher.inner, MockDispatcher::default());
    }

    #[test]
    fn deref() {
        let dispatcher = Async::from(MockDispatcher::<()>::default());
        assert_eq!(&*dispatcher, &MockDispatcher::default());
    }

    #[test]
    fn deref_mut() {
        let mut dispatcher = Async::from(MockDispatcher::<()>::default());
        assert_eq!(&mut *dispatcher, &mut MockDispatcher::default());
    }

    #[allow(clippy::clone_on_copy)]
    #[test]
    fn clone() {
        let dispatcher = Async::from(MockDispatcher::<()>::default());
        assert_eq!(dispatcher, dispatcher.clone());
    }

    #[test]
    fn spawn() -> Result<(), Box<dyn Error>> {
        let dispatcher = Async::new(MockDispatcher::<()>::default());
        let mut executor = ThreadPoolBuilder::new().create()?;
        assert!(dispatcher.spawn(&mut executor).is_ok());
        Ok(())
    }

    proptest! {
        #[test]
        fn dispatch(actions: Vec<u8>) {
            let dispatcher = Async::new(MockDispatcher::default());
            let mut executor = ThreadPoolBuilder::new().create()?;
            let mut handle = dispatcher.spawn(&mut executor).unwrap();

            let promises: Vec<_> = actions
                .iter()
                .map(|&action| handle.dispatch(action))
                .collect();

            drop(handle);

            assert_eq!(block_on(join_all(promises)), actions);
        }
    }
}

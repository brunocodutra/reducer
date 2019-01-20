use crate::dispatcher::{Dispatcher, Store};
use crate::reactor::Reactor;
use crate::reducer::Reducer;
use futures::channel::{mpsc, oneshot};
use futures::executor::ThreadPoolBuilder;
use futures::io::Error;
use futures::stream::StreamExt;
use futures::task::{SpawnError, SpawnExt};
use std::marker::PhantomData;

/// An asynchronous and reactive state container
/// (requires [`async`](index.html#experimental-features)).
///
/// The only way to mutate the internal state managed by AsyncStore is by
/// [spawning](struct.AsyncStore.html#method.spawn) it and [dispatching](trait.Dispatcher.html)
/// actions on its [AsyncStoreHandle](struct.AsyncStoreHandle.html).
/// The associated reactor is notified upon every state transition.
///
/// All actions dispatched on AsyncStore are required to be of the same type `A`.
/// An effective way to fulfill this requirement, is to use an `enum` to represent actions.
///
/// ## Example
/// ```rust
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
///     let store = AsyncStore::new(Calculator(0), Display);
///
///     // Process incoming actions on a background thread.
///     let mut dispatcher = store.spawn_thread()?;
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
pub struct AsyncStore<R: Reducer<A>, S: Reactor<R>, A> {
    inner: Store<R, S>,
    marker: PhantomData<A>,
}

impl<R: Reducer<A>, S: Reactor<R>, A> From<Store<R, S>> for AsyncStore<R, S, A>
where
    Store<R, S>: Dispatcher<A>,
{
    fn from(store: Store<R, S>) -> Self {
        Self {
            inner: store,
            marker: PhantomData,
        }
    }
}

impl<R: Reducer<A>, S: Reactor<R>, A> Into<Store<R, S>> for AsyncStore<R, S, A> {
    fn into(self) -> Store<R, S> {
        self.inner
    }
}

impl<R: Reducer<A>, S: Reactor<R>, A> AsyncStore<R, S, A> {
    /// Constructs the store given the initial state and a reactor.
    pub fn new(state: R, reactor: S) -> Self {
        Store::new(state, reactor).into()
    }

    /// Replaces the reactor and returns the previous one.
    pub fn subscribe(&mut self, reactor: impl Into<S>) -> S {
        self.inner.subscribe(reactor)
    }
}

impl<R, S, A> AsyncStore<R, S, A>
where
    R: Reducer<A> + Send + 'static,
    S: Reactor<R> + Send + 'static,
    S::Output: Send + 'static,
    A: Send + 'static,
{
    /// Spawns the AsyncStore onto an Executor and returns an
    /// [AsyncStoreHandle](struct.AsyncStoreHandle.html) that may be used to dispatch actions.
    ///
    /// The spawned AsyncStore will live as long as the handle (or one of its clones) lives.
    pub fn spawn(
        self,
        executor: &mut impl SpawnExt,
    ) -> Result<AsyncStoreHandle<R, S, A>, SpawnError> {
        let (tx, rx) = mpsc::unbounded();
        executor.spawn(run_async(self, rx))?;
        Ok(AsyncStoreHandle { tx })
    }

    /// Spawns a new thread to run the AsyncStore and returns an
    /// [AsyncStoreHandle](struct.AsyncStoreHandle.html) that may be used to dispatch actions.
    ///
    /// The spawned AsyncStore and its associated thread will live as long as the handle
    /// (or one of its clones) lives.
    pub fn spawn_thread(self) -> Result<AsyncStoreHandle<R, S, A>, Error> {
        let mut executor = ThreadPoolBuilder::new().pool_size(1).create()?;
        Ok(self.spawn(&mut executor).unwrap())
    }
}

// Free function for now to workaround compiler issues.
async fn run_async<R, S, A, Rx>(mut store: AsyncStore<R, S, A>, mut actions: Rx)
where
    R: Reducer<A>,
    S: Reactor<R>,
    Rx: StreamExt<Item = (A, oneshot::Sender<S::Output>)> + Unpin,
{
    while let Some((action, tx)) = await!(actions.next()) {
        tx.send(store.inner.dispatch(action)).ok();
    }
}

/// A handle that allows dispatching actions on an [AsyncStore](struct.AsyncStore.html)
/// (requires [`async`](index.html#experimental-features)).
///
/// As the name suggests, this is just a lightweight handle that may be cloned and passed around.
#[derive(Debug, Clone)]
pub struct AsyncStoreHandle<R: Reducer<A>, S: Reactor<R>, A> {
    tx: mpsc::UnboundedSender<(A, oneshot::Sender<S::Output>)>,
}

impl<R, S, A> Dispatcher<A> for AsyncStoreHandle<R, S, A>
where
    R: Reducer<A>,
    S: Reactor<R>,
{
    type Output = oneshot::Receiver<S::Output>;

    /// Sends an action to the associated [AsyncStore](struct.AsyncStore.html)
    /// and returns a *promise* to the output of the reactor.
    ///
    /// Once the action is received by the [AsyncStore](struct.AsyncStore.html), its internal state
    /// is updated via [`<R as Reducer<A>>::reduce`](trait.Reducer.html#tymethod.reduce) and
    /// the *promise* is fulfilled with the result of calling
    /// [`<S as Reactor<R>>::react`](trait.Reactor.html#tymethod.react) with a reference to the
    /// new state.
    ///
    /// After this call returns, the action is guaranteed to eventually be delivered and to trigger
    /// a state transition, even if the *promise* is dropped or otherwise not polled.
    fn dispatch(&mut self, action: A) -> Self::Output {
        let (tx, rx) = oneshot::channel();
        self.tx.unbounded_send((action, tx)).unwrap();
        rx
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::*;
    use futures::executor::block_on;
    use std::error::Error;

    #[test]
    fn default() {
        let store = AsyncStore::<MockReducer<_>, MockReactor<_>, ()>::default();
        assert_eq!(store.inner, Store::default());
    }

    #[test]
    fn new() {
        let state = MockReducer::new(vec![42]);
        let reactor = MockReactor::default();
        let store = AsyncStore::<_, _, i32>::new(state.clone(), &reactor);

        assert_eq!(store.inner, Store::new(state, &reactor));
    }

    #[test]
    fn from() {
        let state = MockReducer::new(vec![42]);
        let reactor = MockReactor::default();
        let store = AsyncStore::<_, _, i32>::from(Store::new(state.clone(), &reactor));

        assert_eq!(store.inner, Store::new(state, &reactor));
    }

    #[test]
    fn into() {
        let state = MockReducer::new(vec![42]);
        let reactor = MockReactor::default();
        let store: Store<_, _> = AsyncStore::<_, _, i32>::new(state.clone(), &reactor).into();

        assert_eq!(store, AsyncStore::<_, _, i32>::new(state, &reactor).inner);
    }

    #[test]
    fn clone() {
        let store = AsyncStore::<_, _, ()>::new(MockReducer::default(), MockReactor::default());
        assert_eq!(store, store.clone());
    }

    #[test]
    fn spawn() -> Result<(), Box<dyn Error>> {
        let store = AsyncStore::<MockReducer<_>, MockReactor<_>, ()>::default();
        let mut executor = ThreadPoolBuilder::new().pool_size(2).create()?;
        assert!(store.spawn(&mut executor).is_ok());
        Ok(())
    }

    #[test]
    fn spawn_thread() {
        let store = AsyncStore::<MockReducer<_>, MockReactor<_>, ()>::default();
        assert!(store.spawn_thread().is_ok());
    }

    #[test]
    fn dispatch() -> Result<(), Box<dyn Error>> {
        let store = AsyncStore::<MockReducer<_>, MockReactor<_>, _>::default();
        let mut dispatcher = store.spawn_thread()?;

        assert_eq!(
            block_on(dispatcher.dispatch(5)),
            Ok(MockReducer::new(vec![5]))
        );

        assert_eq!(
            block_on(dispatcher.dispatch(1)),
            Ok(MockReducer::new(vec![5, 1]))
        );

        assert_eq!(
            block_on(dispatcher.dispatch(3)),
            Ok(MockReducer::new(vec![5, 1, 3]))
        );

        Ok(())
    }

    #[test]
    fn subscribe() {
        let state = MockReducer::default();
        let reactor = MockReactor::default();
        let mut store = AsyncStore::<_, _, ()>::new(state, Some(reactor));

        store.subscribe(None);
        assert_eq!(store.inner, Store::new(MockReducer::default(), None));
    }
}

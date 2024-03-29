#[cfg(feature = "alloc")]
mod arc;
#[cfg(feature = "alloc")]
mod boxed;
#[cfg(feature = "alloc")]
mod rc;
mod tuple;

/// Trait for types that represent the logical state of an application.
///
/// Perhaps a more accurate mental model for types that implement this trait is that of a
/// _state machine_, where the nodes correspond to the universe of all possible representable
/// values and the edges correspond to _actions_.
pub trait Reducer<A> {
    /// Implements the transition given the current state and an action.
    ///
    /// This method is expected to have no side effects and must never fail.
    /// In many cases, an effective way to handle illegal state transitions is to make
    /// them idempotent, that is to leave the state unchanged.
    ///
    /// # Example
    ///
    /// ```rust
    /// use reducer::Reducer;
    ///
    /// #[derive(Debug)]
    /// struct Todos(Vec<String>);
    ///
    /// // Actions
    /// struct Create(String);
    /// struct Remove(usize);
    ///
    /// impl Reducer<Create> for Todos {
    ///     fn reduce(&mut self, Create(todo): Create) {
    ///         self.0.push(todo);
    ///     }
    /// }
    ///
    /// impl Reducer<Remove> for Todos {
    ///     fn reduce(&mut self, Remove(i): Remove) {
    ///         if i < self.0.len() {
    ///             self.0.remove(i);
    ///         } else {
    ///             // Illegal transition, leave the state unchanged.
    ///         }
    ///     }
    /// }
    ///
    /// let mut todos = Todos(vec![]);
    ///
    /// todos.reduce(Create("Buy milk".to_string()));
    /// println!("{:?}", todos); // ["Buy milk"]
    ///
    /// todos.reduce(Create("Learn Reducer".to_string()));
    /// println!("{:?}", todos); // ["Buy milk", "Learn Reducer"]
    ///
    /// todos.reduce(Remove(42)); // out of bounds
    /// println!("{:?}", todos); // ["Buy milk", "Learn Reducer"]
    ///
    /// todos.reduce(Remove(0));
    /// println!("{:?}", todos); // ["Learn Reducer"]
    /// ```
    fn reduce(&mut self, action: A);
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::{predicate::*, *};
    use test_strategy::proptest;

    mock! {
        pub Reducer<A: 'static> {
            pub fn id(&self) -> usize;
        }

        impl<A: 'static> Reducer<A> for Reducer<A> {
            fn reduce(&mut self, action: A);
        }

        impl<A: 'static> Clone for Reducer<A> {
            fn clone(&self) -> Self;
        }
    }

    #[proptest]
    fn reduce(action: u8) {
        let mut mock = MockReducer::new();

        mock.expect_reduce()
            .with(eq(action))
            .once()
            .return_const(());

        let reducer: &mut dyn Reducer<_> = &mut mock;
        reducer.reduce(action);
    }
}

#[cfg(test)]
pub(crate) use self::tests::MockReducer;

#[cfg(feature = "std")]
mod arc;
mod array;
#[cfg(feature = "std")]
mod boxed;
#[cfg(feature = "std")]
mod rc;
mod reference;
mod slice;
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
    use proptest::prelude::*;
    use std::{boxed::Box, vec::Vec};

    mock! {
        pub(crate) Reducer<A: 'static> {
            fn id(&self) -> usize;
        }
        trait Reducer<A> {
            fn reduce(&mut self, action: A);
        }
        trait Clone {
            fn clone(&self) -> Self;
        }
    }

    proptest! {
        #[test]
        fn reduce(action: u8) {
            let mut mock = MockReducer::new();

            mock.expect_reduce()
                .with(eq(action))
                .times(1)
                .return_const(());

            let reducer: &mut dyn Reducer<_> = &mut mock;
            reducer.reduce(action);
        }
    }
}

#[cfg(test)]
pub(crate) use self::tests::MockReducer;

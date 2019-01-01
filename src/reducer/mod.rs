mod arc;
mod mock;
mod rc;
mod tuple;

/// Trait for types that represent the logical state of an application.
///
/// Perhaps a more accurate mental model for types that implement this trait is that of a
/// _state machine_, where the nodes correspond to the universe of all possible representable
/// values and the edges correspond to [_actions_](trait.Reducer.html#associatedtype.Action).
///
/// Types that implement this trait must be self-contained and should not depend on any external
/// state, hence the required `'static` bound.
///
/// # Splitting Up State Logic
/// Handling the entire state and its transitions in a single Reducer quickly grows out of hand for
/// any meaningful application. As the complexity of your application grows, it's a good idea to
/// break up the state into smaller independent pieces. To help assembling the pieces back
/// together, Reducer is implicitly implemented for tuples.
///
/// ## Example
/// ```rust
/// use reducer::Reducer;
///
/// struct ProductListing { /* ... */ }
/// struct ShoppingCart { /* ... */ }
///
/// #[derive(Clone)]
/// enum Action {
///     AddToCart(/* ... */),
///     // ...
/// }
///
/// impl Reducer for ProductListing {
///     type Action = Action;
///     fn reduce(&mut self, action: Self::Action) {
///         // ...
///     }
/// }
///
/// impl Reducer for ShoppingCart {
///     type Action = Action;
///     fn reduce(&mut self, action: Self::Action) {
///         // ...
///     }
/// }
///
/// let mut shop = (ProductListing { }, ShoppingCart { });
///
/// // `shop` itself implements Reducer
/// shop.reduce(Action::AddToCart( ));
/// ```

pub trait Reducer: 'static {
    /// The type that encodes all possible state transitions.
    type Action;

    /// Implements the transition given the current state and an action.
    ///
    /// This method is expected to be [pure](https://en.wikipedia.org/wiki/Pure_function) and must
    /// never fail. In many cases, an effective way to handle illegal state transitions is to make
    /// them idempotent, that is to leave the state unchanged.
    ///
    /// # Example
    /// ```rust
    /// use reducer::Reducer;
    ///
    /// struct Todos(Vec<String>);
    ///
    /// enum Action {
    ///     Create(String),
    ///     Remove(usize),
    /// }
    ///
    /// use Action::*;
    ///
    /// impl Reducer for Todos {
    ///     type Action = Action;
    ///     fn reduce(&mut self, action: Self::Action) {
    ///         match action {
    ///             Create(todo) => self.0.push(todo),
    ///             Remove(i) if i < self.0.len() => {
    ///                 self.0.remove(i);
    ///             },
    ///             _ => {
    ///                 // Illegal transition,
    ///                 // leave the state unchanged.
    ///             }
    ///         }
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let mut todos = Todos(vec![]);
    ///
    ///     todos.reduce(Create("Buy milk".to_string()));
    ///     // => ["Buy milk"]
    ///
    ///     todos.reduce(Create("Learn Reducer".to_string()));
    ///     // => ["Buy milk", "Learn Reducer"]
    ///
    ///     todos.reduce(Remove(42));
    ///     // => ["Buy milk", "Learn Reducer"]
    ///
    ///     todos.reduce(Remove(0));
    ///     // => ["Learn Reducer"]
    /// }
    /// ```
    fn reduce(&mut self, action: Self::Action);
}

#[cfg(test)]
pub use self::mock::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduce() {
        let mut mock = MockReducer::default();

        {
            let state: &mut Reducer<Action = _> = &mut mock;

            state.reduce(5);
            state.reduce(1);
            state.reduce(3);
        }

        assert_eq!(mock, MockReducer::new(vec![5, 1, 3]));
    }
}

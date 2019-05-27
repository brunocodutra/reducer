mod arc;
mod array;
mod boxed;
mod rc;
mod slice;
mod tuple;

/// Trait for types that represent the logical state of an application.
///
/// Perhaps a more accurate mental model for types that implement this trait is that of a
/// _state machine_, where the nodes correspond to the universe of all possible representable
/// values and the edges correspond to [_actions_].
///
/// Types that implement this trait must be self-contained and should not depend on any external
/// state, hence the required `'static` bound.
///
/// # Splitting Up State Logic
///
/// Handling the entire state and its transitions in a single Reducer quickly grows out of hand for
/// any meaningful application. As the complexity of your application grows, it's a good idea to
/// break up the state into smaller independent pieces. To help assembling the pieces back
/// together, Reducer is implicitly implemented for tuples.
///
/// [_actions_]: trait.Reducer.html#associatedtype.Action
///
/// ## Example
///
/// ```rust
/// use reducer::Reducer;
///
/// struct ProductListing { /* ... */ }
/// struct ShoppingCart { /* ... */ }
///
/// #[derive(Clone)]
/// struct AddToCart( /* ... */ );
///
/// impl Reducer<AddToCart> for ProductListing {
///     fn reduce(&mut self, action: AddToCart) {
///         // ...
///     }
/// }
///
/// impl Reducer<AddToCart> for ShoppingCart {
///     fn reduce(&mut self, action: AddToCart) {
///         // ...
///     }
/// }
///
/// let products = ProductListing { /* ... */ };
/// let cart = ShoppingCart { /* ... */ };
/// let mut shop = (products, cart);
///
/// // `shop` itself implements Reducer<AddToCart>
/// shop.reduce(AddToCart( /* ... */ ));
/// ```

pub trait Reducer<A>: 'static {
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
    /// fn main() {
    ///     let mut todos = Todos(vec![]);
    ///
    ///     todos.reduce(Create("Buy milk".to_string()));
    ///     println!("{:?}", todos); // ["Buy milk"]
    ///
    ///     todos.reduce(Create("Learn Reducer".to_string()));
    ///     println!("{:?}", todos); // ["Buy milk", "Learn Reducer"]
    ///
    ///     todos.reduce(Remove(42)); // out of bounds
    ///     println!("{:?}", todos); // ["Buy milk", "Learn Reducer"]
    ///
    ///     todos.reduce(Remove(0));
    ///     println!("{:?}", todos); // ["Learn Reducer"]
    /// }
    /// ```
    fn reduce(&mut self, action: A);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::*;
    use proptest::*;

    proptest! {
        #[test]
        fn reduce(actions: Vec<char>) {
            let mut mock = MockReducer::default();

            for (i, &action) in actions.iter().enumerate() {
                let reducer: &mut dyn Reducer<_> = &mut mock;
                reducer.reduce(action);
                assert_eq!(mock, MockReducer::new(&actions[0..=i]));
            }
        }
    }
}

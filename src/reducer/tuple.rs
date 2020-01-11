use crate::reducer::*;

macro_rules! impl_reducer_for_tuple {
    ( $($args:ident,)+ ) => {
        /// Updates all [`Reducer`]s in the tuple in order.
        ///
        /// <small>Currently implemented for tuples of up to 12 elements.</small>
        ///
        /// # Example
        ///
        /// ```rust
        /// use reducer::Reducer;
        ///
        /// struct ProductDetails { /* ... */ }
        /// struct ShoppingCart { /* ... */ }
        /// struct UserProfile { /* ... */ }
        ///
        /// #[derive(Clone)]
        /// enum Action {
        ///     ViewProduct(/* ... */),
        ///     AddToShoppingCart(/* ... */),
        ///     UpdatePaymentPreferences(/* ... */),
        ///     // ...
        /// };
        ///
        /// impl Reducer<Action> for ProductDetails {
        ///     fn reduce(&mut self, action: Action) {
        ///         // ...
        ///     }
        /// }
        ///
        /// impl Reducer<Action> for ShoppingCart {
        ///     fn reduce(&mut self, action: Action) {
        ///         // ...
        ///     }
        /// }
        ///
        /// impl Reducer<Action> for UserProfile {
        ///     fn reduce(&mut self, action: Action) {
        ///         // ...
        ///     }
        /// }
        ///
        /// let product = ProductDetails { /* ... */ };
        /// let cart = ShoppingCart { /* ... */ };
        /// let user = UserProfile { /* ... */ };
        ///
        /// let mut shop = (product, cart, user);
        ///
        /// // `shop` itself implements Reducer<Action>
        /// shop.reduce(Action::AddToShoppingCart( /* ... */ ));
        /// ```
        impl<A, $($args,)+> Reducer<A> for ($($args,)+)
        where
            A: Clone,
            $($args: Reducer<A>,)+
        {
            fn reduce(&mut self, action: A) {
                #[allow(non_snake_case)]
                let ($($args,)+) = self;
                $($args.reduce(action.clone());)+
            }
        }
    };
}

macro_rules! impl_reducer_for_tuples {
    () => {};

    ( $head:ident $(, $tail:ident)* $(,)? ) => {
        impl_reducer_for_tuples!($($tail,)*);
        reverse!(impl_reducer_for_tuple!($head $(, $tail)*));
    };
}

impl_reducer_for_tuples!(M, L, K, J, I, H, G, F, E, D, C, B);

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use proptest::prelude::*;

    macro_rules! test_reducer_for_tuples {
        () => {};

        ( $head:ident $(, $tail:ident)* $(,)? ) => {
            proptest! {
                #[test]
                fn $head(action: u8) {
                    let mut mocks: [MockReducer<_>; count!($($tail,)*) + 1] = Default::default();

                    for mock in &mut mocks {
                        mock.expect_reduce()
                            .with(eq(action))
                            .times(1)
                            .return_const(());
                    }

                    let [$head, $($tail,)*] = mocks;
                    let mut reducer = ($head, $($tail,)*);
                    Reducer::reduce(&mut reducer, action);
                }
            }

            test_reducer_for_tuples!($($tail,)*);
        };
    }

    test_reducer_for_tuples!(_12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01);
}

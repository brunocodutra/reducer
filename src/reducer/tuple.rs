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
    use crate::mock::*;
    use proptest::prelude::*;

    mod ok {
        use super::*;

        macro_rules! test_reducer_for_tuples {
            () => {};

            ( $head:ident $(, $tail:ident)* $(,)? ) => {
                type $head<T> = TaggedMock<[(); count!($($tail,)*)], T>;

                proptest! {
                    #[test]
                    fn $head(actions: Vec<u8>) {
                        let mut reducers = ($head::default(), $($tail::default(),)*);

                        for (i, &action) in actions.iter().enumerate() {
                            reduce(&mut reducers, action);

                            let ($head, $($tail,)*) = &reducers;

                            assert_eq!($head.calls(), &actions[0..=i]);
                            $(assert_eq!($tail.calls(), &actions[0..=i]);)*
                        }
                    }
                }

                test_reducer_for_tuples!($($tail,)*);
            };
        }

        test_reducer_for_tuples!(_12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01);
    }
}

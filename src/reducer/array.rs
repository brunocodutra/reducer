use crate::reducer::*;

macro_rules! impl_reducer_for_arrays {
    () => {};

    ( $head:ident $(, $tail:ident)* $(,)? ) => {
        impl_reducer_for_arrays!($($tail,)*);

        /// Updates all [`Reducer`]s in the array in order.
        ///
        /// Currently implemented for arrays of up to 32 elements.
        impl<A, T> Reducer<A> for [T; count!($($tail,)*)]
        where
            A: Clone,
            T: Reducer<A>,
        {
            fn reduce(&mut self, action: A) {
                self[..].reduce(action);
            }
        }
    };
}

impl_reducer_for_arrays!(
    _32, _31, _30, _29, _28, _27, _26, _25, _24, _23, _22, _21, _20, _19, _18, _17, _16, _15, _14,
    _13, _12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01, _00
);

#[cfg(test)]
mod tests {
    use crate::mock::*;
    use proptest::prelude::*;

    mod ok {
        use super::*;

        macro_rules! test_reducer_for_arrays {
            () => {};

            ( $head:ident $(, $tail:ident)* $(,)? ) => {
                proptest! {
                    #[test]
                    fn $head(actions: Vec<u8>) {
                        let mut reducers: [Mock<_>; count!($($tail,)*)] = Default::default();

                        for (i, &action) in actions.iter().enumerate() {
                            reduce(&mut reducers, action);

                            for reducer in &reducers {
                                assert_eq!(reducer.calls(), &actions[0..=i])
                            }
                        }
                    }
                }

                test_reducer_for_arrays!($( $tail,)*);
            };
        }

        test_reducer_for_arrays!(
            _32, _31, _30, _29, _28, _27, _26, _25, _24, _23, _22, _21, _20, _19, _18, _17, _16,
            _15, _14, _13, _12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01, _00
        );
    }
}

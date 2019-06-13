use crate::reducer::*;

macro_rules! impl_reducer_for_array {
    () => {};

    ( $head:ident $(, $tail:ident )* $(,)? ) => {
        dedupe_docs!(($( $tail, )*),
            /// Updates all [`Reducer`]s in the array in order.
            ///
            /// Currently implemented for arrays of up to 32 elements.
            impl<A, R> Reducer<A> for [R; count!($( $tail, )*)]
            where
                A: Clone,
                R: Reducer<A>,
            {
                fn reduce(&mut self, _action: A) {
                    let [$( $tail, )*] = self;
                    $( $tail.reduce(_action.clone()); )*
                }
            }
        );

        impl_reducer_for_array!($($tail, )*);
    };
}

impl_reducer_for_array!(
    _32, _31, _30, _29, _28, _27, _26, _25, _24, _23, _22, _21, _20, _19, _18, _17, _16, _15, _14,
    _13, _12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01, _00
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::*;
    use proptest::*;

    macro_rules! test_reducer_for_array {
        () => {};

        ( $head:ident $(, $tail:ident )* $(,)? ) => {
            proptest!(|(actions: Vec<u8>)| {
                let mut reducers: [MockReducer<_>; count!($( $tail, )*)] = Default::default();

                for (_i, &action) in actions.iter().enumerate() {
                    reducers.reduce(action);

                    assert_eq!(
                        reducers,
                        [$( always!($tail, MockReducer::new(&actions[0..=_i])), )*]
                    );
                }
            });

            test_reducer_for_array!($( $tail, )*);
        };
    }

    #[allow(clippy::cognitive_complexity)]
    #[test]
    fn reduce() {
        test_reducer_for_array!(
            _32, _31, _30, _29, _28, _27, _26, _25, _24, _23, _22, _21, _20, _19, _18, _17, _16,
            _15, _14, _13, _12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01, _00
        );
    }
}

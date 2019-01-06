use reactor::*;

macro_rules! impl_reactor_for_array {
    () => {};

    ( $head:ident $(, $tail:ident )* $(,)* ) => {
        dedupe_docs!(($( $tail, )*),
            /// Notifies all reactors in the array in order.
            ///
            /// Currently implemented for arrays of up to 32 elements.
            impl<S, T> Reactor<S> for [T; count!($( $tail, )*)]
            where
                T: Reactor<S>,
            {
                type Output = [T::Output; count!($( $tail, )*)];

                fn react(&self, _state: &S) -> Self::Output {
                    let [$( $tail, )*] = self;
                    [$( $tail.react(_state), )*]
                }
            }
        );

        impl_reactor_for_array!($($tail, )*);
    };
}

impl_reactor_for_array!(
    _32, _31, _30, _29, _28, _27, _26, _25, _24, _23, _22, _21, _20, _19, _18, _17, _16, _15, _14,
    _13, _12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01, _00
);

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_reactor_for_array {
        () => {};

        ( $head:ident $(, $tail:ident )* $(,)* ) => {
            #[test]
            fn $head() {
                let reactors = [MockReactor; count!($( $tail, )*)];

                assert_eq!(reactors.react(&5), [5; count!($( $tail, )*)]);
                assert_eq!(reactors.react(&1), [1; count!($( $tail, )*)]);
                assert_eq!(reactors.react(&3), [3; count!($( $tail, )*)]);
            }

            test_reactor_for_array!($( $tail, )*);
        };
    }

    test_reactor_for_array!(
        _32, _31, _30, _29, _28, _27, _26, _25, _24, _23, _22, _21, _20, _19, _18, _17, _16, _15,
        _14, _13, _12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01, _00
    );
}

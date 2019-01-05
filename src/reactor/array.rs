use reactor::*;

macro_rules! count {
    () => { 0 };

    ( $head:ident $(, $tail:ident )* $(,)* ) => { (1 + count!($($tail, )*)) };
}

macro_rules! impl_reactor_for_array {
    () => {
        /// Notifies all reactors in the array in order.
        ///
        /// Currently implemented for arrays of up to 32 elements.
        impl<S, T> Reactor<S> for [T; 0]
        where
            T: Reactor<S>,
        {
            type Output = [T::Output; 0];

            fn react(&self, _state: &S) -> Self::Output {
                []
            }
        }
    };

    ( $head:ident $(, $tail:ident )* $(,)* ) => {
        #[doc(hidden)]
        impl<S, T> Reactor<S> for [T; count!($head $(, $tail )*)]
        where
            T: Reactor<S>,
        {
            type Output = [T::Output; count!($head $(, $tail )*)];

            fn react(&self, state: &S) -> Self::Output {
                let [$head, $( $tail, )*] = self;
                [$head.react(state), $( $tail.react(state), )*]
            }
        }

        impl_reactor_for_array!($($tail, )*);
    };
}

impl_reactor_for_array!(
    _32, _31, _30, _29, _28, _27, _26, _25, _24, _23, _22, _21, _20, _19, _18, _17, _16, _15, _14,
    _13, _12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01
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
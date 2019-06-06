use crate::reactor::*;

macro_rules! impl_reactor_for_array {
    () => {};

    ( $head:ident $(, $tail:ident )* $(,)? ) => {
        dedupe_docs!(($( $tail, )*),
            /// Notifies all [`Reactor`]s in the array in order.
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
    use crate::mock::*;
    use proptest::*;

    macro_rules! test_reactor_for_array {
        () => {};

        ( $head:ident $(, $tail:ident )* $(,)? ) => {
            type $head<S> = MockReactor<S>;

            proptest!(|(states: Vec<char>)| {
                let reactors: [MockReactor<_>; count!($( $tail, )*)] = Default::default();

                for (_i, state) in states.iter().enumerate() {
                    assert_eq!(reactors.react(state), [Ok(()); count!($( $tail, )*)]);
                    assert_eq!(reactors, [$( $tail::new(&states[0..=_i]), )*]);
                }
            });

            test_reactor_for_array!($( $tail, )*);
        };
    }

    #[allow(clippy::cognitive_complexity)]
    #[test]
    fn react() {
        test_reactor_for_array!(
            _32, _31, _30, _29, _28, _27, _26, _25, _24, _23, _22, _21, _20, _19, _18, _17, _16,
            _15, _14, _13, _12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01, _00
        );
    }
}

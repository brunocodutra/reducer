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
                type Error = T::Error;

                fn react(&mut self, state: &S) -> Result<(), Self::Error> {
                    self[..].react(state)
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
    use crate::mock::*;
    use proptest::prelude::*;

    mod ok {
        use super::*;

        macro_rules! test_reactor_for_array {
            () => {};

            ( $head:ident $(, $tail:ident )* $(,)? ) => {
                proptest! {
                    #[test]
                    fn $head(states: Vec<u8>) {
                        let mut reactors: [Mock<_>; count!($( $tail, )*)] = Default::default();

                        for (i, state) in states.iter().enumerate() {
                            assert_eq!(react(&mut reactors, state), Ok(()));

                            for reactor in &reactors {
                                assert_eq!(reactor.calls(), &states[0..=i])
                            }
                        }
                    }
                }

                test_reactor_for_array!($( $tail, )*);
            };
        }

        test_reactor_for_array!(
            _32, _31, _30, _29, _28, _27, _26, _25, _24, _23, _22, _21, _20, _19, _18, _17, _16,
            _15, _14, _13, _12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01, _00
        );
    }

    mod err {
        use super::*;

        macro_rules! test_reactor_for_array {
            () => {};

            ( $head:ident $(, $tail:ident )* $(,)? ) => {
                proptest! {
                    #[test]
                    fn $head(state: u8, error: String, at in 0usize..=count!($( $tail, )*)) {
                        let mut reactors: [Mock<_, _>; count!($( $tail, )*) + 1] = Default::default();
                        reactors[at].fail_if(state, &error[..]);

                        assert_eq!(react(&mut reactors, &state), Err(&error[..]));

                        for reactor in reactors.iter().take(at + 1) {
                            assert_eq!(reactor.calls(), &[state])
                        }

                        for reactor in reactors.iter().skip(at + 1) {
                            assert_eq!(reactor.calls(), &[])
                        }
                    }
                }

                test_reactor_for_array!($( $tail, )*);
            };
        }

        test_reactor_for_array!(
            _32, _31, _30, _29, _28, _27, _26, _25, _24, _23, _22, _21, _20, _19, _18, _17, _16,
            _15, _14, _13, _12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01
        );
    }
}

use crate::reactor::*;

macro_rules! impl_reactor_for_tuples {
    () => {};

    ( $head:ident $(, $tail:ident )* $(,)? ) => {
        dedupe_docs!(($( $tail, )*),
            /// Notifies all [`Reactor`]s in the tuple in order.
            ///
            /// Currently implemented for tuples of up to 12 elements.
            impl<S, E, $head, $( $tail, )*> Reactor<S> for ($head, $( $tail, )*)
            where
                $head: Reactor<S, Error = E>,
                $( $tail: Reactor<S, Error = E>, )*
            {
                type Error = E;

                fn react(&mut self, state: &S) -> Result<(), Self::Error> {
                    let ($head, $( $tail, )*) = self;
                    $head.react(state)?;
                    $( $tail.react(state)?; )*
                    Ok(())
                }
            }
        );

        impl_reactor_for_tuples!($( $tail, )*);
    };
}

impl_reactor_for_tuples!(_12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01);

#[cfg(test)]
mod tests {
    use crate::mock::*;
    use proptest::prelude::*;

    mod ok {
        use super::*;

        macro_rules! test_reactor_for_tuples {
            () => {};

            ( $head:ident $(, $tail:ident )* $(,)? ) => {
                type $head<T> = TaggedMock<[(); count!($($tail,)*)], T>;

                proptest! {
                    #[test]
                    fn $head(states: Vec<u8>) {
                        let mut reactors = ($head::default(), $( $tail::default(), )*);

                        for (i, state) in states.iter().enumerate() {
                            assert_eq!(react(&mut reactors, state), Ok(()));

                            let ($head, $( $tail, )*) = &reactors;

                            assert_eq!($head.calls(), &states[0..=i]);
                            $( assert_eq!($tail.calls(), &states[0..=i]); )*
                        }
                    }
                }

                test_reactor_for_tuples!($( $tail, )*);
            };
        }

        test_reactor_for_tuples!(_12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01);
    }

    mod err {
        use super::*;

        macro_rules! test_reactor_for_tuples {
            () => {};

            ( $head:ident $(, $tail:ident )* $(,)? ) => {
                proptest! {
                    #[test]
                    fn $head(state: u8, error: String, at in 0usize..=count!($( $tail, )*)) {
                        let mut mocks: [Mock<_, _>; count!($( $tail, )*) + 1] = Default::default();
                        mocks[at].fail_if(state, &error[..]);

                        {
                            let [$head, $( $tail, )*] = &mut mocks;
                            let mut reactors = ($head, $( $tail, )*);

                            assert_eq!(react(&mut reactors, &state), Err(&error[..]));
                        }

                        for mock in mocks.iter().take(at + 1) {
                            assert_eq!(mock.calls(), &[state])
                        }

                        for mock in mocks.iter().skip(at + 1) {
                            assert_eq!(mock.calls(), &[])
                        }
                    }
                }

                test_reactor_for_tuples!($( $tail, )*);
            };
        }

        test_reactor_for_tuples!(_12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01);
    }
}

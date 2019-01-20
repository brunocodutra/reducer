use crate::reactor::*;

macro_rules! impl_reactor_for_tuples {
    () => {};

    ( $head:ident $(, $tail:ident )* $(,)? ) => {
        dedupe_docs!(($( $tail, )*),
            /// Notifies all reactors in the tuple in order.
            ///
            /// Currently implemented for tuples of up to 12 elements.
            impl<S, $head, $( $tail, )*> Reactor<S> for ($head, $( $tail, )*)
            where
                $head: Reactor<S>,
                $( $tail: Reactor<S>, )*
            {
                type Output = ($head::Output, $( $tail::Output, )*);

                fn react(&self, state: &S) -> Self::Output {
                    let ($head, $( $tail, )*) = self;
                    ($head.react(state), $( $tail.react(state), )*)
                }
            }
        );

        impl_reactor_for_tuples!($( $tail, )*);
    };
}

impl_reactor_for_tuples!(_12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::*;
    use proptest::*;

    macro_rules! test_reactor_for_tuples {
        () => {};

        ( $head:ident $(, $tail:ident )* $(,)? ) => {
            #[derive(Debug, Default, Clone, Eq, PartialEq)]
            struct $head<S: Clone> {
                value: S,
            }

            impl<S: Clone> $head<S> {
                fn new(value: S) -> Self {
                    $head { value }
                }
            }

            impl<S, T: Clone + Default> Reactor<S> for $head<T>
                where
                    MockReactor<T>: Reactor<S, Output = T>,
            {
                type Output = Self;

                fn react(&self, state: &S) -> Self::Output {
                    $head::new(MockReactor::default().react(state))
                }
            }

            proptest!(|(states: Vec<u8>)| {
                let reactor = ($head::default(), $( $tail::default(), )*);

                for state in states {
                    let expected = ($head::new(state), $( $tail::new(state), )*);
                    assert_eq!(reactor.react(&state), expected);
                }
            });

            test_reactor_for_tuples!($( $tail, )*);
        };
    }

    #[test]
    fn react() {
        test_reactor_for_tuples!(_12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01);
    }
}

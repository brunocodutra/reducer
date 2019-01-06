use reactor::*;

macro_rules! impl_reactor_for_tuples {
    () => {};

    ( $head:ident $(, $tail:ident )* $(,)* ) => {
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

    macro_rules! test_reactor_for_tuples {
        () => {};

        ( $head:ident $(, $tail:ident )* $(,)* ) => {
            #[derive(Debug, Default, Clone, Eq, PartialEq)]
            struct $head<S: Clone> {
                value: S,
            }

            impl<S: Clone> $head<S> {
                fn new(value: S) -> Self {
                    $head { value }
                }
            }

            impl<S: Clone> Reactor<S> for $head<S> {
                type Output = Self;

                fn react(&self, state: &S) -> Self::Output {
                    $head::new(state.clone())
                }
            }

            #[test]
            fn $head() {
                let reactor = ($head::default(), $( $tail::default(), )*);

                assert_eq!(reactor.react(&5), ($head::new(5), $( $tail::new(5), )*));
                assert_eq!(reactor.react(&1), ($head::new(1), $( $tail::new(1), )*));
                assert_eq!(reactor.react(&3), ($head::new(3), $( $tail::new(3), )*));
            }

            test_reactor_for_tuples!($( $tail, )*);
        };
    }

    test_reactor_for_tuples!(_12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01);
}

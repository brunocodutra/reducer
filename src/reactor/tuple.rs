use reactor::*;

macro_rules! document_reactor_for_tuples {
    ( ($head:ident), $( $body:tt )+ ) => {
        /// Notifies all reactors in the tuple in order.
        ///
        /// Currently implemented for tuples of up to 12 elements.
        $( $body )+
    };

    ( ($head:ident $(, $tail:ident )+), $( $body:tt )+ ) => {
        #[doc(hidden)]
        $( $body )+
    };
}

macro_rules! impl_reactor_for_tuples {
    () => {};

    ( $head:ident $(, $tail:ident )* $(,)* ) => {
        document_reactor_for_tuples!(($head $(, $tail )*),
            impl<R, E, $head, $( $tail, )*> Reactor<R> for ($head, $( $tail, )*)
            where
                E: Debug,
                $head: Reactor<R, Error = E>,
                $( $tail: Reactor<R, Error = E>, )*
            {
                type Error = E;

                fn react(&self, state: &R) -> Result<(), Self::Error> {
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
    use super::*;

    macro_rules! test_reactor_for_tuples {
        () => {};

        ( $head:ident $(, $tail:ident )* $(,)* ) => {
            #[derive(Debug, Default, Clone, Eq, PartialEq)]
            struct $head<R: Clone> {
                inner: MockReactor<R>,
            }

            impl<R: Clone> Reactor<R> for $head<R> {
                type Error = <MockReactor<R> as Reactor<R>>::Error;

                fn react(&self, state: &R) -> Result<(), Self::Error> {
                    self.inner.react(state)
                }
            }

            #[test]
            fn $head() {
                let sbc = ($head::default(), $( $tail::default(), )*);

                assert!(sbc.react(&5).is_ok());
                assert!(sbc.react(&1).is_ok());
                assert!(sbc.react(&3).is_ok());

                let ($head, $( $tail, )*) = sbc;

                assert_eq!($head.inner, MockReactor::new(vec![5, 1, 3]));
                $( assert_eq!($tail.inner, MockReactor::new(vec![5, 1, 3])); )*
            }

            test_reactor_for_tuples!($( $tail, )*);
        };
    }

    test_reactor_for_tuples!(_12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01);
}

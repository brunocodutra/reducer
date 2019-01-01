use subscriber::*;

macro_rules! document_subscriber_for_tuples {
    ( ($head:ident), $( $body:tt )+ ) => {
        /// Notifies all subscribers in the tuple in order.
        ///
        /// Currently implemented for tuples of up to 12 elements.
        $( $body )+
    };

    ( ($head:ident $(, $tail:ident )+), $( $body:tt )+ ) => {
        #[doc(hidden)]
        $( $body )+
    };
}

macro_rules! impl_subscriber_for_tuples {
    () => {};

    ( $head:ident $(, $tail:ident )* $(,)* ) => {
        document_subscriber_for_tuples!(($head $(, $tail )*),
            impl<R, E, $head, $( $tail, )*> Subscriber<R> for ($head, $( $tail, )*)
            where
                E: Debug,
                $head: Subscriber<R, Error = E>,
                $( $tail: Subscriber<R, Error = E>, )*
            {
                type Error = E;

                fn notify(&self, state: &R) -> Result<(), Self::Error> {
                    let ($head, $( $tail, )*) = self;
                    $head.notify(state)?;
                    $( $tail.notify(state)?; )*
                    Ok(())
                }
            }
        );

        impl_subscriber_for_tuples!($( $tail, )*);
    };
}

impl_subscriber_for_tuples!(_12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01);

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_subscriber_for_tuples {
        () => {};

        ( $head:ident $(, $tail:ident )* $(,)* ) => {
            #[derive(Debug, Default, Clone, Eq, PartialEq)]
            struct $head<R: Clone> {
                inner: MockSubscriber<R>,
            }

            impl<R: Clone> Subscriber<R> for $head<R> {
                type Error = <MockSubscriber<R> as Subscriber<R>>::Error;

                fn notify(&self, state: &R) -> Result<(), Self::Error> {
                    self.inner.notify(state)
                }
            }

            #[test]
            fn $head() {
                let sbc = ($head::default(), $( $tail::default(), )*);

                assert!(sbc.notify(&5).is_ok());
                assert!(sbc.notify(&1).is_ok());
                assert!(sbc.notify(&3).is_ok());

                let ($head, $( $tail, )*) = sbc;

                assert_eq!($head.inner, MockSubscriber::new(vec![5, 1, 3]));
                $( assert_eq!($tail.inner, MockSubscriber::new(vec![5, 1, 3])); )*
            }

            test_subscriber_for_tuples!($( $tail, )*);
        };
    }

    test_subscriber_for_tuples!(_12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01);
}

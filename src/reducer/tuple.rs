use reducer::*;

macro_rules! document_reducer_for_tuples {
    ( ($head:ident), $( $body:tt )+ ) => {
        /// Updates all reducers in the tuple in order.
        ///
        /// Currently implemented for tuples of up to 12 elements.
        $( $body )+
    };

    ( ($head:ident $(, $tail:ident )+), $( $body:tt )+ ) => {
        #[doc(hidden)]
        $( $body )+
    };
}

macro_rules! impl_reducer_for_tuples {
    () => {};

    ( $head:ident $(, $tail:ident )* $(,)* ) => {
        document_reducer_for_tuples!(($head $(, $tail )*),
            impl<A, $head, $( $tail, )*> Reducer for ($head, $( $tail, )*)
            where
                A: Clone,
                $head: Reducer<Action = A>,
                $( $tail: Reducer<Action = A>, )*
            {
                type Action = A;

                fn reduce(&mut self, action: Self::Action) {
                    let ($head, $( $tail, )*) = self;
                    $head.reduce(action.clone());
                    $( $tail.reduce(action.clone()); )*
                }
            }
        );

        impl_reducer_for_tuples!($( $tail, )*);
    };
}

impl_reducer_for_tuples!(_12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01);

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_reducer_for_tuples {
        () => {};

        ( $head:ident $(, $tail:ident )* $(,)* ) => {
            #[derive(Debug, Default, Clone, Eq, PartialEq)]
            struct $head<A: 'static> {
                inner: MockReducer<A>,
            }

            impl<A: 'static + Clone> Reducer for $head<A> {
                type Action = A;

                fn reduce(&mut self, action: A) {
                    self.inner.reduce(action);
                }
            }

            #[test]
            fn $head() {
                let mut states = ($head::default(), $( $tail::default(), )*);

                states.reduce(5);
                states.reduce(1);
                states.reduce(3);

                let ($head, $( $tail, )*) = states;

                assert_eq!($head.inner, MockReducer::new(vec![5, 1, 3]));
                $( assert_eq!($tail.inner, MockReducer::new(vec![5, 1, 3])); )*
            }

            test_reducer_for_tuples!($( $tail, )*);
        };
    }

    test_reducer_for_tuples!(_12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01);
}

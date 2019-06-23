use crate::reducer::*;

macro_rules! impl_reducer_for_tuples {
    () => {};

    ( $head:ident $(, $tail:ident )* $(,)? ) => {
        dedupe_docs!(($( $tail, )*),
            /// Updates all [`Reducer`]s in the tuple in order.
            ///
            /// Currently implemented for tuples of up to 12 elements.
            impl<A, $head, $( $tail, )*> Reducer<A> for ($head, $( $tail, )*)
            where
                A: Clone,
                $head: Reducer<A>,
                $( $tail: Reducer<A>, )*
            {
                fn reduce(&mut self, action: A) {
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
    use crate::mock::*;
    use proptest::prelude::*;

    mod ok {
        use super::*;

        macro_rules! test_reducer_for_tuples {
            () => {};

            ( $head:ident $(, $tail:ident )* $(,)? ) => {
                type $head<T> = TaggedMock<[(); count!($($tail,)*)], T>;

                proptest! {
                    #[test]
                    fn $head(actions: Vec<u8>) {
                        let mut reducers = ($head::default(), $( $tail::default(), )*);

                        for (i, &action) in actions.iter().enumerate() {
                            reduce(&mut reducers, action);

                            let ($head, $( $tail, )*) = &reducers;

                            assert_eq!($head.calls(), &actions[0..=i]);
                            $( assert_eq!($tail.calls(), &actions[0..=i]); )*
                        }
                    }
                }

                test_reducer_for_tuples!($( $tail, )*);
            };
        }

        test_reducer_for_tuples!(_12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01);
    }
}

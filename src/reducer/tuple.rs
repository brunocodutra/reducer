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
    use super::*;
    use crate::mock::*;
    use proptest::*;

    macro_rules! test_reducer_for_tuples {
        () => {};

        ( $head:ident $(, $tail:ident )* $(,)? ) => {
            type $head<T> = TaggedMock<T, [(); count!($($tail,)*)]>;

            proptest!(|(actions: Vec<u8>)| {
                let mut reducers = ($head::default(), $( $tail::default(), )*);

                for (i, &action) in actions.iter().enumerate() {
                    reducers.reduce(action);

                    assert_eq!(reducers, (
                        $head::new(&actions[0..=i]),
                        $( $tail::new(&actions[0..=i]), )*
                    ));
                }
            });

            test_reducer_for_tuples!($( $tail, )*);
        };
    }

    #[test]
    fn reduce() {
        test_reducer_for_tuples!(_12, _11, _10, _09, _08, _07, _06, _05, _04, _03, _02, _01);
    }
}

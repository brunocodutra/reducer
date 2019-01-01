use subscriber::*;

macro_rules! document_subscriber_for_array {
    ( show, $( $body:tt )+ ) => {
        /// Notifies all subscribers in the array in order.
        ///
        /// Currently implemented for arrays of up to 32 elements.
        $( $body )+
    };

    ( hide, $( $body:tt )+ ) => {
        #[doc(hidden)]
        $( $body )+
    };
}

macro_rules! impl_subscriber_for_array {
    ($size:expr, $doc:tt) => {
        document_subscriber_for_array!(
            $doc,
            impl<R, T> Subscriber<R> for [T; $size]
            where
                T: Subscriber<R>,
            {
                type Error = T::Error;

                fn notify(&self, state: &R) -> Result<(), Self::Error> {
                    AsRef::<[T]>::as_ref(self).notify(state)
                }
            }
        );
    };
}

impl_subscriber_for_array!(0, hide);
impl_subscriber_for_array!(1, hide);
impl_subscriber_for_array!(2, hide);
impl_subscriber_for_array!(3, hide);
impl_subscriber_for_array!(4, hide);
impl_subscriber_for_array!(5, hide);
impl_subscriber_for_array!(6, hide);
impl_subscriber_for_array!(7, hide);
impl_subscriber_for_array!(8, hide);
impl_subscriber_for_array!(9, hide);
impl_subscriber_for_array!(10, hide);
impl_subscriber_for_array!(11, hide);
impl_subscriber_for_array!(12, hide);
impl_subscriber_for_array!(13, hide);
impl_subscriber_for_array!(14, hide);
impl_subscriber_for_array!(15, hide);
impl_subscriber_for_array!(16, hide);
impl_subscriber_for_array!(17, hide);
impl_subscriber_for_array!(18, hide);
impl_subscriber_for_array!(19, hide);
impl_subscriber_for_array!(20, hide);
impl_subscriber_for_array!(21, hide);
impl_subscriber_for_array!(22, hide);
impl_subscriber_for_array!(23, hide);
impl_subscriber_for_array!(24, hide);
impl_subscriber_for_array!(25, hide);
impl_subscriber_for_array!(26, hide);
impl_subscriber_for_array!(27, hide);
impl_subscriber_for_array!(28, hide);
impl_subscriber_for_array!(29, hide);
impl_subscriber_for_array!(30, hide);
impl_subscriber_for_array!(31, hide);
impl_subscriber_for_array!(32, show);

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_subscriber_for_array {
        ($name:ident, $size:expr) => {
            #[test]
            fn $name() {
                let sbcs: [MockSubscriber<_>; $size] = Default::default();

                assert!(sbcs.notify(&5).is_ok());
                assert!(sbcs.notify(&1).is_ok());
                assert!(sbcs.notify(&3).is_ok());

                assert_eq!(
                    sbcs.to_vec(),
                    vec![MockSubscriber::new(vec![5, 1, 3]); $size]
                );
            }
        };
    }

    test_subscriber_for_array!(_00, 0);
    test_subscriber_for_array!(_01, 1);
    test_subscriber_for_array!(_02, 2);
    test_subscriber_for_array!(_03, 3);
    test_subscriber_for_array!(_04, 4);
    test_subscriber_for_array!(_05, 5);
    test_subscriber_for_array!(_06, 6);
    test_subscriber_for_array!(_07, 7);
    test_subscriber_for_array!(_08, 8);
    test_subscriber_for_array!(_09, 9);
    test_subscriber_for_array!(_10, 10);
    test_subscriber_for_array!(_11, 11);
    test_subscriber_for_array!(_12, 12);
    test_subscriber_for_array!(_13, 13);
    test_subscriber_for_array!(_14, 14);
    test_subscriber_for_array!(_15, 15);
    test_subscriber_for_array!(_16, 16);
    test_subscriber_for_array!(_17, 17);
    test_subscriber_for_array!(_18, 18);
    test_subscriber_for_array!(_19, 19);
    test_subscriber_for_array!(_20, 20);
    test_subscriber_for_array!(_21, 21);
    test_subscriber_for_array!(_22, 22);
    test_subscriber_for_array!(_23, 23);
    test_subscriber_for_array!(_24, 24);
    test_subscriber_for_array!(_25, 25);
    test_subscriber_for_array!(_26, 26);
    test_subscriber_for_array!(_27, 27);
    test_subscriber_for_array!(_28, 28);
    test_subscriber_for_array!(_29, 29);
    test_subscriber_for_array!(_30, 30);
    test_subscriber_for_array!(_31, 31);
    test_subscriber_for_array!(_32, 32);
}

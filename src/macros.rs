#![macro_use]

macro_rules! count {
    () => { 0 };

    ( $head:ident $(, $tail:ident )* $(,)* ) => { (1 + count!($($tail, )*)) };
}

macro_rules! dedupe_docs {
    ( (), $( $definition:tt )+ ) => {
        $( $definition )+
    };

    ( ($head:ident $(, $tail:ident )* $(,)*), $( $definition:tt )+ ) => {
        #[doc(hidden)]
        $( $definition )+
    };
}

#[cfg(feature = "parallel")]
macro_rules! join {
    ( ) => {
        ()
    };

    ( $a:ident $(,)* ) => {
        ($a(),)
    };

    ( $a:ident, $b:ident $(,)* ) => {
        rayon::join($a, $b)
    };

    ( $a:ident, $b:ident $(, $tail:ident )+ $(,)* ) => {
        {
            let ($a, ($b $(, $tail )+)) = rayon::join($a, || join!($b $(, $tail )+));
            ($a, $b $(, $tail )+)
        }
    };
}

#[cfg(test)]
mod test {
    #[cfg(feature = "parallel")]
    #[test]
    fn join() {
        let a = || 5;
        let b = || 1;
        let c = || 3;

        assert_eq!(join!(a), (5,));
        assert_eq!(join!(a, b), (5, 1));
        assert_eq!(join!(a, b, c), (5, 1, 3));
    }
}

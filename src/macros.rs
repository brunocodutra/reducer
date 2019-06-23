#![macro_use]

macro_rules! count {
    () => { 0 };

    ( $head:ident $(, $tail:ident )* $(,)? ) => { (1 + count!($($tail, )*)) };
}

macro_rules! dedupe_docs {
    ( (), $( $definition:tt )+ ) => {
        $( $definition )+
    };

    ( ($head:ident $(, $tail:ident )* $(,)?), $( $definition:tt )+ ) => {
        #[doc(hidden)]
        $( $definition )+
    };
}

#![macro_use]

#[cfg(test)]
macro_rules! count {
    ( $(,)? ) => { 0 };
    ( $a:ident $(,)? ) => { 1 };
    ( $a:ident, $b:ident $(,)? ) => { 2 };
    ( $( $a:ident, $b:ident $(,)? )+ ) => { 2 * count!($($a,)*) };
    ( $a:ident $(, $rest:ident)* $(,)? ) => { 1 + count!($($rest,)*) };
}

macro_rules! reverse {
    ( PRIVATE $macro:ident () ($($args:tt)*) ) => {
        $macro!($($args)*);
    };

    ( PRIVATE $macro:ident ($head:tt $(, $tail:tt)* $(,)?) ($($args:tt)*) ) => {
        reverse!(PRIVATE $macro ($($tail,)*) ($head, $($args)*));
    };

    ( $macro:ident!($($args:tt)*) ) => {
        reverse!(PRIVATE $macro ($($args)*) ());
    };
}

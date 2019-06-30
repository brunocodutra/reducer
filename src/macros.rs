#![macro_use]

macro_rules! count {
    ( $(,)? ) => { 0 };
    ( $a:ident $(,)? ) => { 1 };
    ( $a:ident, $b:ident $(,)? ) => { 2 };
    ( $a:ident, $b:ident, $c:ident $(,)? ) => { 3 };
    ( $a:ident, $b:ident, $c:ident, $d:ident $(,)? ) => { 4 };
    ( $a:ident, $b:ident, $c:ident, $d:ident, $e:ident $(, $rest:ident)* $(,)? ) => {
        (5 + count!($($rest,)*))
    };
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

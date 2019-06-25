#![macro_use]

macro_rules! count {
    () => { 0 };

    ( $head:ident $(, $tail:ident)* $(,)? ) => {
        (1 + count!($($tail,)*))
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

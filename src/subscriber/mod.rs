mod array;
mod mock;
mod reference;
mod slice;
mod tuple;

use std::fmt::Debug;

pub trait Subscriber<S> {
    type Error: Debug;
    fn notify(&self, state: &S) -> Result<(), Self::Error>;
}

#[cfg(test)]
pub use self::mock::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notify() {
        let mock = MockSubscriber::default();

        {
            let sbc: &Subscriber<_, Error = _> = &mock;

            assert!(sbc.notify(&5).is_ok());
            assert!(sbc.notify(&1).is_ok());
            assert!(sbc.notify(&3).is_ok());
        }

        assert_eq!(mock, MockSubscriber::new(vec![5, 1, 3]));
    }
}

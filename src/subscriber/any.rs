use smallbox::SmallBox;
use std::marker::PhantomData;
use subscriber::*;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
struct Dispatcher<'e, T>(T, PhantomData<&'e ()>);

impl<'e, R, T, E> Subscriber<R> for Dispatcher<'e, T>
where
    T: Subscriber<R, Error = E>,
    E: Debug + 'e,
{
    type Error = Box<dyn Debug + 'e>;

    fn notify(&self, state: &R) -> Result<(), Self::Error> {
        match self.0.notify(state) {
            // TODO: match box?
            Err(e) => Err(Box::new(e)),
            Ok(()) => Ok(()),
        }
    }
}

/// A container that can be constructed from any type that implements the trait
/// [Subscriber](trait.Subscriber.html).
///
/// AnySubscriber helps modeling situations where different reactors need to be subscribed to the
/// [Store](struct.Store.html) at different times during the execution of your application.
/// To improve cache locality, _sufficiently small_ objects (currently 32 bytes or less)
/// are stored inline rather than resorting to heap allocations.
pub struct AnySubscriber<'a, 'e: 'a, R>(
    SmallBox<dyn Subscriber<R, Error = Box<dyn Debug + 'e>> + 'a, [u8; 32]>,
);

impl<'a, 'e: 'a, R> AnySubscriber<'a, 'e, R> {
    pub fn new(subscriber: impl Subscriber<R, Error = impl Debug + 'e> + 'a) -> Self {
        AnySubscriber(smallbox!(Dispatcher(subscriber, PhantomData)))
    }
}

impl<'a, 'e: 'a, R> Subscriber<R> for AnySubscriber<'a, 'e, R> {
    type Error = Box<dyn Debug + 'e>;

    fn notify(&self, state: &R) -> Result<(), Self::Error> {
        (*self.0).notify(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn any() {
        let mock = &MockSubscriber::default();

        {
            let sbc = AnySubscriber::new(&mock);

            assert!(sbc.notify(&5).is_ok());
            assert!(sbc.notify(&1).is_ok());
            assert!(sbc.notify(&3).is_ok());
        }

        assert_eq!(mock, &MockSubscriber::new(vec![5, 1, 3]));
    }
}

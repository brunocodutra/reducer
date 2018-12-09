mod mock;
mod rc;
mod tuple;

pub trait Reducer: 'static {
    type Action;
    fn reduce(&mut self, action: Self::Action);
}

#[cfg(test)]
pub use self::mock::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduce() {
        let mut mock = MockReducer::default();

        {
            let state: &mut Reducer<Action = _> = &mut mock;

            state.reduce(5);
            state.reduce(1);
            state.reduce(3);
        }

        assert_eq!(mock, MockReducer::new(vec![5, 1, 3]));
    }
}

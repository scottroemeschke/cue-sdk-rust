use std::default::Default;

#[derive(Debug)]
pub(crate) struct CuePropertyValueHolder<T> {
    value: T,
}

impl<T> CuePropertyValueHolder<T>
where
    T: Default,
{
    pub fn new() -> Self {
        Self {
            value: Default::default(),
        }
    }
}

impl<T> CuePropertyValueHolder<T> {
    pub fn new_with_initial_value(value: T) -> Self {
        Self { value }
    }

    pub fn mut_ptr(&mut self) -> *mut T {
        &mut self.value as *mut T
    }

    pub fn value(self) -> T {
        self.value
    }
}

#[cfg(test)]
mod test {
    use super::CuePropertyValueHolder;

    #[derive(Debug, PartialEq)]
    struct FakeThing(i32);

    impl Default for FakeThing {
        fn default() -> Self {
            FakeThing(42)
        }
    }

    #[test]
    fn test_new_default() {
        let fake_thing = CuePropertyValueHolder::<FakeThing>::new().value();
        assert_eq!(fake_thing, FakeThing(42));
    }

    #[test]
    fn test_single_update_and_consume() {
        let mut vh = CuePropertyValueHolder::new_with_initial_value(50);
        let ptr = vh.mut_ptr();
        unsafe { std::mem::swap(&mut (*ptr), &mut 95) };
        let value_out = vh.value();
        assert_eq!(value_out, 95);
    }

    #[test]
    fn test_multiple_updates_and_consume() {
        let mut vh = CuePropertyValueHolder::new_with_initial_value(50);
        let ptr = vh.mut_ptr();
        unsafe { std::mem::swap(&mut (*ptr), &mut 95) };
        unsafe { std::mem::swap(&mut (*ptr), &mut 22) };
        let value_out = vh.value();
        assert_eq!(value_out, 22);
    }

    #[test]
    fn test_gets_proper_initial_value() {
        let mut vh = CuePropertyValueHolder::new_with_initial_value(FakeThing(999));
        let _ = vh.mut_ptr();
        //don't use the mut ptr
        let value_out = vh.value();
        assert_eq!(value_out, FakeThing(999));
    }
}

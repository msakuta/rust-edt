use super::BoolLike;

macro_rules! impl_int {
    ($target:ty) => {
        impl BoolLike for $target {
            fn as_bool(&self) -> bool {
                *self != 0
            }
        }
    };
}

impl BoolLike for bool {
    fn as_bool(&self) -> bool {
        *self
    }
}

impl_int!(u8);
impl_int!(i8);
impl_int!(u16);
impl_int!(i16);
impl_int!(i32);
impl_int!(u32);
impl_int!(i64);
impl_int!(u64);
impl_int!(i128);
impl_int!(u128);

macro_rules! impl_float {
    ($target:ty) => {
        impl BoolLike for $target {
            fn as_bool(&self) -> bool {
                *self != 0.
            }
        }
    };
}

impl_float!(f32);
impl_float!(f64);

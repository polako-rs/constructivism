use std::marker::PhantomData;

pub trait Constructor {
    type Fields: 'static;
    type UndefinedParams;
    type DefinedParams;
    fn construct_fields() -> &'static Self::Fields;
    fn construct_params() -> Self::UndefinedParams;
    fn construct(params: Self::DefinedParams)-> Self;
}

pub struct Field<T>(PhantomData<T>);
impl<T> Field<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

trait FetchField<F, T> {
    fn field(&self, f: &Field<T>) -> F;
}

trait ExtractValue {
    type Value;
    fn extract_value(self) -> Self::Value;
}

trait ExtractValues {
    type Values;
    fn extract_values(self) -> Self::Values;
}

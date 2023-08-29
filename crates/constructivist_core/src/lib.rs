use constructivist_macro_support::*;

pub trait Constructor {
    type Fields: 'static;
    type UndefinedParams;
    type DefinedParams;
    fn construct_fields() -> &'static Self::Fields;
    fn construct_params() -> Self::UndefinedParams;
    fn construct(params: Self::DefinedParams)-> Self;
}

pub struct Params<T>(T);

pub struct Field<T>(PhantomData<T>);
impl<T> Field<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

pub trait IntoFieldValue<T> {
    type Output;
    fn into_field_value(self, value: T) -> Self::Output;
}

trait ExtractField<F, T> {
    fn extract_field(&self, f: &Field<T>) -> F;
}
trait ExtractValue {
    type Value;
    fn extract_value(self) -> Self::Value;
}

trait ExtractValues {
    type Values;
    fn extract_values(self) -> Self::Values;
}

construct_implementations! { 4 }

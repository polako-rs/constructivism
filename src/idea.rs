use std::marker::PhantomData;

#[test]
fn test_main() {
    // let user = construct!(...);
    // output of
    // let user = construct!( User {
    //  name: "hello",
    //  age: 23,
    // });
    // user 
    let user = {
        let params = User::construct_params();
        let name = User::construct_fields().name();
        let name = params.field(&name).define(name.value("hello"));
        let age = User::construct_fields().age();
        let age = params.field(&age).define(age.value(23));
        let params = params + name;
        let params = params + age;
        let user = User::construct(params.extract_values());
        user
    };
    println!("{user:?}, {}, {}", user.name, user.age);
}

// #[derive(Builder)]
#[derive(Debug)]
struct User {
    name: String,
    age: usize,
}

// output of derive:

impl Constructor for User {
    type Fields = user_fields::Fields;
    type UndefinedParams = Params<(U0<user_fields::name>, U1<user_fields::age>)>;
    type DefinedParams = (user_fields::name, user_fields::age);
    fn construct_fields() -> &'static Self::Fields {
        &user_fields::Fields
    }
    fn construct_params() -> Self::UndefinedParams {
        Params((U0(PhantomData), U1(PhantomData)))
    }

    fn construct(params: Self::DefinedParams)-> Self {
        let (name, age) = params;
        Self {
            name: name.0,
            age: age.0
        }
    }
}

mod user_fields {
    use super::*;
    #[allow(non_camel_case_types)]
    pub struct name(pub String);


    #[allow(non_camel_case_types)]
    pub struct age(pub usize);

    impl<T: Into<String>> FieldValue<T> for Field<name> {
        type Output = name;
        fn value(self, value: T) -> Self::Output {
            name(value.into())
        }    
    }

    impl FieldValue<usize> for Field<age> {
        type Output = age;
        fn value(self, value: usize) -> Self::Output {
            age(value)
        }    
    }

    pub struct Fields;
    impl Fields {
        pub fn name(&self) -> Field<name> {
            Field::new()
        }
        pub fn age(&self) -> Field<age> {
            Field::new()
        }
    }
}

// library expanded content for two types (I want 64 types at least)

trait FetchField<F, T> {
    fn field(&self, f: &Field<T>) -> F;
}

pub trait Constructor {
    type Fields: 'static;
    type UndefinedParams;
    type DefinedParams;
    fn construct_fields() -> &'static Self::Fields;
    fn construct_params() -> Self::UndefinedParams;
    fn construct(params: Self::DefinedParams)-> Self;
}

// wrapped defined values
struct D0<T>(T);
struct D1<T>(T);

// undefined values with type associated
struct U0<T>(PhantomData<T>);
struct U1<T>(PhantomData<T>);

// fild markers
struct F0<T>(PhantomData<T>);
struct F1<T>(PhantomData<T>);

// params holder
pub struct Params<T>(T);

pub trait FieldValue<T> {
    type Output;
    fn value(self, value: T) -> Self::Output;
}

impl<T> F0<T> {
    fn define(self, value: T) -> D0<T> {
        D0(value)
    }
}
impl<T> F1<T> {
    fn define(self, value: T) -> D1<T> {
        D1(value)
    }
}


pub struct Field<T>(PhantomData<T>);
impl<T> Field<T> {
    fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T0, A1> FetchField<F0<T0>, T0> for Params<(U0<T0>, A1)> {
    fn field(&self, _: &Field<T0>) -> F0<T0> {
        F0(PhantomData)
    }
}
impl<A0, T1> FetchField<F1<T1>, T1> for Params<(A0, U1<T1>)> {
    fn field(&self, _: &Field<T1>) -> F1<T1> {
        F1(PhantomData)
    }
}

impl<T0, A1> FetchField<F0<T0>, T0> for Params<(D0<T0>, A1)> {
    fn field(&self, _: &Field<T0>) -> F0<T0> {
        F0(PhantomData)
    }
}
impl<A0, T1> FetchField<F1<T1>, T1> for Params<(A0, D1<T1>)> {
    fn field(&self, _: &Field<T1>) -> F1<T1> {
        F1(PhantomData)
    }
}




impl<T0, A1> std::ops::Add<D0<T0>> for Params<(U0<T0>, A1)> {
    type Output = Params<(D0<T0>, A1)>;
    fn add(self, rhs: D0<T0>) -> Self::Output {
        let (_, p1) = self.0;
        Params((rhs, p1))
    }
}
impl<A0, T1> std::ops::Add<D1<T1>> for Params<(A0, U1<T1>)> {
    type Output = Params<(A0, D1<T1>)>;
    fn add(self, rhs: D1<T1>) -> Self::Output {
        let (p0, _) = self.0;
        Params((p0, rhs))
    }
}





trait ExtractValue {
    type Value;
    fn extract_value(self) -> Self::Value;
}

impl<T: Default> ExtractValue for U0<T> {
    type Value = T;
    fn extract_value(self) -> T {
        T::default()
    }
}
impl<T> ExtractValue for D0<T> {
    type Value = T;
    fn extract_value(self) -> T {
        self.0
    }
}

impl<T: Default> ExtractValue for U1<T> {
    type Value = T;
    fn extract_value(self) -> T {
        T::default()
    }
}
impl<T> ExtractValue for D1<T> {
    type Value = T;
    fn extract_value(self) -> T {
        self.0
    }
}


trait ExtractValues {
    type Values;
    fn extract_values(self) -> Self::Values;
}

impl<T0: ExtractValue, T1: ExtractValue> ExtractValues for Params<(T0, T1)> {
    type Values = (T0::Value, T1::Value);
    fn extract_values(self) -> Self::Values {
        let (p0, p1) = self.0;
        (
            p0.extract_value(),
            p1.extract_value(),
        )
    }
}
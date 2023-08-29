use std::marker::PhantomData;

macro_rules! construct {
    ($t:ty { $($f:ident: $e:expr,)+ }) => {
        {
            let fields = <$t>::construct_fields();
            let params = <$t>::construct_params();
            $(
                let param = fields.$f();
                let param = params
                    .extract_field(&param.field())
                    .define(param.field().into_field_value($e));
                let params = params + param;
            )+
            // let (params, _rest) = <$t>::split_params(params);
            <$t>::construct(params.extract_values())
        }
        
    };
}

#[test]
fn test_main() {
    let user = construct!(User {
        age: 23.,
    });
    assert_eq!(user, User { name: "".into(), age: 23.});
}

// #[derive(Construct)]
#[derive(Debug, PartialEq)]
struct User {
    name: String,
    age: f32,
}

// output of derive:

impl Construct for User {
    type Fields = user_construct::Fields;
    type UndefinedParams = (user_construct::name, user_construct::age);
    type DefinedParams = (user_construct::name, user_construct::age);
    fn construct_fields() -> &'static Self::Fields {
        &user_construct::Fields
    }
    fn construct_params() -> Params<<<Self as Construct>::UndefinedParams as IntoParams>::Target> {
        <Self::UndefinedParams>::into_params()
    }
    
    fn construct(params: Self::DefinedParams)-> Self {
        let (user_construct::name(name), user_construct::age(age)) = params;
        Self { name, age }
    }
}

// impl<A: Construct<2>, B: Construct<2>> Construct<4> for (A, B) {
//     type Fields = (A::Fields, B::Fields);
//     type UndefinedParams = ;
// }

mod user_construct {
    use super::*;
    #[allow(non_camel_case_types)]
    #[derive(Default)]
    pub struct name(pub String);


    #[allow(non_camel_case_types)]
    #[derive(Default)]
    pub struct age(pub f32);

    impl IntoField for name {
        fn into_field() -> Field<Self> {
            Field(PhantomData)
        }
    }

    impl IntoField for age {
        fn into_field() -> Field<Self> {
            Field(PhantomData)
        }
    }

    impl<T: Into<String>> IntoFieldValue<T> for Field<name> {
        type Output = name;
        fn into_field_value(self, value: T) -> Self::Output {
            name(value.into())
        }    
    }

    impl<T: Into<f32>> IntoFieldValue<T> for Field<age> {
        type Output = age;
        fn into_field_value(self, value: T) -> Self::Output {
            age(value.into())
        }    
    }

    pub struct Fields;
    impl Fields {
        #[allow(unused)]
        pub fn name(&self) -> Param<name, String> {
            Param::new()
        }
        #[allow(unused)]
        pub fn age(&self) -> Param<age, f32> {
            Param::new()
        }
    }
}

// library expanded content for two types (I want 64 types at least)

// hand written:

pub trait Construct {
    type Fields: 'static;
    type UndefinedParams: IntoParams;
    type DefinedParams;
    fn construct_fields() -> &'static Self::Fields;
    fn construct_params() -> Params<<<Self as Construct>::UndefinedParams as IntoParams>::Target>;
    fn construct(params: Self::DefinedParams)-> Self;
    // fn split_params<T: SplitAt<{I}>>(params: T) -> (T::Left, T::Right){
    //     T::split(params)
    // }
}

impl Construct for () {
    type Fields = ();
    type DefinedParams = ();
    type UndefinedParams = ();
    fn construct_fields() -> &'static Self::Fields {
        &()
    }
    fn construct(_: Self::DefinedParams)-> Self {
        ()
    }
    fn construct_params() -> Params<<<Self as Construct>::UndefinedParams as IntoParams>::Target> {
        Params(())
    }
}

// pub fn split<const I: usize, T:SplitAt<I>

pub struct Params<T>(T);
pub struct Param<T, V>(PhantomData<(T, V)>);
impl<T, V> Param<T, V> {
    fn new() -> Self {
        Self(PhantomData)
    }
    fn field(&self) -> Field<T> {
        Field(PhantomData)
    }
} 
pub struct Field<T>(PhantomData<T>);

pub struct D<const I: u8, T>(T);
pub struct U<const I: u8, T>(PhantomData<T>);
struct F<const I: u8, T>(PhantomData<T>);

trait A<const I: u8, T> { }

trait ExtractField<F, T> {
    fn extract_field(&self, f: &Field<T>) -> F;
}

pub trait IntoField where Self: Sized {
    fn into_field() -> Field<Self>;
}

pub trait IntoFieldValue<T> {
    type Output;
    fn into_field_value(self, value: T) -> Self::Output;
}

pub trait MoveTo<const I: u8> {
    type Target;
    fn move_to(self) -> Self::Target;
}

trait ExtractValues {
    type Values;
    fn extract_values(self) -> Self::Values;
}


trait ExtractValue {
    type Value;
    fn extract_value(self) -> Self::Value;
}

impl<const I: u8, T> F<I, T> {
    fn define(self, value: T) -> D<I, T> {
        D::<I, T>(value)
    }
}


impl<const I: u8, T> A<I, T> for D<I, T> { }
impl<const I: u8, T> A<I, T> for U<I, T> { }

impl<const I: u8, const J: u8, T> MoveTo<J> for D<I, T> {
    type Target = D<J, T>;
    fn move_to(self) -> Self::Target {
        D::<J, T>(self.0)
    }
}
impl<const I: u8, const J: u8, T> MoveTo<J> for U<I, T> {
    type Target = U<J, T>;
    fn move_to(self) -> Self::Target {
        U::<J, T>(PhantomData)
    }
}
impl<const I: u8, const J: u8, T> MoveTo<J> for F<I, T> {
    type Target = F<J, T>;
    fn move_to(self) -> Self::Target {
        F::<J, T>(PhantomData)
    }
}

impl<const I: u8, T: Default> ExtractValue for U<I, T> {
    type Value = T;
    fn extract_value(self) -> T {
        T::default()
    }
}
impl<const I: u8, T> ExtractValue for D<I, T> {
    type Value = T;
    fn extract_value(self) -> T {
        self.0
    }
}

pub trait SplitAt<const I: u8> {
    type Left;
    type Right;
    fn split(self) -> (Self::Left, Self::Right);
}


impl<T> SplitAt<0> for Params<T> {
    type Left = ();
    type Right = Self;
    fn split(self) -> (Self::Left, Self::Right) {
        ((), self)
    }
}


// generated by macro

impl<T0, A0: A<0, T0>> ExtractField<F<0, T0>, T0> for Params<(A0,)> {
    fn extract_field(&self, _: &Field<T0>) -> F<0, T0> {
        F::<0, T0>(PhantomData)
    }
}
impl<T0, A0: A<0, T0>, A1> ExtractField<F<0, T0>, T0> for Params<(A0, A1)> {
    fn extract_field(&self, _: &Field<T0>) -> F<0, T0> {
        F::<0, T0>(PhantomData)
    }
}
impl<A0, A1: A<1, T1>, T1> ExtractField<F<1, T1>, T1> for Params<(A0, A1)> {
    fn extract_field(&self, _: &Field<T1>) -> F<1, T1> {
        F::<1, T1>(PhantomData)
    }
}
impl<T0, A0: A<0, T0>, A1, A2> ExtractField<F<0, T0>, T0> for Params<(A0, A1, A2)> {
    fn extract_field(&self, _: &Field<T0>) -> F<0, T0> {
        F::<0, T0>(PhantomData)
    }
}
impl<T1, A0, A1: A<1, T1>, A2> ExtractField<F<1, T1>, T1> for Params<(A0, A1, A2)> {
    fn extract_field(&self, _: &Field<T1>) -> F<1, T1> {
        F::<1, T1>(PhantomData)
    }
}
impl<T2, A0, A1, A2: A<2, T2>> ExtractField<F<2, T2>, T2> for Params<(A0, A1, A2)> {
    fn extract_field(&self, _: &Field<T2>) -> F<2, T2> {
        F::<2, T2>(PhantomData)
    }
}
impl<T3, A0, A1, A2, A3: A<3, T3>> ExtractField<F<3, T3>, T3> for Params<(A0, A1, A2, A3)> {
    fn extract_field(&self, _: &Field<T3>) -> F<3, T3> {
        F::<3, T3>(PhantomData)
    }
}


impl<T0> std::ops::Add<D<0, T0>> for Params<(U<0, T0>,)> {
    type Output = Params<(D<0, T0>,)>;
    fn add(self, rhs: D<0, T0>) -> Self::Output {
        Params((rhs,))
    }
}
impl<T0, A1> std::ops::Add<D<0, T0>> for Params<(U<0, T0>, A1)> {
    type Output = Params<(D<0, T0>, A1)>;
    fn add(self, rhs: D<0, T0>) -> Self::Output {
        let (_, p1) = self.0;
        Params((rhs, p1))
    }
}
impl<A0, T1> std::ops::Add<D<1, T1>> for Params<(A0, U<1, T1>)> {
    type Output = Params<(A0, D<1, T1>)>;
    fn add(self, rhs: D<1, T1>) -> Self::Output {
        let (p0, _) = self.0;
        Params((p0, rhs))
    }
}
impl<T0, A1, A2> std::ops::Add<D<0, T0>> for Params<(U<0, T0>, A1, A2)> {
    type Output = Params<(D<0, T0>, A1, A2)>;
    fn add(self, rhs: D<0, T0>) -> Self::Output {
        let (_, p1, p2) = self.0;
        Params((rhs, p1, p2))
    }
}
impl<T1, A0, A2> std::ops::Add<D<1, T1>> for Params<(A0, U<1, T1>, A2)> {
    type Output = Params<(A0, D<1, T1>, A2)>;
    fn add(self, rhs: D<1, T1>) -> Self::Output {
        let (p0, _, p2) = self.0;
        Params((p0, rhs, p2))
    }
}
impl<T2, A0, A1> std::ops::Add<D<2, T2>> for Params<(A0, A1, U<2, T2>)> {
    type Output = Params<(A0, A1, D<2, T2>)>;
    fn add(self, rhs: D<2, T2>) -> Self::Output {
        let (p0, p1, _) = self.0;
        Params((p0, p1, rhs))
    }
}
impl<T0, A1, A2, A3> std::ops::Add<D<0, T0>> for Params<(U<0, T0>, A1, A2, A3)> {
    type Output = Params<(D<0, T0>, A1, A2, A3)>;
    fn add(self, rhs: D<0, T0>) -> Self::Output {
        let (_, p1, p2, p3) = self.0;
        Params((rhs, p1, p2, p3))
    }
}
impl<T1, A0, A2, A3> std::ops::Add<D<1, T1>> for Params<(A0, U<1, T1>, A2, A3)> {
    type Output = Params<(A0, D<1, T1>, A2, A3)>;
    fn add(self, rhs: D<1, T1>) -> Self::Output {
        let (p0, _, p2, p3) = self.0;
        Params((p0, rhs, p2, p3))
    }
}
impl<T2, A0, A1, A3> std::ops::Add<D<2, T2>> for Params<(A0, A1, U<2, T2>, A3)> {
    type Output = Params<(A0, A1, D<2, T2>, A3)>;
    fn add(self, rhs: D<2, T2>) -> Self::Output {
        let (p0, p1, _, p3) = self.0;
        Params((p0, p1, rhs, p3))
    }
}
impl<T3, A0, A1, A2> std::ops::Add<D<3, T3>> for Params<(A0, A1, A2, U<3, T3>)> {
    type Output = Params<(A0, A1, A2, D<3, T3>)>;
    fn add(self, rhs: D<3, T3>) -> Self::Output {
        let (p0, p1, p2, _) = self.0;
        Params((p0, p1, p2, rhs))
    }
}
impl<T0: ExtractValue> ExtractValues for Params<(T0,)> {
    type Values = (T0::Value,);
    fn extract_values(self) -> Self::Values {
        let (p0,) = self.0;
        (
            p0.extract_value(),
        )
    }
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
impl<T0: ExtractValue, T1: ExtractValue, T2: ExtractValue> ExtractValues for Params<(T0, T1, T2)> {
    type Values = (T0::Value, T1::Value, T2::Value);
    fn extract_values(self) -> Self::Values {
        let (p0, p1, p2) = self.0;
        (
            p0.extract_value(),
            p1.extract_value(),
            p2.extract_value(),
        )
    }
}
impl<T0: ExtractValue, T1: ExtractValue, T2: ExtractValue, T3: ExtractValue> ExtractValues for Params<(T0, T1, T2, T3)> {
    type Values = (T0::Value, T1::Value, T2::Value, T3::Value);
    fn extract_values(self) -> Self::Values {
        let (p0, p1, p2, p3) = self.0;
        (
            p0.extract_value(),
            p1.extract_value(),
            p2.extract_value(),
            p3.extract_value(),
        )
    }
}


impl<T0, T1: MoveTo<0>> SplitAt<1> for Params<(T0, T1)> {
    type Left = Params<(T0,)>;
    type Right = Params<(T1::Target,)>;
    fn split(self) -> (Self::Left, Self::Right) {
        let (p0, p1) = self.0;
        (Params((p0,)), Params((p1.move_to(),)))
    }
}

impl<T0, T1: MoveTo<0>, T2: MoveTo<1>> SplitAt<1> for Params<(T0, T1, T2)> {
    type Left = Params<(T0,)>;
    type Right = Params<(T1::Target, T2::Target)>;
    fn split(self) -> (Self::Left, Self::Right) {
        let (p0, p1, p2) = self.0;
        (Params((p0,)), Params((p1.move_to(),p2.move_to())))
    }
}
impl<T0, T1> SplitAt<2> for Params<(T0, T1)> {
    type Left = Params<(T0, T1)>;
    type Right = Params<()>;
    fn split(self) -> (Self::Left, Self::Right) {
        let (p0, p1) = self.0;
        (Params((p0, p1,)), Params(()))
    }
}
impl<T0, T1, T2: MoveTo<0>> SplitAt<2> for Params<(T0, T1, T2)> {
    type Left = Params<(T0, T1)>;
    type Right = Params<(T2::Target,)>;
    fn split(self) -> (Self::Left, Self::Right) {
        let (p0, p1, p2) = self.0;
        (Params((p0,p1)), Params((p2.move_to(),)))
    }
}

impl<T0, T1, T2> SplitAt<3> for Params<(T0, T1, T2)> {
    type Left = Params<(T0, T1, T2)>;
    type Right = Params<()>;
    fn split(self) -> (Self::Left, Self::Right) {
        let (p0, p1, p2) = self.0;
        (Params((p0,p1,p2)), Params(()))
    }
}

pub trait IntoParams {
    type Target;
    fn into_params() -> Params<Self::Target>;
}
impl IntoParams for () {
    type Target = ();
    fn into_params() -> Params<Self::Target> {
        Params(())
    }
}
impl<T0: IntoField> IntoParams for (T0,) {
    type Target = (U<0, T0>,);
    fn into_params() -> Params<Self::Target> {
        Params((U::<0, _>(PhantomData),))
    }
}
impl<T0: IntoField> IntoParams for (T0,()) {
    type Target = (U<0, T0>,);
    fn into_params() -> Params<Self::Target> {
        Params((U::<0, _>(PhantomData),))
    }
}
impl<T0: IntoField, T1: IntoField> IntoParams for (T0, T1) {
    type Target = (U<0, T0>, U<1, T1>);
    fn into_params() -> Params<Self::Target> {
        Params((U::<0, _>(PhantomData), U::<1, _>(PhantomData)))
    }
}
impl<T0: IntoField, T1: IntoField> IntoParams for (T0, T1, ()) {
    type Target = (U<0, T0>, U<1, T1>);
    fn into_params() -> Params<Self::Target> {
        Params((U::<0, _>(PhantomData), U::<1, _>(PhantomData)))
    }
}
impl<T0: IntoField, T1: IntoField> IntoParams for (T0, (T1,)) {
    type Target = (U<0, T0>, U<1, T1>);
    fn into_params() -> Params<Self::Target> {
        Params((U::<0, _>(PhantomData), U::<1, _>(PhantomData)))
    }
}

impl<T0: IntoField, T1: IntoField, T2: IntoField> IntoParams for (T0, T1, T2) {
    type Target = (U<0, T0>, U<1, T1>, U<2, T2>);
    fn into_params() -> Params<Self::Target> {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
        ))
    }
}

impl<T0: IntoField, T1: IntoField, T2: IntoField> IntoParams for (T0, T1, (T2,)) {
    type Target = (U<0, T0>, U<1, T1>, U<2, T2>);
    fn into_params() -> Params<Self::Target> {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
        ))
    }
}

impl<T0: IntoField, T1: IntoField, T2: IntoField> IntoParams for (T0, (T1, T2)) {
    type Target = (U<0, T0>, U<1, T1>, U<2, T2>);
    fn into_params() -> Params<Self::Target> {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
        ))
    }
}

impl<T0: IntoField, T1: IntoField, T2: IntoField, T3: IntoField> IntoParams for (T0, T1, T2, T3) {
    type Target = (U<0, T0>, U<1, T1>, U<2, T2>, U<3, T3>);
    fn into_params() -> Params<Self::Target> {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
            U::<3, _>(PhantomData),
        ))
    }
}
impl<T0: IntoField, T1: IntoField, T2: IntoField, T3: IntoField> IntoParams for (T0, T1, T2, (T3,)) {
    type Target = (U<0, T0>, U<1, T1>, U<2, T2>, U<3, T3>);
    fn into_params() -> Params<Self::Target> {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
            U::<3, _>(PhantomData),
        ))
    }
}
impl<T0: IntoField, T1: IntoField, T2: IntoField, T3: IntoField> IntoParams for (T0, T1, (T2, T3,)) {
    type Target = (U<0, T0>, U<1, T1>, U<2, T2>, U<3, T3>);
    fn into_params() -> Params<Self::Target> {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
            U::<3, _>(PhantomData),
        ))
    }
}
impl<T0: IntoField, T1: IntoField, T2: IntoField, T3: IntoField> IntoParams for (T0, (T1, T2, T3,)) {
    type Target = (U<0, T0>, U<1, T1>, U<2, T2>, U<3, T3>);
    fn into_params() -> Params<Self::Target> {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
            U::<3, _>(PhantomData),
        ))
    }
}
impl<T0: IntoField, T1: IntoField, T2: IntoField, T3: IntoField> IntoParams for (T0, T1, (T2, (T3,))) {
    type Target = (U<0, T0>, U<1, T1>, U<2, T2>, U<3, T3>);
    fn into_params() -> Params<Self::Target> {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
            U::<3, _>(PhantomData),
        ))
    }
}
impl<T0: IntoField, T1: IntoField, T2: IntoField, T3: IntoField> IntoParams for (T0, (T1, T2, (T3,))) {
    type Target = (U<0, T0>, U<1, T1>, U<2, T2>, U<3, T3>);
    fn into_params() -> Params<Self::Target> {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
            U::<3, _>(PhantomData),
        ))
    }
}
impl<T0: IntoField, T1: IntoField, T2: IntoField, T3: IntoField> IntoParams for (T0, (T1, (T2, (T3,)))) {
    type Target = (U<0, T0>, U<1, T1>, U<2, T2>, U<3, T3>);
    fn into_params() -> Params<Self::Target> {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
            U::<3, _>(PhantomData),
        ))
    }
}



// impl<T0, T1> IntoParams<2> for (U<0, T0>, U<1, T1>) {
//     type Target = (U<0, T0>, U<1, T1>);
//     fn into_params(self) -> Params<2, (U<0, T0>, U<1, T1>)> {
//         Params::<2, _>((self.0, self.1))
//     }
// }

// impl<T0, T1, P0, P1, P:IntoParams<2, Target = (U<0, P0>, U<1, P1>)>> IntoParams<4> for (U<0, T0>, U<1, T1>, P) {
//     type Target = (U<0, T0>, U<1, T1>, U<2, P0>, U<3, P1>);
//     fn into_params(self) -> Params<4, Self::Target> {
//         let (t0, t1, p) = self;
//         let (p0, p1) = p.into_params().0;
//         Params::<4, _>((
//             t0,
//             t1,
//             <U<0, _> as MoveTo<2>>::move_to(p0),
//             <U<1, _> as MoveTo<3>>::move_to(p1)
//         ))
        
//     }
// }

struct min(f32);
struct max(f32);
struct label(String);
struct color(String);
trait Widget {
    // type Params: IntoParams;
}

struct RangeWidget;
impl Widget for RangeWidget {
    // type Params = (U<0, min>, U<1, max>);
}



// impl Widget<4, (U<0, min>, U<0, max>) for 
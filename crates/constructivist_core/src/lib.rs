use std::marker::PhantomData;

use constructivist_macro_support::*;

pub mod traits {
    pub use super::Construct;
    pub use super::A;
    pub use super::Singleton;
    pub use super::ExtractField;
    pub use super::AsField;
    pub use super::MoveTo;
    pub use super::DefinedValue;
    pub use super::DefinedValues;
    pub use super::Flattern;
    pub use super::NonUnit;
    pub use super::AsParams;
}

pub trait Construct {
    type Fields: Singleton;
    type Params: AsParams;
    type Wraps: Construct;
    type Wrapped;
    type WrappedParams: AsParams;
    fn construct_fields() -> &'static Self::Fields;
    // fn construct_params() -> Params<<<Self as Construct>::UndefinedParams as IntoParams>::Target>;
    fn construct(params: Self::Params)-> Self;
    // fn split_params<T: SplitAt<{I}>>(params: T) -> (T::Left, T::Right){
    //     T::split(params)
    // }
    fn construct_all<P>(params: P) -> <Self as Construct>::Wrapped
    where 
        Self: Sized,
        P: DefinedValues<Self::Params, Output = <<<Self as Construct>::Wraps as Construct>::WrappedParams as AsParams>::Defined >;
}

#[macro_export]
macro_rules! construct {
    ($t:ty { $($f:ident: $e:expr,)+ }) => {
        {
            use crate::traits::*;
            let fields = <$t as crate::Construct>::construct_fields();
            let params = <<$t as crate::Construct>::Params as crate::AsParams>::as_undefined();
            $(
                let param = fields.$f();
                let field = param.field();
                let param = params.field(&field).define($e.into());
                let params = params + param;
            )+
            let (params, _rest) = <crate::Params<_> as crate::DefinedValues<<$t as crate::Construct>::Params>>::extract_values(params);
            <$t as crate::Construct>::construct(params)
        }
        
    };
}
#[macro_export]
macro_rules! constructall {
    ($t:ty { $($f:ident: $e:expr,)+ }) => {
        {
            use crate::traits::*;
            let fields = <$t as crate::Construct>::construct_fields();
            let params = <<$t as crate::Construct>::WrappedParams as crate::AsParams>::as_undefined();
            $(
                let param = fields.$f();
                let field = param.field();
                let param = params.field(&field).define($e.into());
                let params = params + param;
            )+
            let defined_params = params.defined();
            <$t as crate::Construct>::construct_all(defined_params).flattern()
        }
        
    };
}

impl Construct for () {
    type Fields = ();
    type Params = ();
    type Wraps = ();
    type Wrapped = ();
    type WrappedParams = ();
    fn construct_fields() -> &'static Self::Fields {
        &()
    }
    fn construct(_: Self::Params)-> Self {
        ()
    }
    fn construct_all<P>(_: P) -> <Self as Construct>::Wrapped
    where Self: Sized, P: DefinedValues<
        Self::Params,
        Output = <<<Self as Construct>::Wraps as Construct>::WrappedParams as AsParams>::Defined
    > {
        ()
    }
}

pub struct Params<T>(T);
pub struct Param<T, V>(PhantomData<(T, V)>);
impl<T, V> Param<T, V> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
    pub fn field(&self) -> Field<T> {
        Field(PhantomData)
    }
} 
pub struct Field<T>(PhantomData<T>);
impl<T> Field<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
    pub fn value<V: Into<T>>(&self, value: V) -> T {
        value.into()
    }
}

pub struct D<const I: u8, T>(T);
pub struct U<const I: u8, T>(PhantomData<T>);
pub struct F<const I: u8, T>(PhantomData<T>);

pub trait A<const I: u8, T> { }

pub trait Singleton {
    fn instance() -> &'static Self;
}

impl Singleton for () {
    fn instance() -> &'static Self {
        &()
    }
}

pub trait ExtractField<F, T> {
    fn field(&self, f: &Field<T>) -> F;
}

pub trait AsField where Self: Sized {
    fn as_field() -> Field<Self>;
}

pub trait MoveTo<const I: u8> {
    type Target;
    fn move_to(self) -> Self::Target;
}

pub trait DefinedValues<T> {
    type Output;
    fn extract_values(self) -> (T, Self::Output);
}


pub trait DefinedValue {
    type Value;
    fn extract_value(self) -> Self::Value;
}

pub trait NonUnit { }

pub trait Flattern {
    type Output;
    fn flattern(self) -> Self::Output;
}

impl<T0: NonUnit> Flattern for (T0, ()) {
    type Output = T0;
    fn flattern(self) -> Self::Output {
        let (p0, _) = self;
        p0
    }
}

impl <T0: NonUnit, T1: NonUnit> Flattern for (T0, (T1, ())) {
    type Output = (T0, T1);
    fn flattern(self) -> Self::Output {
        let (p0, (p1, _)) = self;
        (p0, p1)
    }
}

impl <T0: NonUnit, T1: NonUnit, T2: NonUnit> Flattern for (T0, (T1, (T2, ()))) {
    type Output = (T0, T1, T2);
    fn flattern(self) -> Self::Output {
        let (p0, (p1, (p2, _))) = self;
        (p0, p1, p2)
    }
}

impl <T0: NonUnit, T1: NonUnit, T2: NonUnit, T3: NonUnit> Flattern for (T0, (T1, (T2, (T3, ())))) {
    type Output = (T0, T1, T2, T3);
    fn flattern(self) -> Self::Output {
        let (p0, (p1, (p2, (p3, _)))) = self;
        (p0, p1, p2, p3)
    }
}

impl<const I: u8, T> F<I, T> {
    pub fn define(self, value: T) -> D<I, T> {
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

impl<const I: u8, T: Default> DefinedValue for U<I, T> {
    type Value = T;
    fn extract_value(self) -> T {
        T::default()
    }
}
impl<const I: u8, T> DefinedValue for D<I, T> {
    type Value = T;
    fn extract_value(self) -> T {
        self.0
    }
}

// generated by macro

impl<T0, A0: A<0, T0>> ExtractField<F<0, T0>, T0> for Params<(A0,)> {
    fn field(&self, _: &Field<T0>) -> F<0, T0> {
        F::<0, T0>(PhantomData)
    }
}
impl<T0, A0: A<0, T0>, A1> ExtractField<F<0, T0>, T0> for Params<(A0, A1)> {
    fn field(&self, _: &Field<T0>) -> F<0, T0> {
        F::<0, T0>(PhantomData)
    }
}
impl<A0, A1: A<1, T1>, T1> ExtractField<F<1, T1>, T1> for Params<(A0, A1)> {
    fn field(&self, _: &Field<T1>) -> F<1, T1> {
        F::<1, T1>(PhantomData)
    }
}
impl<T0, A0: A<0, T0>, A1, A2> ExtractField<F<0, T0>, T0> for Params<(A0, A1, A2)> {
    fn field(&self, _: &Field<T0>) -> F<0, T0> {
        F::<0, T0>(PhantomData)
    }
}
impl<T1, A0, A1: A<1, T1>, A2> ExtractField<F<1, T1>, T1> for Params<(A0, A1, A2)> {
    fn field(&self, _: &Field<T1>) -> F<1, T1> {
        F::<1, T1>(PhantomData)
    }
}
impl<T2, A0, A1, A2: A<2, T2>> ExtractField<F<2, T2>, T2> for Params<(A0, A1, A2)> {
    fn field(&self, _: &Field<T2>) -> F<2, T2> {
        F::<2, T2>(PhantomData)
    }
}


impl<T0, A0: A<0, T0>, A1, A2, A3> ExtractField<F<0, T0>, T0> for Params<(A0, A1, A2, A3)> {
    fn field(&self, _: &Field<T0>) -> F<0, T0> {
        F::<0, T0>(PhantomData)
    }
}
impl<T1, A0, A1: A<1, T1>, A2, A3> ExtractField<F<1, T1>, T1> for Params<(A0, A1, A2, A3)> {
    fn field(&self, _: &Field<T1>) -> F<1, T1> {
        F::<1, T1>(PhantomData)
    }
}
impl<T2, A0, A1, A2: A<2, T2>, A3> ExtractField<F<2, T2>, T2> for Params<(A0, A1, A2, A3)> {
    fn field(&self, _: &Field<T2>) -> F<2, T2> {
        F::<2, T2>(PhantomData)
    }
}
impl<T3, A0, A1, A2, A3: A<3, T3>> ExtractField<F<3, T3>, T3> for Params<(A0, A1, A2, A3)> {
    fn field(&self, _: &Field<T3>) -> F<3, T3> {
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
impl<T> DefinedValues<()> for Params<T> {
    type Output = Self;
    fn extract_values(self) -> ((), Self::Output) {
        ((), self)
    }
}

impl<T0: DefinedValue<Value = P0>, P0> DefinedValues<(P0,)> for Params<(T0,)> {
    type Output = Params<()>;
    fn extract_values(self) -> ((P0,), Self::Output) {
        let (p0,) = self.0;
        ((
            p0.extract_value(),
        ), Params(()))
    }
}

impl DefinedValues<()> for () {
    type Output = Params<()>;
    fn extract_values(self) -> ((), Self::Output) {
        ((), Params(()))
    }
}

impl<T0, T1, P0> DefinedValues<(P0,)> for Params<(T0, T1)>
where
    T0: DefinedValue<Value = P0>,
    T1: MoveTo<0>,

{
    type Output = Params<(T1::Target,)>;
    fn extract_values(self) -> ((P0,), Self::Output) {
        let (p0, p1) = self.0;
        ((
            p0.extract_value(),
        ), Params((p1.move_to(),)))
    }
}
impl<T0, T1, T2, P0> DefinedValues<(P0,)> for Params<(T0, T1, T2)>
where
    T0: DefinedValue<Value = P0>,
    T1: MoveTo<0>,
    T2: MoveTo<1>,

{
    type Output = Params<(T1::Target, T2::Target)>;
    fn extract_values(self) -> ((P0,), Self::Output) {
        let (p0, p1, p2) = self.0;
        ((
            p0.extract_value(),
        ), Params((
            p1.move_to(),
            p2.move_to(),
        )))
    }
}
impl<T0, T1, T2, T3, P0> DefinedValues<(P0,)> for Params<(T0, T1, T2, T3)>
where
    T0: DefinedValue<Value = P0>,
    T1: MoveTo<0>,
    T2: MoveTo<1>,
    T3: MoveTo<2>,

{
    type Output = Params<(T1::Target, T2::Target, T3::Target)>;
    fn extract_values(self) -> ((P0,), Self::Output) {
        let (p0, p1, p2, p3) = self.0;
        ((
            p0.extract_value(),
        ), Params((
            p1.move_to(),
            p2.move_to(),
            p3.move_to(),
        )))
    }
}

impl<P0, P1, T0: DefinedValue<Value = P0>, T1: DefinedValue<Value = P1>> DefinedValues<(P0, P1)> for Params<(T0, T1)> {
    type Output = Params<()>;
    fn extract_values(self) -> ((P0, P1), Self::Output) {
        let (p0, p1) = self.0;
        ((
            p0.extract_value(),
            p1.extract_value(),
        ), Params(()))
    }
}
impl<P0, P1, T0, T1, T2> DefinedValues<(P0, P1)> for Params<(T0, T1, T2)>
where
    T0: DefinedValue<Value = P0>,
    T1: DefinedValue<Value = P1>,
    T2: MoveTo<0>,
{
    type Output = Params<(T2::Target,)>;
    fn extract_values(self) -> ((P0, P1), Self::Output) {
        let (p0, p1, p2) = self.0;
        ((
            p0.extract_value(),
            p1.extract_value(),
        ), Params((p2.move_to(),)))
    }
}
impl<P0, P1, T0, T1, T2, T3> DefinedValues<(P0, P1)> for Params<(T0, T1, T2, T3)>
where
    T0: DefinedValue<Value = P0>,
    T1: DefinedValue<Value = P1>,
    T2: MoveTo<0>,
    T3: MoveTo<1>,
{
    type Output = Params<(T2::Target, T3::Target)>;
    fn extract_values(self) -> ((P0, P1), Self::Output) {
        let (p0, p1, p2, p3) = self.0;
        ((
            p0.extract_value(),
            p1.extract_value(),
        ), Params((
            p2.move_to(),
            p3.move_to(),
        )))
    }
}
impl<P0, P1, P2, T0: DefinedValue<Value = P0>, T1: DefinedValue<Value = P1>, T2: DefinedValue<Value = P2>> DefinedValues<(P0, P1, P2)> for Params<(T0, T1, T2)> {
    type Output = Params<()>;
    fn extract_values(self) -> ((P0, P1, P2), Self::Output) {
        let (p0, p1, p2) = self.0;
        ((
            p0.extract_value(),
            p1.extract_value(),
            p2.extract_value(),
        ), Params(()))
    }
}
impl<P0, P1, P2, T0, T1, T2, T3> DefinedValues<(P0, P1, P2)> for Params<(T0, T1, T2, T3)>
where
    T0: DefinedValue<Value = P0>,
    T1: DefinedValue<Value = P1>,
    T2: DefinedValue<Value = P2>,
    T3: MoveTo<0>,
{
    type Output = Params<(T3::Target,)>;
    fn extract_values(self) -> ((P0, P1, P2), Self::Output) {
        let (p0, p1, p2, p3) = self.0;
        ((
            p0.extract_value(),
            p1.extract_value(),
            p2.extract_value(),
        ), Params((
            p3.move_to(),
        )))
    }
}
impl<P0, P1, P2, P3, T0, T1, T2, T3> DefinedValues<(P0, P1, P2, P3)> for Params<(T0, T1, T2, T3)>
where
    T0: DefinedValue<Value = P0>,
    T1: DefinedValue<Value = P1>,
    T2: DefinedValue<Value = P2>,
    T3: DefinedValue<Value = P3>,
{
    type Output = Params<()>;
    fn extract_values(self) -> ((P0, P1, P2, P3), Self::Output) {
        let (p0, p1, p2, p3) = self.0;
        ((
            p0.extract_value(),
            p1.extract_value(),
            p2.extract_value(),
            p3.extract_value(),
        ), Params(()))
    }
}

pub trait AsParams {
    type Undefined;
    type Defined;
    fn as_undefined() -> Self::Undefined;
}
impl AsParams for () {
    type Undefined = Params<()>;
    type Defined = Params<()>;
    fn as_undefined() -> Self::Undefined {
        Params(())
    }
}
impl<T0: AsField> AsParams for (T0,) {
    type Undefined = Params<(U<0, T0>,)>;
    type Defined = Params<(D<0, T0>,)>;
    fn as_undefined() -> Self::Undefined {
        Params((U::<0, _>(PhantomData),))
    }
}
impl<T0: AsField> AsParams for (T0,()) {
    type Undefined = Params<(U<0, T0>,)>;
    type Defined = Params<(D<0, T0>,)>;
    fn as_undefined() -> Self::Undefined {
        Params((U::<0, _>(PhantomData),))
    }
}

// impl<T0: AsField, T1:
impl<T0: AsField, T1: AsField> AsParams for (T0, T1) {
    type Undefined = Params<(U<0, T0>, U<1, T1>)>;
    type Defined = Params<(D<0, T0>, D<1, T1>)>;
    fn as_undefined() -> Self::Undefined {
        Params((U::<0, _>(PhantomData), U::<1, _>(PhantomData)))
    }
}
impl<T0: AsField, T1: AsField> AsParams for (T0, T1, ()) {
    type Undefined = Params<(U<0, T0>, U<1, T1>)>;
    type Defined = Params<(D<0, T0>, D<1, T1>)>;
    fn as_undefined() -> Self::Undefined {
        Params((U::<0, _>(PhantomData), U::<1, _>(PhantomData)))
    }
}
impl<T0: AsField, T1: AsField> AsParams for (T0, (T1, ())) {
    type Undefined = Params<(U<0, T0>, U<1, T1>)>;
    type Defined = Params<(D<0, T0>, D<1, T1>)>;
    fn as_undefined() -> Self::Undefined {
        Params((U::<0, _>(PhantomData), U::<1, _>(PhantomData)))
    }
}

impl<T0: AsField, T1: AsField, T2: AsField> AsParams for (T0, T1, T2) {
    type Undefined = Params<(U<0, T0>, U<1, T1>, U<2, T2>)>;
    type Defined = Params<(D<0, T0>, D<1, T1>, D<2, T2>)>;
    fn as_undefined() -> Self::Undefined {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
        ))
    }
}
impl<T0: AsField, T1: AsField, T2: AsField> AsParams for (T0, T1, T2, ()) {
    type Undefined = Params<(U<0, T0>, U<1, T1>, U<2, T2>)>;
    type Defined = Params<(D<0, T0>, D<1, T1>, D<2, T2>)>;
    fn as_undefined() -> Self::Undefined {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
        ))
    }
}

impl<T0: AsField, T1: AsField, T2: AsField> AsParams for (T0, T1, (T2, ())) {
    type Undefined = Params<(U<0, T0>, U<1, T1>, U<2, T2>)>;
    type Defined = Params<(D<0, T0>, D<1, T1>, D<2, T2>)>;
    fn as_undefined() -> Self::Undefined {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
        ))
    }
}

impl<T0: AsField, T1: AsField, T2: AsField> AsParams for (T0, (T1, T2, ())) {
    type Undefined = Params<(U<0, T0>, U<1, T1>, U<2, T2>)>;
    type Defined = Params<(D<0, T0>, D<1, T1>, D<2, T2>)>;
    fn as_undefined() -> Self::Undefined {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
        ))
    }
}

impl<T0: AsField, T1: AsField, T2: AsField, T3: AsField> AsParams for (T0, T1, T2, T3) {
    type Undefined = Params<(U<0, T0>, U<1, T1>, U<2, T2>, U<3, T3>)>;
    type Defined = Params<(D<0, T0>, D<1, T1>, D<2, T2>, D<3, T3>)>;
    fn as_undefined() -> Self::Undefined {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
            U::<3, _>(PhantomData),
        ))
    }
}
impl<T0: AsField, T1: AsField, T2: AsField, T3: AsField> AsParams for (T0, T1, T2, T3, ()) {
    type Undefined = Params<(U<0, T0>, U<1, T1>, U<2, T2>, U<3, T3>)>;
    type Defined = Params<(D<0, T0>, D<1, T1>, D<2, T2>, D<3, T3>)>;
    fn as_undefined() -> Self::Undefined {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
            U::<3, _>(PhantomData),
        ))
    }
}
impl<T0: AsField, T1: AsField, T2: AsField, T3: AsField> AsParams for (T0, T1, T2, (T3, ())) {
    type Undefined = Params<(U<0, T0>, U<1, T1>, U<2, T2>, U<3, T3>)>;
    type Defined = Params<(D<0, T0>, D<1, T1>, D<2, T2>, D<3, T3>)>;
    fn as_undefined() -> Self::Undefined {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
            U::<3, _>(PhantomData),
        ))
    }
}
impl<T0: AsField, T1: AsField, T2: AsField, T3: AsField> AsParams for (T0, T1, (T2, T3, ())) {
    type Undefined = Params<(U<0, T0>, U<1, T1>, U<2, T2>, U<3, T3>)>;
    type Defined = Params<(D<0, T0>, D<1, T1>, D<2, T2>, D<3, T3>)>;
    fn as_undefined() -> Self::Undefined {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
            U::<3, _>(PhantomData),
        ))
    }
}
impl<T0: AsField, T1: AsField, T2: AsField, T3: AsField> AsParams for (T0, (T1, T2, T3, ())) {
    type Undefined = Params<(U<0, T0>, U<1, T1>, U<2, T2>, U<3, T3>)>;
    type Defined = Params<(D<0, T0>, D<1, T1>, D<2, T2>, D<3, T3>)>;
    fn as_undefined() -> Self::Undefined {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
            U::<3, _>(PhantomData),
        ))
    }
}
impl<T0: AsField, T1: AsField, T2: AsField, T3: AsField> AsParams for (T0, T1, (T2, (T3, ()))) {
    type Undefined = Params<(U<0, T0>, U<1, T1>, U<2, T2>, U<3, T3>)>;
    type Defined = Params<(D<0, T0>, D<1, T1>, D<2, T2>, D<3, T3>)>;
    fn as_undefined() -> Self::Undefined {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
            U::<3, _>(PhantomData),
        ))
    }
}
impl<T0: AsField, T1: AsField, T2: AsField, T3: AsField> AsParams for (T0, (T1, T2, (T3, ()))) {
    type Undefined = Params<(U<0, T0>, U<1, T1>, U<2, T2>, U<3, T3>)>;
    type Defined = Params<(D<0, T0>, D<1, T1>, D<2, T2>, D<3, T3>)>;
    fn as_undefined() -> Self::Undefined {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
            U::<3, _>(PhantomData),
        ))
    }
}
impl<T0: AsField, T1: AsField, T2: AsField, T3: AsField> AsParams for (T0, (T1, (T2, (T3, ())))) {
    type Undefined = Params<(U<0, T0>, U<1, T1>, U<2, T2>, U<3, T3>)>;
    type Defined = Params<(D<0, T0>, D<1, T1>, D<2, T2>, D<3, T3>)>;
    fn as_undefined() -> Self::Undefined {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
            U::<3, _>(PhantomData),
        ))
    }
}

impl Params<()> {
    pub fn defined(self) -> Self {
        self
    }
}

impl<T0: DefinedValue> Params<(T0,)> {
    pub fn defined(self) -> Params<(D<0, T0::Value>,)> {
        let (p0,) = self.0;
        Params((
            D::<0, _>(p0.extract_value()),
        ))
    }
}
impl<T0: DefinedValue, T1: DefinedValue> Params<(T0, T1)> {
    pub fn defined(self) -> Params<(D<0, T0::Value>, D<1, T1::Value>)> {
        let (p0,p1) = self.0;
        Params((
            D::<0, _>(p0.extract_value()),
            D::<1, _>(p1.extract_value()),
        ))
    }
}
impl<T0: DefinedValue, T1: DefinedValue, T2: DefinedValue> Params<(T0, T1, T2)> {
    pub fn defined(self) -> Params<(D<0, T0::Value>, D<1, T1::Value>, D<2, T2::Value>)> {
        let (p0,p1,p2) = self.0;
        Params((
            D::<0, _>(p0.extract_value()),
            D::<1, _>(p1.extract_value()),
            D::<2, _>(p2.extract_value()),
        ))
    }
}
impl<T0: DefinedValue, T1: DefinedValue, T2: DefinedValue, T3: DefinedValue> Params<(T0, T1, T2, T3)> {
    pub fn defined(self) -> Params<(D<0, T0::Value>, D<1, T1::Value>, D<2, T2::Value>, D<3, T3::Value>)> {
        let (p0,p1,p2,p3) = self.0;
        Params((
            D::<0, _>(p0.extract_value()),
            D::<1, _>(p1.extract_value()),
            D::<2, _>(p2.extract_value()),
            D::<3, _>(p3.extract_value()),
        ))
    }
}
use std::marker::PhantomData;

pub mod traits {
    pub use super::AsField;
    pub use super::Construct;
    pub use super::ConstructItem;
    pub use super::ExtractField;
    pub use super::ExtractValue;
    pub use super::Flattern;
    pub use super::Mixed;
    pub use super::New;
    pub use super::Segment;
    pub use super::Singleton;
    pub use super::A;
}

pub trait ConstructItem {
    type Params: Extractable;
    fn construct_item(params: Self::Params) -> Self;
}

pub trait Construct: ConstructItem {
    type Base: Construct;
    type Sequence;

    type Fields: Singleton;
    type Design: Singleton;

    type MixedParams: Extractable;
    type ExpandedParams: Extractable;
    type NestedSequence: Flattern;

    fn construct<P, const I: u8>(params: P) -> Self::NestedSequence where P: ExtractParams<
        I, Self::MixedParams,
        Value = <Self::MixedParams as Extractable>::Output,
        Rest = <<<Self::Base as Construct>::ExpandedParams as Extractable>::Input as AsParams>::Defined
    >;
}

pub trait Segment: ConstructItem {
    type Fields<T: Singleton + 'static>: Singleton;
    type Design<T: Singleton + 'static>: Singleton;
}

#[macro_export]
macro_rules! design {
    ($t:ty) => {
        <<$t as $crate::Construct>::Design as $crate::Singleton>::instance()
    };
}

impl ConstructItem for () {
    type Params = ();

    fn construct_item(_: Self::Params) -> Self {
        ()
    }
}

impl Construct for () {
    type Base = ();
    type Fields = ();
    type Design = ();
    type NestedSequence = ();
    type MixedParams = ();
    type ExpandedParams = ();
    type Sequence = <Self::NestedSequence as Flattern>::Output;
    fn construct<P, const I: u8>(_: P) -> Self::NestedSequence where P: ExtractParams<
        I, Self::MixedParams,
        Value = <Self::MixedParams as Extractable>::Output,
        Rest = <<<Self::Base as Construct>::ExpandedParams as Extractable>::Input as AsParams>::Defined
    >{
        ()
    }
}

pub struct Params<T>(T);
impl<T> Params<T> {
    pub fn validate<P>(&self, _: P) -> fn() -> () {
        || {}
    }
}

pub trait Extends<T: Construct> {}
impl<
        E: Construct<NestedSequence = BaseSeq>,
        T: Construct<NestedSequence = Seq>,
        Seq: Contains<Exclusive, BaseSeq>,
        BaseSeq,
    > Extends<E> for T
{
}
pub trait Is<T: Construct> {}
impl<
        E: Construct<NestedSequence = BaseSeq>,
        T: Construct<NestedSequence = Seq>,
        Seq: Contains<Inclusive, BaseSeq>,
        BaseSeq,
    > Is<E> for T
{
}

pub struct Inclusive;
pub struct Exclusive;
pub trait Contains<I, T> {}

pub struct ParamConflict<N>(PhantomData<N>);
impl<N> ParamConflict<N> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
    pub fn validate<T>(&self, _: &Param<N, T>) -> ParamRedefined<N> {
        ParamRedefined(PhantomData)
    }
}

pub struct ParamRedefined<N>(PhantomData<N>);

pub struct Param<N, T>(pub PhantomData<(N, T)>);
impl<N, T> Param<N, T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
    pub fn field(&self) -> Field<N> {
        Field(PhantomData)
    }
}

pub trait New<T> {
    fn new(from: T) -> Self;
}
impl<N: New<T>, T> Param<N, T> {
    pub fn value(&self, value: T) -> N {
        N::new(value)
    }
}

pub struct Field<T>(PhantomData<T>);
impl<T> Field<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

pub struct D<const I: u8, T>(pub T);
pub struct U<const I: u8, T>(pub PhantomData<T>);
pub struct F<const I: u8, T>(PhantomData<T>);

pub trait A<const I: u8, T> {}

pub trait Singleton {
    fn instance() -> &'static Self;
}

impl Singleton for () {
    fn instance() -> &'static Self {
        &()
    }
}

pub trait Extractable {
    type Input: AsParams;
    type Output;
    fn extract(input: Self::Input) -> Self::Output;

    fn as_params() -> <Self::Input as AsParams>::Undefined {
        <Self::Input as AsParams>::as_params()
    }
}

impl Extractable for () {
    type Input = ();
    type Output = ();
    fn extract(_: Self::Input) -> Self::Output {
        ()
    }
}

pub trait Mixed<Right>
where
    Self: Sized,
{
    type Output;
    fn split(mixed: Self::Output) -> (Self, Right);
}

impl Mixed<()> for () {
    type Output = ();
    fn split(_: Self::Output) -> (Self, ()) {
        ((), ())
    }
}
pub struct Mix<L, R>(PhantomData<(L, R)>);

impl<O: AsParams, L: Extractable, R: Extractable> Extractable for Mix<L, R>
where
    L::Input: Mixed<R::Input, Output = O>,
{
    type Input = O;
    type Output = (L::Output, R::Output);
    fn extract(input: Self::Input) -> Self::Output {
        let (left, right) = <L::Input as Mixed<R::Input>>::split(input);
        (L::extract(left), R::extract(right))
    }
}

pub trait ExtractParams<const S: u8, T> {
    type Value;
    type Rest;
    fn extract_params(self) -> (Self::Value, Self::Rest);
}

// impl ExtractParams<0, ()> for Params<()> {
//     type Value = ();
//     type Rest = Params<()>;
//     fn extract_params(self) -> (Self::Value, Self::Rest) {
//         ((), Params(()))
//     }
// }
impl<E: Extractable<Input = ()>> ExtractParams<0, E> for Params<()> {
    type Value = E::Output;
    type Rest = Params<()>;
    fn extract_params(self) -> (Self::Value, Self::Rest) {
        (E::extract(()), Params(()))
    }
}

pub trait ExtractField<F, T> {
    fn field(&self, f: &Field<T>) -> F;
}

pub trait AsField
where
    Self: Sized,
{
    fn as_field() -> Field<Self>;
}

pub trait Shift<const I: u8> {
    type Target;
    fn shift(self) -> Self::Target;
}

pub trait ExtractValue {
    type Value;
    fn extract_value(self) -> Self::Value;
}

pub trait Flattern {
    type Output;
    fn flattern(self) -> Self::Output;
}
impl Flattern for () {
    type Output = ();
    fn flattern(self) -> Self::Output {
        ()
    }
}

impl<const I: u8, T> F<I, T> {
    pub fn define(self, value: T) -> D<I, T> {
        D::<I, T>(value)
    }
}

impl<const I: u8, T> A<I, T> for D<I, T> {}
impl<const I: u8, T> A<I, T> for U<I, T> {}

impl<const I: u8, const J: u8, T> Shift<J> for D<I, T> {
    type Target = D<J, T>;
    fn shift(self) -> Self::Target {
        D::<J, T>(self.0)
    }
}
impl<const I: u8, const J: u8, T> Shift<J> for U<I, T> {
    type Target = U<J, T>;
    fn shift(self) -> Self::Target {
        U::<J, T>(PhantomData)
    }
}
impl<const I: u8, const J: u8, T> Shift<J> for F<I, T> {
    type Target = F<J, T>;
    fn shift(self) -> Self::Target {
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

impl Params<()> {
    pub fn defined(self) -> Self {
        self
    }
}

pub trait AsParams {
    type Defined;
    type Undefined;
    fn as_params() -> Self::Undefined;
}

use constructivism_macro::implement_constructivism_core; /* @constructivist-no-expose */
implement_constructivism_core!(16); /* @constructivist-no-expose */

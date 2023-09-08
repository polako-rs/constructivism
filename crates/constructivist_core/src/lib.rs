use std::marker::PhantomData;

use constructivist_macro_support::*;

pub mod traits {
    pub use super::Construct;
    pub use super::A;
    pub use super::Singleton;
    pub use super::ExtractField;
    pub use super::AsField;
    pub use super::DefinedValue;
    pub use super::Flattern;
    pub use super::NonUnit;
    pub use super::Strip;
    pub use super::New;
    pub use super::Object;
    pub use super::Mixin;
    pub use super::Mixed;
}


pub trait Construct {
    type Props: Extractable;
    fn construct(props: Self::Props) -> Self;
}

pub trait Object: Construct {
    type Extends: Object;
    type Fields: Singleton;
    type Methods: Singleton;
    type MixedProps: Extractable;
    type Hierarchy;
    type ExpandedProps: Extractable;
    
    
    fn build<P, const I: u8>(p: P) -> Self::Hierarchy where P: ExtractParams<
        I, Self::MixedProps,
        Value = <Self::MixedProps as Extractable>::Output,
        Rest = <<<Self::Extends as Object>::ExpandedProps as Extractable>::Input as AsParams>::Defined
    >;
}

pub trait Mixin: Construct {
    type Fields<T: Singleton + 'static>: Singleton;
    type Methods<T: Singleton + 'static>: Singleton;
}


#[macro_export]
macro_rules! constructall {
    ($t:ty { $($f:ident: $e:expr,)+ }) => {
        {
            use $crate::traits::*;
            let fields = <<$t as $crate::Object>::Fields as $crate::Singleton>::instance();
            let props = <<$t as $crate::Object>::ExpandedProps as $crate::Extractable>::as_params();
            $(
                let param = &fields.$f;
                let field = param.field();
                let param = props.field(&field).define(param.value($e.into()));
                let props = props + param;
            )+
            let defined_props = props.defined();
            <$t as $crate::Object>::build(defined_props).flattern()
        }
        
    };
}

#[macro_export]
macro_rules! new {
    (@field $fields:ident $props:ident $f:ident $e:expr) => {
        let prop = &$fields.$f;
        let field = prop.field();
        let value = $props.field(&field).define(prop.value($e.into()));
        let $props = $props + value;
        $props.validate(&prop)();
    };
    (@fields $fields:ident $props:ident $f:ident: $e:expr) => {
        new!(@field $fields $props $f $e)
    };
    (@fields $fields:ident $props:ident $f:ident) => {
        new!(@field $fields $props $f $f);
        new!(@fields $fields $props $($rest)*)
    };
    (@fields $fields:ident $props:ident $f:ident: $e:expr,) => {
        new!(@field $fields $props $f $e);
    };
    (@fields $fields:ident $props:ident $f:ident,) => {
        new!(@field $fields $props $f $f);
        new!(@fields $fields $props $($rest)*)
    };
    (@fields $fields:ident $props:ident $f:ident: $e:expr, $($rest:tt)*) => {
        new!(@field $fields $props $f $e);
        new!(@fields $fields $props $($rest)*)
    };
    (@fields $fields:ident $props:ident $f:ident, $($rest:tt)*) => {
        new!(@field $fields $props $f $f);
        new!(@fields $fields $props $($rest)*)
    };
    ($t:ty { $($rest:tt)* } ) => {
        {
            use $crate::traits::*;
            type Fields = <$t as $crate::Object>::Fields;
            let fields = <<$t as $crate::Object>::Fields as $crate::Singleton>::instance();
            let props = <<$t as $crate::Object>::ExpandedProps as $crate::Extractable>::as_params();
            new!(@fields fields props $($rest)*);
            let defined_props = props.defined();
            <$t as $crate::Object>::build(defined_props)
        }
    };
}

impl Construct for () {
    type Props = ();
    
    fn construct(_: Self::Props)-> Self {
        ()
    }
}

impl Object for () {
    type Fields = ();
    type Methods = ();
    type Extends = ();
    type Hierarchy = ();
    // type Mixed = ();
    type MixedProps = ();
    type ExpandedProps = ();
    
    fn build<P, const I: u8>(_: P) -> Self::Hierarchy where P: ExtractParams<
        I, Self::MixedProps,
        Value = <Self::MixedProps as Extractable>::Output,
        Rest = <<<Self::Extends as Object>::ExpandedProps as Extractable>::Input as AsParams>::Defined
    > {
        ()
    }

}

pub struct Props<T>(T);
impl<T> Props<T> {
    pub fn validate<P>(&self, _: P) -> fn() -> () {
        || { }
    }

}

// impl<C: Object> Props<<<C::ExpandedProps as Extractable>::Input as AsParams>::Undefined> {
//     pub fn for_construct() -> Self {

//     }
// }

pub struct PropConflict<N>(PhantomData<N>);
impl<N> PropConflict<N> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
    pub fn validate<T>(&self, _: &Prop<N, T>) -> PropRedefined<N> {
        PropRedefined(PhantomData)
    }
}

pub struct PropRedefined<N>(PhantomData<N>);

pub struct Prop<N, T>(pub PhantomData<(N, T)>);
impl<N, T> Prop<N, T> {
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
impl<N: New<T>, T> Prop<N,T> {
    pub fn value(&self, value: T) -> N {
        N::new(value)
    }
} 

pub trait Methods<Protocol: ?Sized> {

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

pub trait A<const I: u8, T> { }

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

pub trait Mixed<Right> where Self: Sized {
    type Output;
    fn split(mixed: Self::Output) -> (Self, Right);
}
pub struct Mix<L, R>(PhantomData<(L, R)>);

impl<O: AsParams, L: Extractable, R: Extractable> Extractable for Mix<L, R> where
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

// impl ExtractParams<0, ()> for Props<()> {
//     type Value = ();
//     type Rest = Props<()>;
//     fn extract_params(self) -> (Self::Value, Self::Rest) {
//         ((), Props(()))
//     }
// }
impl<E: Extractable<Input = ()>> ExtractParams<0, E> for Props<()>
{
    type Value = E::Output;
    type Rest = Props<()>;
    fn extract_params(self) -> (Self::Value, Self::Rest) {
        (E::extract(()), Props(()))
    }
}


pub trait ExtractField<F, T> {
    fn field(&self, f: &Field<T>) -> F;
}

pub trait AsField where Self: Sized {
    fn as_field() -> Field<Self>;
}

pub trait Shift<const I: u8> {
    type Target;
    fn shift(self) -> Self::Target;
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

pub trait Strip {
    type Output;
    fn strip(self) -> Self::Output;
}

impl<const I: u8, T> F<I, T> {
    pub fn define(self, value: T) -> D<I, T> {
        D::<I, T>(value)
    }
}


impl<const I: u8, T> A<I, T> for D<I, T> { }
impl<const I: u8, T> A<I, T> for U<I, T> { }

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

impl Props<()> {
    pub fn defined(self) -> Self {
        self
    }
}

pub trait AsParams {
    type Defined;
    type Undefined;
    fn as_params() -> Self::Undefined;
}



impl<T: NonUnit> Strip for (T, ()) {
    type Output = T;
    fn strip(self) -> Self::Output {
        self.0
    }
}

impl<T: NonUnit, S:Strip> Strip for (T, S) {
    type Output = (T, S::Output);
    fn strip(self) -> Self::Output {
        (self.0, (self.1.strip()))
    }
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

construct_implementations! { }

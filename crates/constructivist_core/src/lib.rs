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
    pub use super::AsProps;
    pub use super::Strip;
}

pub trait Construct {
    type Fields: Singleton;
    type Props: AsProps;
    type Wraps: Construct;
    type Wrapped;
    type WrappedProps: AsProps;
    fn construct_fields() -> &'static Self::Fields;
    // fn construct_props() -> Props<<<Self as Construct>::UndefinedProps as IntoProps>::Target>;
    fn construct(props: Self::Props)-> Self;
    // fn split_props<T: SplitAt<{I}>>(props: T) -> (T::Left, T::Right){
    //     T::split(props)
    // }
    fn construct_all<P>(props: P) -> <Self as Construct>::Wrapped
    where 
        Self: Sized,
        P: DefinedValues<Self::Props, Output = <<<Self as Construct>::Wraps as Construct>::WrappedProps as AsProps>::Defined >;
}

#[macro_export]
macro_rules! construct {
    ($t:ty { $($f:ident: $e:expr,)+ }) => {
        {
            use $crate::traits::*;
            let fields = <$t as $crate::Construct>::construct_fields();
            let props = <<$t as $crate::Construct>::Props as $crate::AsProps>::as_props();
            $(
                let param = &fields.$f;
                let field = param.field();
                let param = props.field(&field).define($e.into());
                let props = props + param;
            )+
            let (props, _rest) = <$crate::Props<_> as $crate::DefinedValues<<$t as $crate::Construct>::Props>>::extract_values(props);
            <$t as $crate::Construct>::construct(props)
        }
        
    };
}
#[macro_export]
macro_rules! constructall {
    ($t:ty { $($f:ident: $e:expr,)+ }) => {
        {
            use $crate::traits::*;
            let fields = <$t as $crate::Construct>::construct_fields();
            let props = <<$t as $crate::Construct>::WrappedProps as $crate::AsProps>::as_props();
            $(
                let param = &fields.$f;
                let field = param.field();
                let param = props.field(&field).define($e.into());
                let props = props + param;
            )+
            let defined_props = props.defined();
            <$t as $crate::Construct>::construct_all(defined_props).flattern()
        }
        
    };
}

#[macro_export]
macro_rules! new {
    (@field $fields:ident $props:ident $f:ident $e:expr) => {
        let prop = &$fields.$f;
        let field = prop.field();
        let value = $props.field(&field).define($e.into());
        let $props = $props + value;
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
            type Fields = <$t as $crate::Construct>::Fields;
            let fields = <$t as $crate::Construct>::construct_fields();
            let props = <<$t as $crate::Construct>::WrappedProps as $crate::AsProps>::as_props();
            new!(@fields fields props $($rest)*);
            let defined_props = props.defined();
            <$t as $crate::Construct>::construct_all(defined_props)
        }
    };
}

impl Construct for () {
    type Fields = ();
    type Props = ();
    type Wraps = ();
    type Wrapped = ();
    type WrappedProps = ();
    fn construct_fields() -> &'static Self::Fields {
        &()
    }
    fn construct(_: Self::Props)-> Self {
        ()
    }
    fn construct_all<P>(_: P) -> <Self as Construct>::Wrapped
    where Self: Sized, P: DefinedValues<
        Self::Props,
        Output = <<<Self as Construct>::Wraps as Construct>::WrappedProps as AsProps>::Defined
    > {
        ()
    }
}

pub struct Props<T>(T);
pub struct Prop<T, V>(pub PhantomData<(T, V)>);
impl<T, V> Prop<T, V> {
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

impl Props<()> {
    pub fn defined(self) -> Self {
        self
    }
}
impl<T> DefinedValues<()> for Props<T> {
    type Output = Self;
    fn extract_values(self) -> ((), Self::Output) {
        ((), self)
    }
}
impl DefinedValues<()> for () {
    type Output = Props<()>;
    fn extract_values(self) -> ((), Self::Output) {
        ((), Props(()))
    }
}

pub trait AsFlatProps {
    type Defined;
    type Undefined;
    fn as_flat_props() -> Self::Undefined;
}

impl<T: AsFlatProps> AsProps for T {
    type Defined = Props<T::Defined>;
    type Undefined = Props<T::Undefined>;
    fn as_props() -> Self::Undefined {
        Props(T::as_flat_props())
    }
}
impl<T: AsField> AsFlatProps for (T,) {
    type Defined = (D<0, T>,);
    type Undefined = (U<0, T>,);
    fn as_flat_props() -> Self::Undefined {
        (U::<0u8, _>(PhantomData),)
    }
}

pub trait AsProps {
    type Undefined;
    type Defined;
    fn as_props() -> Self::Undefined;
}
impl AsProps for () {
    type Undefined = Props<()>;
    type Defined = Props<()>;
    fn as_props() -> Self::Undefined {
        Props(())
    }
}

pub trait JoinProps<T> {
    type DefinedResult;
    type UndefinedResult;
    fn join() -> Self::UndefinedResult;
}

impl <T: AsField> AsFlatProps for T {
    type Defined = (D<0, T>,);
    type Undefined = (U<0, T>,);
    fn as_flat_props() -> Self::Undefined {
        (U::<0, _>(PhantomData),)
    }
}

construct_implementations! { }

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

struct X;
struct Y;
struct Z;
impl NonUnit for X { }
impl NonUnit for Y { }
impl NonUnit for Z { }

fn test() {
    let s1 = (X, ());
    let r1 = s1.strip();

    let s2 = (X, (Y, ()));
    let r2 = s2.strip();

    let s3 = (X, (Y, (Z, ())));
    let r3 = s3.strip();
    let (x, (y, z)) = r3;

    let s4 = (X, (Y, (Z, (X, ()))));
    let r4 = s4.strip();
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
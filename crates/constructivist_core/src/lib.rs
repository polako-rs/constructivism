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
    pub use super::New;
    pub use super::Object;
    pub use super::Mixin;
    // pub use super::AsProps;
}


pub trait Construct {
    type Props: AsProps + Extractable;
    fn construct(props: Self::Props) -> Self;
}

pub trait Object: Construct {
    type Extends: Object;
    type Fields: Singleton;
    type Methods: Singleton;
    // type Mixed;
    type MixedProps: AsProps;
    type Hierarchy;
    type ExpandedProps: AsProps;
    // fn mixed(props: Self::MixedProps) -> Self::Mixed;
    // fn build<P>(props: P) -> <Self as Object>::Hierarchy where 
    //     Self: Sized,
    //     P: DefinedValues<Self::MixedProps, Output = <<<Self as Object>::Extends as Object>::ExpandedProps as AsProps>::Defined >;
    fn construct_all<P>(props: P) -> <Self as Object>::Hierarchy where 
        Self: Sized,
        P: DefinedValues<Self::MixedProps, Output = <<<Self as Object>::Extends as Object>::ExpandedProps as AsProps>::Defined >;

    fn build<P, const I: u8>(p: P) -> Self::Hierarchy where P: ExtractParams<
        I, Self::Props,
        Value = <Self::Props as Extractable>::Output,
        Rest = <<Self::Extends as Object>::ExpandedProps as AsProps>::Defined
    >;
}

pub trait Mixin: Construct {
    type Fields<T: Singleton + 'static>: Singleton;
    type Methods<T: Singleton + 'static>: Singleton;
}


#[macro_export]
macro_rules! construct {
    ($t:ty { $($f:ident: $e:expr,)+ }) => {
        {
            use $crate::traits::*;
            let fields = <<$t as $crate::Object>::Fields as $crate::Singleton>::instance();
            let props = <<$t as $crate::Construct>::Props as $crate::AsProps>::as_props();
            $(
                let param = &fields.$f;
                let field = param.field();
                let param = props.field(&field).define(param.value($e.into()));
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
            let fields = <<$t as $crate::Object>::Fields as $crate::Singleton>::instance();
            let props = <<$t as $crate::Object>::ExpandedProps as $crate::AsProps>::as_props();
            $(
                let param = &fields.$f;
                let field = param.field();
                let param = props.field(&field).define(param.value($e.into()));
                let props = props + param;
            )+
            let defined_props = props.defined();
            <$t as $crate::Object>::construct_all(defined_props).flattern()
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
            let props = <<$t as $crate::Object>::ExpandedProps as $crate::AsProps>::as_props();
            new!(@fields fields props $($rest)*);
            let defined_props = props.defined();
            <$t as $crate::Object>::construct_all(defined_props)
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
    fn construct_all<P>(_: P) -> <Self as Object>::Hierarchy
    where Self: Sized, P: DefinedValues<
        Self::MixedProps,
        Output = <<<Self as Object>::Extends as Object>::ExpandedProps as AsProps>::Defined
    > {
        ()
    }
    fn build<P, const I: u8>(_: P) -> Self::Hierarchy where P: ExtractParams<
        I, Self::Props,
        Value = <Self::Props as Extractable>::Output,
        Rest = <<Self::Extends as Object>::ExpandedProps as AsProps>::Defined
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
    type Input;
    type Output;
    fn extract(input: Self::Input) -> Self::Output;
}

impl Extractable for () {
    type Input = ();
    type Output = ();
    fn extract(_: Self::Input) -> Self::Output {
        ()
    }
}


pub trait ExtractParams<const S: u8, T> { 
    type Value;
    type Rest;
    fn extract_params(self) -> (Self::Value, Self::Rest);
}

impl ExtractParams<0, ()> for Props<()> {
    type Value = ();
    type Rest = Props<()>;
    fn extract_params(self) -> (Self::Value, Self::Rest) {
        ((), Props(()))
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

impl<T: AsFlatProps> JoinProps<T> for () {
    type DefinedResult = T::Defined;
    type UndefinedResult = T::Undefined;
    fn join() -> Self::UndefinedResult {
        T::as_flat_props()
    }
}


// impl<T: AsProps> JoinProps<T> for () {
//     type DefinedResult = T::Defined;
//     type UndefinedResult = T::Undefined;
//     fn join() -> Self::UndefinedResult {
//         T::as_props()
//     }
// }


pub struct Join<L, R>(pub PhantomData<(L, R)>);

impl<L: AsFlatProps, R: AsFlatProps> JoinProps<R::Undefined> for Join<L, R> where
<R as AsFlatProps>::Undefined: JoinProps<L::Undefined>
{
    type DefinedResult = <<R as AsFlatProps>::Undefined as JoinProps<L::Undefined>>::DefinedResult;
    type UndefinedResult = <<R as AsFlatProps>::Undefined as JoinProps<L::Undefined>>::UndefinedResult;
    fn join() -> Self::UndefinedResult {
        <R as AsFlatProps>::Undefined::join()
    }
}

// impl<L: AsFlatProps, R: AsFlatProps + JoinProps<L::Undefined>> AsProps for Join<L, R> {
//     type Defined = <R as JoinProps<L::Undefined>>::DefinedResult;
//     type Undefined = <R as JoinProps<L::Undefined>>::UndefinedResult;
//     fn as_props() -> Self::Undefined {
//         Self::join()
//     }
// }

// impl<L: AsFlatProps, R: AsFlatProps + JoinProps<L::Undefined>> AsProps for Join<L, R> {
//     type Defined = Props<<R as JoinProps<L::Undefined>>::DefinedResult>;
//     type Undefined = Props<<R as JoinProps<L::Undefined>>::UndefinedResult>;
//     fn as_props() -> Self::Undefined {
//         Props(Self::join())
//     }
// }
impl<L: AsFlatProps, R: AsFlatProps> AsFlatProps for Join<L, R> where
<R as AsFlatProps>::Undefined: JoinProps<L::Undefined>
{
    type Defined = <<R as AsFlatProps>::Undefined as JoinProps<L::Undefined>>::DefinedResult;
    type Undefined = <<R as AsFlatProps>::Undefined as JoinProps<L::Undefined>>::UndefinedResult;
    fn as_flat_props() -> Self::Undefined {
        Self::join()
    }
}
impl<L: AsFlatProps> AsFlatProps for Join<L, ()> where
{
    type Defined = L::Defined;
    type Undefined = L::Undefined;
    fn as_flat_props() -> Self::Undefined {
        L::as_flat_props()
    }
}
// impl<L: AsFlatProps> AsProps for Join<L, ()> {
//     type Defined = Props<L::Defined>;
//     type Undefined = Props<L::Undefined>;
//     fn as_props() -> Self::Undefined {
//         Props(L::as_flat_props())
//     }
// }

// impl<L: AsFlatProps + JoinProps<R::Undefined>, R: AsFlatProps> JoinProps<L::Undefined> for Join<L, R> {
//     type DefinedResult = L::DefinedResult;
//     type UndefinedResult = L::UndefinedResult;
//     fn join() -> Self::UndefinedResult {
//         L::join()
//     }
// }

// impl<L: AsFlatProps + JoinProps<R::Undefined>, R: AsFlatProps> AsProps for Join<L, R> {
//     type Defined = Props<<L as JoinProps<R::Undefined>>::DefinedResult>;
//     type Undefined = Props<<L as JoinProps<R::Undefined>>::UndefinedResult>;
//     fn as_props() -> Self::Undefined {
//         Props(Self::join())
//     }
// }
// impl<L: AsFlatProps> AsProps for Join<L, ()> {
//     type Defined = Props<L::Defined>;
//     type Undefined = Props<L::Undefined>;
//     fn as_props() -> Self::Undefined {
//         L::as_props()
//     }
// }
// impl AsFlatProps for () {
//     type Defined = ();
//     type Undefined = ();
//     fn as_flat_props() -> Self::Undefined {
//         ()
//     }
// }

// impl <T: AsField> AsFlatProps for T {
//     type Defined = (D<0, T>,);
//     type Undefined = (U<0, T>,);
//     fn as_flat_props() -> Self::Undefined {
//         (U::<0, _>(PhantomData),)
//     }
// }

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

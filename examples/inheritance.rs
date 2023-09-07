use constructivist_core::new;
use constructivist_core::AsProps;
use constructivist_core::Construct;
use constructivist_core::DefinedValues;
use constructivist_core::Methods;
use constructivist_core::Mixin;
use constructivist_core::Object;
use constructivist_core::Singleton;
use constructivist_core::D;
use constructivist_macro_support::Construct;
use constructivist_macro_support::Mixin;
use std::fmt::Debug;
use std::marker::PhantomData;

macro_rules! methods {
    ($t:ty) => {
        <$t as Object>::Methods::instance()
    };
}

// define some trait
pub trait AddChildren {
    fn add_children<P: Debug, C: Debug>(&self, parent: P, children: Vec<C>);
}

// and implement it for some Construct
impl<T: Methods<Div>> AddChildren for T {
    fn add_children<P: std::fmt::Debug, C: std::fmt::Debug>(&self, parent: P, children: Vec<C>) {
        println!("Calling add_children({parent:?}, {children:?})")
    }
}

pub fn main() {
    // direct use:
    // let border_methods = <Border as Object>::Methods::instance();
    // border_methods.add_children(1, vec![23]);

    // with custom macro:
    methods!(Div).add_children(1, vec![23]);
    // let _this = new!(Div {
    //     width: 25.,
    //     // visible: false,
    // });
    let _ = {
        use constructivist_core::traits::*;
        type Fields = <Div as constructivist_core::Object>::Fields;
        let fields = <<Div as constructivist_core::Object>::Fields as constructivist_core::Singleton>::instance();
        let props = <<Div as constructivist_core::Object>::ExpandedProps as constructivist_core::AsProps>::as_props();
        let prop = &fields.width;
        let field = prop.field();
        let value = props.field(&field).define(prop.value(25.));
        let props = props + value;
        props.validate(&prop)();
        let defined_props = props.defined();
        <Div as constructivist_core::Object>::build(defined_props)
    };
    // methods!(Border).add_children(1, vec![23]);
    // let _this = new!(Border {
    //     width: 25.,
    //     // visible: false,
    // });
    // let a = methods!(Border).width((&this).into());
    // TODO: add this example to compilation tests
    // uncoment next to get complation error (Slider doesn't have Div in hierarchy)
    // methods!(Slider).add_children(1, vec![23]);
}

#[allow(dead_code)]
#[derive(Mixin)]
pub struct Visibility {
    visible: bool,
}

#[allow(dead_code)]
#[derive(Construct)]
pub struct Node {
    width: f32,
    height: f32,
}

#[allow(dead_code)]
#[derive(Construct)]
#[extends(Node)]
// #[mixin(Visibility)]
pub struct Div {
    color: f32,
    alpha: f32,
}

// #[allow(dead_code)]
// // #[derive(Construct)]
// // #[extends(Div)]
// pub struct Border {
//     border_width: f32,
// }
// // Recursive expansion of Construct macro
// // =======================================

// mod border_construct {
//     use super::*;
//     pub struct Fields {
//         #[allow(unused_variables)]
//         pub border_width: ::constructivist_core::Prop<border_width, f32>,
//     }
//     pub struct Methods;

//     impl ::constructivist_core::Singleton for Fields {
//         fn instance() -> &'static Self {
//             &Fields {
//                 border_width: ::constructivist_core::Prop(::std::marker::PhantomData),
//             }
//         }
//     }
//     impl ::constructivist_core::Singleton for Methods {
//         fn instance() -> &'static Self {
//             &Methods
//         }
//     }
//     impl ::std::ops::Deref for Fields {
//         type Target = <Div as ::constructivist_core::Object>::Fields;
//         fn deref(&self) -> &Self::Target {
//             < <Div as ::constructivist_core::Object> ::Fields as ::constructivist_core::Singleton> ::instance()
//         }
//     }
//     impl ::constructivist_core::Methods<Border> for Methods {}

//     impl ::std::ops::Deref for Methods {
//         type Target = <Div as ::constructivist_core::Object>::Methods;
//         fn deref(&self) -> &Self::Target {
//             < <Div as ::constructivist_core::Object> ::Methods as ::constructivist_core::Singleton> ::instance()
//         }
//     }
//     impl Default for border_width {
//         fn default() -> Self {
//             border_width(Default::default())
//         }
//     }
//     #[allow(non_camel_case_types)]
//     pub struct border_width(pub f32);

//     impl<T: Into<f32>> From<T> for border_width {
//         fn from(__value__: T) -> Self {
//             border_width(__value__.into())
//         }
//     }
//     impl ::constructivist_core::AsField for border_width {
//         fn as_field() -> ::constructivist_core::Field<Self> {
//             ::constructivist_core::Field::new()
//         }
//     }
//     impl ::constructivist_core::AsFlatProps for border_width {
//         type Defined = (::constructivist_core::D<0, border_width>,);
//         type Undefined = (::constructivist_core::U<0, border_width>,);
//         fn as_flat_props() -> Self::Undefined {
//             (::constructivist_core::U::<0, _>(::std::marker::PhantomData),)
//         }
//     }
//     impl ::constructivist_core::New<f32> for border_width {
//         fn new(from: f32) -> border_width {
//             border_width(from)
//         }
//     }
// }
// impl ::constructivist_core::NonUnit for Border {}

// impl ::constructivist_core::Construct for Border {
//     type Props = (border_construct::border_width,);
//     fn construct(props: Self::Props) -> Self {
//         let (border_construct::border_width(mut border_width),) = props;
//         Self { border_width }
//     }
// }
// impl ::constructivist_core::Object for Border {
//     type Extends = Div;
//     type Fields = border_construct::Fields;
//     type Methods = border_construct::Methods;
//     type MixedProps = (border_construct::border_width,);
//     type Hierarchy = (
//         Self,
//         <Self::Extends as ::constructivist_core::Object>::Hierarchy,
//     );
//     type ExpandedProps = ::constructivist_core::Join<
//         (border_construct::border_width,),
//         <Self::Extends as ::constructivist_core::Object>::ExpandedProps,
//     >;
//     fn construct_all<P>(props:P) ->  <Self as ::constructivist_core::Object> ::Hierarchy where
//         Self:Sized,
//         P: ::constructivist_core::DefinedValues<
//             Self::MixedProps,
//             Output =  << <Self as ::constructivist_core::Object> ::Extends as ::constructivist_core::Object> ::ExpandedProps as ::constructivist_core::AsProps> ::Defined
//     >{
//         let (args, props) = props.extract_values();
//         // let p: &dyn DefinedValues< <Self::Extends as Object>::MixedProps>  = &props;
//         let rest = <<Self as ::constructivist_core::Object> ::Extends as ::constructivist_core::Object> ::construct_all(props);
//         (<Self as ::constructivist_core::Construct> ::construct(args), rest)
//     }
// }

// #[allow(dead_code)]
// #[derive(Construct)]
// #[extends(Node)]
// pub struct Slider {
//     min: f32,
//     max: f32,
//     value: f32,
// }

pub struct ImutableMethod<T, A> {
    body: fn(&T, A),
}
pub struct MutableMethod<T, A> {
    body: fn(&mut T, A),
}

pub enum Method<T, A> {
    Imutable(ImutableMethod<T, A>),
    Mutable(MutableMethod<T, A>),
}

impl Node {
    pub fn extend(&mut self, w: f32, h: f32) {
        self.width += w;
        self.height += h;
    }
}

pub trait NodeMethods {
    fn extend(&self) -> Method<Node, (f32, f32)>;
}

impl<T: Methods<Node>> NodeMethods for T {
    fn extend(&self) -> Method<Node, (f32, f32)> {
        Method::Mutable(MutableMethod {
            body: |node, (w, h)| node.extend(w, h),
        })
    }
}

pub struct Field<T>(PhantomData<T>);

pub struct Disabled(bool);

pub struct InputFields<E: Singleton + 'static> {
    pub disabled: Field<Disabled>,
    __extends__: PhantomData<E>,
}

impl<E: Singleton + 'static> Singleton for InputFields<E> {
    fn instance() -> &'static Self {
        &InputFields {
            disabled: Field(PhantomData),
            __extends__: PhantomData,
        }
    }
}
impl<E: Singleton + 'static> std::ops::Deref for InputFields<E> {
    type Target = E;
    fn deref(&self) -> &Self::Target {
        E::instance()
    }
}
pub struct Visible(bool);
pub struct VisibleFields<E: Singleton + 'static> {
    pub visible: Field<Visible>,
    __extends__: PhantomData<E>,
}

impl<E: Singleton + 'static> Singleton for VisibleFields<E> {
    fn instance() -> &'static Self {
        &VisibleFields {
            visible: Field(PhantomData),
            __extends__: PhantomData,
        }
    }
}
impl<E: Singleton + 'static> std::ops::Deref for VisibleFields<E> {
    type Target = E;
    fn deref(&self) -> &Self::Target {
        E::instance()
    }
}

pub struct Width(f32);
pub struct Height(f32);
pub struct DivFields {
    pub width: Field<Width>,
    pub height: Field<Height>,
}

impl Singleton for DivFields {
    fn instance() -> &'static Self {
        &DivFields {
            width: Field(PhantomData),
            height: Field(PhantomData),
        }
    }
}

pub struct ButtonFields;

impl std::ops::Deref for ButtonFields {
    type Target = InputFields<VisibleFields<DivFields>>;
    fn deref(&self) -> &Self::Target {
        InputFields::<VisibleFields<DivFields>>::instance()
    }
}

fn test_mixins() {
    let btn = ButtonFields;
    let _ = btn.visible;
}

pub trait Extract {
    type Input;
    type Output;
    fn extract(input: Self::Input) -> Self::Output;
}

pub struct Size<const S: u8>;

impl Extract for () {
    type Input = ();
    type Output = ();
    fn extract(input: Self::Input) -> Self::Output {
        ()
    }
}

impl<T0> Extract for (T0,) {
    type Input = (D<0, T0>,);
    type Output = (T0,);
    fn extract(input: Self::Input) -> Self::Output {
        (input.0 .0,)
    }
}

impl<T0, T1> Extract for (T0, T1) {
    type Input = (D<0, T0>, D<1, T1>);
    type Output = (T0, T1);
    fn extract(input: Self::Input) -> Self::Output {
        (input.0 .0, input.1 .0)
    }
}

pub trait Mixed<const S: u8> {
    type Input;
    type Output;
    fn extract(input: Self::Input) -> Self::Output;
}

impl<T0, T1, M: Mixed<2, Input = (D<0, T0>, D<1, T1>)>> Extract for M {
    type Input = (D<0, T0>, D<1, T1>);
    type Output = M::Output;
    fn extract(input: Self::Input) -> Self::Output {
        M::extract(input)
    }
}

struct Mix<L, R>(PhantomData<(L, R)>);

impl<L0, R0, L: Extract<Input = (D<0, L0>,)>, R: Extract<Input = (D<0, R0>,)>> Mixed<2> for Mix<L, R>
{
    type Input = (D<0, L0>, D<1, R0>);
    type Output = (L::Output, R::Output);
    fn extract(input: Self::Input) -> Self::Output {
        let (d0, d1) = input;
        let d1 = D::<0, _>(d1.0);
        (L::extract((d0,)), R::extract((d1,)))
    }
}

pub trait ExtractProps<const S: u8, T> {
    type Value;
    type Rest;

    fn extract_props(self) -> (Self::Value, Self::Rest);
}

struct Params<T>(T);

impl<T0, T1, T2, E: Extract<Input = (T0,)>> ExtractProps<1, E> for Params<(T0, T1, T2)> {
    type Value = E::Output;
    type Rest = Params<(T1, T2)>;
    fn extract_props(self) -> (Self::Value, Self::Rest) {
        let (p0, p1, p2) = self.0;
        (E::extract((p0,)), Params((p1, p2)))
    }
}

impl<T0, T1, T2, E: Extract<Input = (T0, T1)>> ExtractProps<2, E> for Params<(T0, T1, T2)> {
    type Value = E::Output;
    type Rest = Params<(T2,)>;
    fn extract_props(self) -> (Self::Value, Self::Rest) {
        let (p0, p1, p2) = self.0;
        (E::extract((p0, p1)), Params((p2,)))
    }
}

// pub trait IntoParams {
//     type Output;
//     fn into_params() -> Self::Output;
// }

// impl<T0> IntoParams for (T0,) {
//     type Output = Params<(D<0, T0>),>;
//     fn into_params() -> Self::Output {
//         Params((D::<0,_>()))
//     }
// }

trait Obj {
    type Props: Extract;
    type Extends: Obj;

    fn construct<P, const I: u8>(p: P)
    where
        P: ExtractProps<I, Self::Props, Value = <Self::Props as Extract>::Output>;
}

impl Obj for () {
    type Extends = ();
    type Props = ();
    fn construct<P, const I: u8>(p: P)
    where
        P: ExtractProps<I, Self::Props, Value = <Self::Props as Extract>::Output>,
    {
    }
}

struct O1;

impl Obj for O1 {
    type Props = (usize,);
    type Extends = ();
    // type ExpandedProps = Params<()>;
    fn construct<P, const I: u8>(p: P)
    where
        P: ExtractProps<I, Self::Props, Value = <Self::Props as Extract>::Output>,
    {
        let (v, rest) = p.extract_props();
    }
}

struct O2;

impl Obj for O2 {
    type Props = Mix<(usize,), (usize,)>;
    type Extends = O2;
    fn construct<P, const I: u8>(p: P)
    where
        P: ExtractProps<
            I,
            Self::Props,
            Value = <Self::Props as Extract>::Output,
            // Rest = <Self::Extends as AsProps>::Defined,
        >,
    {
        let (v, rest) = p.extract_props();
        // <Self::Extends as Obj>::construct(rest);
    }
}

fn ttt() {
    // let m = (PhantomData);
    // <((usize,), (usize,)) as Extract>::extract(input)
    let v = <Mix<(usize,), (usize,)> as Extract>::extract((D::<0, _>(0), D::<1, _>(1)));

    let p1 = Params((D::<0, _>(0), D::<1, _>(1), D::<2, _>(2)));
    let p2 = Params((D::<0, _>(0), D::<1, _>(1), D::<2, _>(2)));
    let p3 = Params((D::<0, _>(0), D::<1, _>(1), D::<2, _>(2), D::<3, _>(3)));

    O1::construct(p1);
    O2::construct(p2);
    // O1::construct(p3);
}

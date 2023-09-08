use constructivist_core::new;
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
        let fields = <<Div as constructivist_core::Object>::Fields as constructivist_core::Singleton>::instance();
        let props = <<Div as constructivist_core::Object>::ExpandedProps as constructivist_core::Extractable>::as_params();
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
#[mixin(Visibility)]
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

// impl<L0, R0, M: Mixed<1, 1, Input = (D<0, L0>, D<1, R0>)>> Extract for M {
//     type Input = (D<0, L0>, D<1, R0>);
//     type Output = M::Output;
//     fn extract(input: Self::Input) -> Self::Output {
//         M::extract(input)
//     }
// }
// impl<L0, R0, M: Mixed<1, 2, Input = (D<0, L0>, D<1, R0>, D<2, R0>)>> Extract for M {
//     type Input = (D<0, L0>, D<1, R0>, D<2, R0>);
//     type Output = M::Output;
//     fn extract(input: Self::Input) -> Self::Output {
//         M::extract(input)
//     }
// }

// impl<const L: u8, const R: u8, M: Mixed<L, R>> Extract for M {
//     type Input = M::Input;
//     type Output = M::Output;
//     fn extract(input: Self::Input) -> Self::Output {
//         M::extract(input)
//     }
// }

impl<O, L: Extract, R: Extract> Extract for Mix<L, R> where
L::Input: Mixed<R::Input, Output = O>,
{
    type Input = O;
    type Output = (L::Output, R::Output);
    fn extract(input: Self::Input) -> Self::Output {
        let (left, right) = <L::Input as Mixed<R::Input>>::split(input);
        (L::extract(left), R::extract(right))
    }
}

pub trait Mixed<Right> where Self: Sized {
    type Output;
    fn split(mixed: Self::Output) -> (Self, Right);
}

impl<L0, R0> Mixed<(D<0, R0>,)> for (D<0, L0>,) {
    type Output = (D<0, L0>, D<1, R0>);
    fn split(joined: Self::Output) -> (Self, (D<0, R0>,)) {
        let (l0, r0) = joined;
        let r0 = D::<0, _>(r0.0);
        ((l0,), (r0,))

    }
}

impl<L0, R0, R1> Mixed<(D<0, R0>, D<1, R1>)> for (D<0, L0>,) {
    type Output = (D<0, L0>, D<1, R0>, D<2, R1>);
    fn split(joined: Self::Output) -> (Self, (D<0, R0>, D<1, R1>)) {
        let (l0, r0, r1) = joined;
        let r0 = D::<0, _>(r0.0);
        let r1 = D::<1, _>(r1.0);
        ((l0,), (r0, r1))
        
    }
}

impl<L0, L1, R0> Mixed<(D<0, R0>,)> for (D<0, L0>, D<1, L1>) {
    type Output = (D<0, L0>, D<1, L1>, D<2, R0>);
    fn split(joined: Self::Output) -> (Self, (D<0, R0>,)) {
        let (l0, l1, r0) = joined;
        let r0 = D::<0, _>(r0.0);
        ((l0, l1), (r0,))
        
    }
}

struct Mix<L, R>(PhantomData<(L, R)>);


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

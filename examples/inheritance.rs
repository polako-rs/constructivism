use std::fmt::Debug;
use std::marker::PhantomData;
use constructivist_core::Methods;
use constructivist_core::Mixin;
use constructivist_core::Object;
use constructivist_core::new;
use constructivist_macro_support::Construct;
use constructivist_core::Construct;
use constructivist_core::Singleton;
use constructivist_macro_support::Mixin;


macro_rules! methods {
    ($t:ty) => { <$t as Object>::Methods::instance() };
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
    let border_methods = <Border as Object>::Methods::instance();
    border_methods.add_children(1, vec![23]);

    // with custom macro:
    methods!(Div).add_children(1, vec![23]);
    methods!(Border).add_children(1, vec![23]);
    let _this = new!(Border { 
        width: 25.,
        // visible: false,
    });
    // let a = methods!(Border).width((&this).into());
    // TODO: add this example to compilation tests
    // uncoment next to get complation error (Slider doesn't have Div in hierarchy)
    // methods!(Slider).add_children(1, vec![23]);
}

#[derive(Mixin)]
pub struct Visibility {
    visible: bool
}

#[allow(dead_code)]
#[derive(Construct)]
pub struct Node {
    width: f32,
    height: f32,
}


#[derive(Construct)]
#[extends(Node)]
// #[mixin(Visibility)]
pub struct Div {
    color: f32,

}

#[allow(dead_code)]
#[derive(Construct)]
#[extends(Div)]
pub struct Border {
    border_width: f32,
}

#[allow(dead_code)]
#[derive(Construct)]
#[extends(Node)]
pub struct Slider {
    min: f32,
    max: f32,
    value: f32
}





pub struct ImutableMethod<T, A> {
    body: fn(&T, A)
}
pub struct MutableMethod<T, A> {
    body: fn(&mut T, A)
}


pub enum Method<T, A> {
    Imutable(ImutableMethod<T, A>),
    Mutable(MutableMethod<T, A>)
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
            body: |node, (w, h)| node.extend(w, h)
        })
    }
}





pub struct Field<T>(PhantomData<T>);

pub struct Disabled(bool);


pub struct InputFields<E: Singleton + 'static> {
    pub disabled: Field<Disabled>,
    __extends__: PhantomData<E>
}

impl<E: Singleton + 'static> Singleton for InputFields<E> {
    fn instance() -> &'static Self {
        &InputFields {
            disabled: Field(PhantomData),
            __extends__: PhantomData
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
    __extends__: PhantomData<E>
}

impl<E: Singleton + 'static> Singleton for VisibleFields<E> {
    fn instance() -> &'static Self {
        &VisibleFields {
            visible: Field(PhantomData),
            __extends__: PhantomData
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
        &DivFields { width: Field(PhantomData), height: Field(PhantomData) }
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

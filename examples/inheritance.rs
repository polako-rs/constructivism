use std::fmt::Debug;
use constructivist_core::Methods;
use constructivist_macro_support::Construct;
use constructivist_core::Construct;
use constructivist_core::Singleton;


macro_rules! methods {
    ($t:ty) => { <$t as Construct>::Methods::instance() };
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
    let border_methods = <Border as Construct>::Methods::instance();
    border_methods.add_children(1, vec![23]);

    // with custom macro:
    methods!(Div).add_children(1, vec![23]);
    methods!(Border).add_children(1, vec![23]);
    // TODO: add this example to compilation tests
    // uncoment next to get complation error (Slider doesn't have Div in hierarchy)
    // methods!(Slider).add_children(1, vec![23]);
}

#[allow(dead_code)]
#[derive(Construct)]
pub struct Node {
    width: f32,
    height: f32,
}

#[derive(Construct)]
#[extends(Node)]
pub struct Div {

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
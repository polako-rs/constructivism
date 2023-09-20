// 0. Use `constructivism`
use constructivism::*;

// 1. `constructivism` constist of `Construct`s. `Construct` can
// be declared only in front of the another `Construct`. `constructivism`
// provide only `Nothing` construct. To define new `Constrcut` you can:

#[derive(Construct)]
#[construct(Node -> Nothing)]
pub struct Node {
    // You can provide custom default values.
    #[default(true)]
    visible: bool,
    position: (f32, f32),
}


// 2. You can `construct!` the `Construct`:
fn def_02() {
    let node = construct!(Node {
        position: (10., 10.),
        visible: true
    });
    assert_eq!(node.position.0, 10.);
    assert_eq!(node.visible, true);
}

// 3. You can skip declaration of non-required fields
fn def_03() {
    let node = construct!(Node {
        visible: false
    });
    assert_eq!(node.position.0, 0.)
}

// 4. You have to mark non-default fields with `#[required]` or you 
// get compilation error.
#[derive(Clone, Copy)]
pub struct Entity(usize);

#[derive(Construct)]
#[construct(Reference -> Nothing)]
struct Reference {
    #[required]
    target: Entity,
    count: usize,
}

// 5. You have to pass all required fiels to `construct!(..)`
// or you get compilation error
fn def_05() {
    let reference = construct!(Reference {
        target: Entity(23)
    });
    assert_eq!(reference.target.0, 23);
    assert_eq!(reference.count, 0);
}

// 6. The `Construct`'s relation (`Node -> Nothing`) called `Sequence` in
// constructivism. You define only local `Sequence` to the next `Construct`:
#[derive(Construct)]
#[construct(Rect -> Node)]
pub struct Rect {
    #[default((100., 100.))]
    size: (f32, f32)
}


// 7. The `Sequence` for `Rect` becomes `Rect -> Node -> Nothing`. The 
// unwrapped `#[derive(Construct)]` for Rect looks something like this:
// impl Construct for Rect {
//     type Sequence = (Rect, Node, Nothing)
//     /* other Construct types and methods */
// }

// 8. You can `construct!` the whole `Sequence` within the single call:
fn def_08() {
    let (rect, node, /* nothing */) = construct!(Rect {
        position: (10., 10.),
        size: (10., 10.),
        // I can skip fields, no `visible` here, for example
    });
    assert_eq!(rect.size.0, 10.);
    assert_eq!(node.position.1, 10.);
    assert_eq!(node.visible, true);
}

// 9. Every construct have it's own `Design`. You can implement
// methods for construct's design:
impl NodeDesign {
    #[allow(unused_variables)]
    pub fn move_to(&self, entity: Entity, position: (f32, f32)) { }
}
impl RectDesign {
    #[allow(unused_variables)]
    pub fn expand_to(&self, entity: Entity, size: (f32, f32)) { }
}

// 10. You can call methods on construct's design. Method resolves
// within the `Sequence` order:
fn def_10() {
    let rect_entity = Entity(20);
    design!(Rect).expand_to(rect_entity, (10., 10.));
    design!(Rect).move_to(rect_entity, (10., 10.));
}

// 11. You can define and insert `Segment` into construct's `Sequence`:
#[derive(Segment)]
pub struct Input {
    disabled: bool
}

#[derive(Construct)]
#[construct(Button -> Input -> Rect)]
pub struct Button {
    pressed: bool
}


// 12. The `Sequence` for `Button` becomes 
// `Button -> Input -> Rect -> Node -> Nothing`.
// You still can `construct!` the whole `Sequence` within
// the single call:
fn def_12() {
    let (button, input, rect, node) = construct!(Button {
        disabled: true
    });
    assert_eq!(button.pressed, false);
    assert_eq!(input.disabled, true);
    assert_eq!(rect.size.0, 100.);
    assert_eq!(node.position.0, 0.);
}

// 13. `Segment` has its own `Design` as well. And the method call
// resolves within the `Sequence` order as well. Segment's designes
// has one generic parameter - next segment/construct, so you need to
// respect it when implement Segment's designes:
impl<T> InputDesign<T> {
    #[allow(unused_variables)]
    fn focus(&self, entity: Entity) { }
}

fn def_13() {
    let btn = Entity(42);
    design!(Button).focus(btn);
}

// 14. Sometimes you want to implement Construct for foreign type
// or provide custom constructor for your type. You can use
// `derive_construct!` proc macro. `min: f32 = 0.` syntax defines `min`
// param with default value of 0. If you doesn't provide default value,
// this param counts as required.

pub struct ProgressBar {
    min: f32,
    val: f32,
    max: f32,
}

derive_construct! {
    // sequence
    ProgressBar -> Rect

    // params
    (min: f32 = 0., max: f32 = 1., val: f32 = 0.)
    
    // constructor
    {
        if max < min {
            max = min;
        }
        val = val.min(max).max(min);
        Self { min, val, max }
    }
}

// 15. Provided constructor will be called for instancing Range
fn def_15() {
    let (range, _, _) = construct!(ProgressBar { val: 100. });
    assert_eq!(range.min, 0.);
    assert_eq!(range.max, 1.);
    assert_eq!(range.val, 1.);
}

// 16. You can derive segments same way:
pub struct Range {
    min: f32,
    max: f32,
    val: f32,
}
derive_segment!{
    Range(min: f32 = 0., max: f32 = 1., val: f32 = 0.) {
        if max < min {
            max = min;
        }
        val = val.min(max).max(min);
        Self { min, val, max }
    }
}

#[derive(Construct)]
#[construct(Slider -> Range -> Rect)]
pub struct Slider;

fn def_16() {
    let (_slider, range, _, _) = construct!(Slider {
        val: 10.
    });
    assert_eq!(range.min, 0.0);
    assert_eq!(range.max, 1.0);
    assert_eq!(range.val, 1.0);
}

fn main() {
    def_02();
    def_03();
    def_05();
    def_08();
    def_10();
    def_12();
    def_13();
    def_15();
    def_16();
}
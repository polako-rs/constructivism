use constructivist::*;


// 1. You can derive `Construct`
#[derive(Construct)]
pub struct Node {
    // You can provide custom default values.
    #[default(true)]
    visible: bool,
    position: (f32, f32),
}

// 2. You can use construct! macro for instancing Node.
fn step_01() {
    let node = construct!(Node {
        position: (10., 10.),
        visible: true
    });
    assert_eq!(node.position.0, 10.);
    assert_eq!(node.visible, true);
}

// 3. You can skip declaration of default values
fn step_03() {
    let node = construct!(Node {
        visible: false
    });
    assert_eq!(node.position.0, 0.)
}

// 4. You have to mark non-default required fields with `#[required]`
// or you get compilation error.
pub struct Entity(usize);

#[derive(Construct)]
struct Reference {
    #[required]
    target: Entity,
    count: usize,
}

// 5. You have to pass required field to construct!(..) or you get compilation error
fn step_05() {
    let reference = construct!(Reference {
        target: Entity(23)
    });
    assert_eq!(reference.target.0, 23);
    assert_eq!(reference.count, 0);
}

// 6. You derive Construct using `constructable! { .. }`, define custom params
// and provide custom constructor. `min: f32 = 0.` syntax defines min param with 
// default value of 0. If you doesn't provide default value, this param counts as
// required.

pub struct Range {
    min: f32,
    val: f32,
    max: f32,
}

constructable! { 
    Range(min: f32 = 0., max: f32 = 1., val: f32 = 0.) {
        if max < min {
            max = min;
        }
        val = val.min(max).max(min);
        Self { min, val, max }
    }
}

// 7. Provided constructor will be called for instancing Range
fn step_07() {
    let range = construct!(Range {
        val: 100.
    });
    assert_eq!(range.min, 0.);
    assert_eq!(range.max, 1.);
    assert_eq!(range.val, 1.);
}

// 8. You can extend one construct from another construct

#[derive(Construct)]
#[extends(Node)]
pub struct Rect {
    #[default((100., 100.))]
    size: (f32, f32)
}

// 9. You can pass params for all structs in inheritance branch with single call
fn step_09() {
    let (rect, node) = construct!(Rect {
        position: (10., 10.),
        size: (10., 10.),
    });
    assert_eq!(rect.size.0, 10.);
    assert_eq!(node.position.1, 10.);
    assert_eq!(node.visible, true);
}

// 10. You can derive Mixin as well.
#[derive(Mixin)]
pub struct Input {
    disabled: bool
}

// 11. You can inject mixins into constructs:
#[derive(Construct)]
#[extends(Rect)]
#[mixin(Input)]
pub struct Button {
    pressed: bool
}

// 12. You can pass arguments to inheritance tree (with mixins) as well
fn step_12() {
    let (button, input, rect, node) = construct!(Button {
        disabled: true
    });
    assert_eq!(button.pressed, false);
    assert_eq!(input.disabled, true);
    assert_eq!(rect.size.0, 100.);
    assert_eq!(node.position.0, 0.);
}

// 13. When you extend from other construct, you extend from its mixins as well.
#[derive(Construct)]
#[extends(Button)]
pub struct Radio {
    #[required]
    value: String
}
fn step_13() {
    let (radio, button, input, rect, node) = construct!(Radio {
        value: "option_0"
    });
    assert_eq!(button.pressed, false);
    assert_eq!(input.disabled, false);
    assert_eq!(rect.size.0, 100.);
    assert_eq!(node.position.0, 0.);
    assert_eq!(radio.value, "option_0".to_string());
}

// TODO: docstring bypassing

// TODO: methods and method resolve order in the inheritance tree.

// TODO: mixable! { ... }

// TODO: union props

// TODO: generics

// TODO: nested results, like 
// let (radio, base) = construct!(*Radio { ... });

// TODO: nested construct inference (looks like possible):
// #[derive(Construct, Default)]
// pub struct Vec2 {
//     x: f32,
//     y: f32
// }

// #[derive(Construct)]
// pub struct Div {
//     position: Vec2,
//     size: Vec2,
// }

// fn step_inference() {
//     let div = construct!(Div {
//         position: {{ x: 23., y: 20. }},
//         size: {{ x: 23., y: 20. }}
//     })
// }

fn main() {
    step_01();
    step_03();
    step_05();
    step_07();
    step_09();
    step_12();
    step_13();
}
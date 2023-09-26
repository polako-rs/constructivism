## Introduction

`constructivism` is a Rust sample-library designed to simplify the construction of structured data by defining and manipulating sequences of Constructs. This README provides an overview of how to use `constructivism` and how it can be inlined into you project using `constructivist` library.

## Installation

To use Constructivism in your Rust project, add it as a dependency in your `Cargo.toml` file:

```toml
[dependencies]
constructivism = "0.0.1"
```

Or let the cargo do the stuff:

```bash
cargo add constructivism
```

Constructivism can be inlined into you library as for example `yourlibrary_constructivism` within `constructivist` crate. See [instructions](./crates/constructivist).

## Guide

See also [examples/tutorial.rs](examples/tutorial.rs)

### Getting Started

Usually you start with

```rust
use constructivism::*;
```

### Constructs and Sequences

<a name="1-1">1.1</a>. **Constructs**: Constructivism revolves around the concept of Constructs. A Construct can be declared only in front of another Construct. `constructivism` comes with only Nothing construct. You can define new Constructs like this:

```rust
#[derive(Construct)]
#[construct(Node -> Nothing)]
pub struct Node {
    #[default(false)]       // You can provide custom default values.
    hidden: bool,
    position: (f32, f32),   // Or Default::default() will be used.
}
```

<a name="1-2">1.2</a> **`construct!`**: You can use the `construct!` macro to create instances of Constructs. Please ***note*** the dots at the beginning of the each param, they are required and you will find this syntax quite useful.

```rust
fn create_node() {
    let node = construct!(Node {
        .position: (10., 10.),
        .hidden: true
    });
    assert_eq!(node.position.0, 10.);
    assert_eq!(node.hidden, true);
}
```

<a name="1-3">1.3</a> **Skipping Fields**: You can skip the declaration of non-required fields. You can also use true-aliases: single `field` is an alias for `field: true`.

```rust
fn create_another_node() {
    let node = construct!(Node {
        .hidden
    });
    assert_eq!(node.position.0, 0.);
}
```

<a name="1-4">1.4</a> **Required Fields**: Non-default fields must be marked with `#[required]` to avoid compilation errors.



```rust
#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Entity;

#[derive(Construct)]
#[construct(Reference -> Nothing)]
pub struct Reference {
    #[required]
    target: Entity,
    count: usize,
}
```

<a name="1-5">1.5</a> **Passing Required Fields**: You must pass all required fields to `construct!(..)` to avoid compilation errors.

```rust
fn create_reference() {
    let reference = construct!(Reference {
        .target: Entity
    });
    assert_eq!(reference.target, Entity);
    assert_eq!(reference.count, 0);
}
```

<a name="1-6">1.6</a> **Sequences**: Constructs are organized in Sequences. For example, `Node -> Nothing` is a sequence. You define a local sequence to the next Construct like this:

```rust
#[derive(Construct)]
#[construct(Rect -> Node)]
pub struct Rect {
    #[default((100., 100.))]
    size: (f32, f32)
}
```

<a name="1-7">1.7</a> **Using Sequences**: The Sequence for Rect becomes `Rect -> Node -> Nothing` in example above. You can `construct!` the entire sequence within a single call:

```rust
fn create_sequence() {
    let (rect, node, /* nothing */) = construct!(Rect {
        .position: (10., 10.),
        .size: (10., 10.),
        // You can skip fields, no `hidden` here, for example
    });
    assert_eq!(rect.size.0, 10.);
    assert_eq!(node.position.1, 10.);
    assert_eq!(node.hidden, false);
}
```

### Design and Methods

<a name="2-1">2.1</a> **Designs and Methods**: Every Construct has its own Design. You can implement methods for a Construct's design:

```rust
impl NodeDesign {
    pub fn move_to(&self, entity: Entity, position: (f32, f32)) { }
}

impl RectDesign {
    pub fn expand_to(&self, entity: Entity, size: (f32, f32)) { }
}
```

<a name="2-2">2.2</a> **Calling Methods**: You can call methods on a Construct's design. Method resolution follows the sequence order:

```rust
fn use_design() {
    let rect_entity = Entity;
    design!(Rect).expand_to(rect_entity, (10., 10.));
    design!(Rect).move_to(rect_entity, (10., 10.)); // move_to implemented for NodeDesign
}
```

### Segments

<a name="3-1">3.1</a> **Segments**: Segments allow you to define and insert segments into a Construct's sequence:

```rust
#[derive(Segment)]
pub struct Input {
    disabled: bool
}

#[derive(Construct)]
#[construct(Button -> Input -> Rect)]
pub struct Button {
    pressed: bool
}
```

<a name="3-2">3.2</a> **Sequence with Segments**: The Sequence for Button becomes `Button -> Input -> Rect -> Node -> Nothing`. You can instance the entire sequence of a Construct containing segments within a single `construct!` call:

```rust
fn create_button() {
    let (button, input, rect, node) = construct!(Button {
        .disabled: true
    });
    assert_eq!(button.pressed, false);
    assert_eq!(input.disabled, true);
    assert_eq!(rect.size.0, 100.);
    assert_eq!(node.position.0, 0.);
}
```

<a name="3-3">3.3</a> **Segment Design**: Segment has its own Design as well. And the method call resolves within the Sequence order as well. Segment's designes has one generic parameter - the next segment/construct, so you have to respect it when implement Segment's Design:

```rust
impl<T> InputDesign<T> {
    fn focus(&self, entity: Entity) {
        /* do the focus stuff */
    }
}

fn focus_button() {
    let btn = Entity;
    design!(Button).focus(btn);
}
```

### Props

<a name="4-1">4.1</a>  **Props**: By deriving Constructs or Segments you also get the ability to set and get properties on items with respect of Sequence:

```rust
fn button_props() {
    let (mut button, mut input, mut rect, mut node) = construct!(Button {});
    
    // You can access to props knowing only the top-level Construct
    let pos         /* Prop<Node, (f32, f32)> */    = prop!(Button.position);
    let size        /* Prop<Rect, (f32, f32)> */    = prop!(Button.size);
    let disabled    /* Prop<Input, bool> */         = prop!(Button.disabled);
    let pressed     /* Prop<Button, bool */         = prop!(Button.pressed);

    // You can read props. You have to pass exact item to the get()
    let x = pos.get(&node).as_ref().0;
    let w = size.get(&rect).as_ref().0;
    let is_disabled = *disabled.get(&input).as_ref();
    let is_pressed = *pressed.get(&button).as_ref();
    assert_eq!(0., x);
    assert_eq!(100., w);
    assert_eq!(false, is_disabled);
    assert_eq!(false, is_pressed);

    // You can set props. You have to pass exact item to set()
    pos.set(&mut node, (1., 1.));
    size.set(&mut rect, (10., 10.));
    disabled.set(&mut input, true);
    pressed.set(&mut button, true);
    assert_eq!(node.position.0, 1.);
    assert_eq!(rect.size.0, 10.);
    assert_eq!(input.disabled, true);
    assert_eq!(button.pressed, true);

}
```

<a name="4-2">4.2</a> **Expand props**: If you have field with Construct type, you can access this fields props as well:

```rust
#[derive(Construct, Default)]
#[construct(Vec2 -> Nothing)]
pub struct Vec2 {
    x: f32,
    y: f32,
}

#[derive(Construct)]
#[construct(Node2d -> Nothing)] 
pub struct Node2d {
    #[prop(construct)]      // You have to mark expandable props with #[prop(construct)]
    position: Vec2
}

fn modify_position_x() {
    let mut node = construct!(Node2d {});
    assert_eq!(node.position.x, 0.);
    assert_eq!(node.position.y, 0.);

    let x = prop!(Node2d.position.x);

    x.set(&mut node, 100.);
    assert_eq!(node.position.x, 100.);
    assert_eq!(node.position.y, 0.);
}
```

### Custom Constructors

<a name="5-1">5.1</a> **Custom Constructors**: Sometimes you may want to implement Construct for a foreign type or provide a custom constructor. You can use `derive_construct!` for this purpose:

```rust
pub struct ProgressBar {
    min: f32,
    val: f32,
    max: f32,
}


impl ProgressBar {
    pub fn min(&self) -> f32 {
        self.min
    }
    pub fn set_min(&mut self, min: f32) {
        self.min = min;
        if self.max < min {
            self.max = min;
        }
        if self.val < min {
            self.val = min;
        }
    }
    pub fn max(&self) -> f32 {
        self.max
    }
    pub fn set_max(&mut self, max: f32) {
        self.max = max;
        if self.min > max {
            self.min = max;
        }
        if self.val > max {
            self.val = max;
        }
    }
    pub fn val(&self) -> f32 {
        self.val
    }
    pub fn set_val(&mut self, val: f32) {
        self.val = val.max(self.min).min(self.max)
    }
}

derive_construct! {
    // Sequence
    seq => ProgressBar -> Rect;

    // Constructor, all params with defult values
    construct => (min: f32 = 0., max: f32 = 1., val: f32 = 0.) -> {
        if max < min {
            max = min;
        }
        val = val.min(max).max(min);
        Self { min, val, max }
    };

    // Props using getters and setters
    props => {
        min: f32 = [min, set_min];
        max: f32 = [max, set_max];
        val: f32 = [val, set_val];
    };
}
```

<a name="5-2">5.2</a> **Using Custom Constructors**: The provided constructor will be called when creating instances:

```rust
fn create_progress_bar() {
    let (pb, _, _) = construct!(ProgressBar { .val: 100. });
    assert_eq!(pb.min, 0.);
    assert_eq!(pb.max, 1.);
    assert_eq!(pb.val, 1.);
}
```

<a name="5-3">5.3</a> **Custom Construct Props**: In the example above `derive_construct!` declares props using getters and setters. This setters and getters are called when you use `Prop::get` and `Prop::set`

```rust
fn modify_progress_bar() {
    let (mut pb, _, _) = construct!(ProgressBar {});
    let min = prop!(ProgressBar.min);
    let val = prop!(ProgressBar.val);
    let max = prop!(ProgressBar.max);

    assert_eq!(pb.val, 0.);

    val.set(&mut pb, 2.);
    assert_eq!(pb.val, 1.0);    //becouse default for max = 1.0

    min.set(&mut pb, 5.);
    max.set(&mut pb, 10.);
    assert_eq!(pb.min, 5.);
    assert_eq!(pb.val, 5.);
    assert_eq!(pb.max, 10.);
}
```

<a name="5-4">5.4</a> **Deriving Segments**: You can derive Segments in a similar way:

```rust
pub struct Range {
    min: f32,
    max: f32,
    val: f32,
}
derive_segment!{
    // use `seg` to provide type you want to derive Segment
    seg => Range;
    construct => (min: f32 = 0., max: f32 = 1., val: f32 = 0.) -> {
        if max < min {
            max = min;
    }
        val = val.min(max).max(min);
        Self { min, val, max }
    };
    // Props using fields directly
    props => {
        min: f32 = value;
        max: f32 = value;
        val: f32 = value;
    };
}

#[derive(Construct)]
#[construct(Slider -> Range -> Rect)]
pub struct Slider;

fn create_slider() {
    let (slider, range, _, _) = construct!(Slider {
        .val: 10.
    });
    assert_eq!(range.min, 0.0);
    assert_eq!(range.max, 1.0);
    assert_eq!(range.val, 1.0);
}
```


## Limitations

- only public structs (or enums with `constructable!`)
- no generics supported yet (looks very possible)
- limited number of params for the whole inheritance tree (default version compiles with 16, tested with 64)
- only static structs/enums (no lifetimes)

## Cost

I didn't perform any stress-tests. It should run pretty fast: there is no heap allocations, only some deref calls per `construct!` per defined prop per depth level. Cold compilation time grows with number of params limit (1.5 mins for 64), but the size of the binary doesn't changes.


## Roadmap

- [ ] add `#![forbid(missing_docs)]` to the root of each crate
- [ ] docstring bypassing
- [ ] generics
- [ ] union params, so you can pas only one param from group. For example, Range could have `min`, `max`, `abs` and `rel` constructor params, and you can't pass `abs` and `rel` both.
- [ ] nested construct inference (looks like possible):
```rust
#[derive(Construct, Default)]
pub struct Vec2 {
    x: f32,
    y: f32
}
#[derive(Construct)]
pub struct Div {
    position: Vec2,
    size: Vec2,
}
fn step_inference() {
    let div = construct!(Div {
        position: {{ x: 23., y: 20. }},
        size: {{ x: 23., y: 20. }}
    })
}
```

## Contributing

I welcome contributions to Constructivism! If you'd like to contribute or have any questions, please feel free to open an issue or submit a pull request.

## License

The `constructivism` is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

This means you can select the license you prefer!
This dual-licensing approach is the de-facto standard in the Rust ecosystem and there are [very good reasons](https://github.com/bevyengine/bevy/issues/2373) to include both.
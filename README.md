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

Constructivism can be inlined into you library as for example `yourlibrary_constructivism` within `constructivist` crate. 

> TODO: Write instruction and/or link example projects

## Theory by practice

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
    #[default(true)]        // You can provide custom default values.
    visible: bool,
    position: (f32, f32),   // Or Default::default() will be used.
}
```

<a name="1-2">1.2</a> **`construct!`**: You can use the `construct!` macro to create instances of Constructs:

```rust
fn create_node() {
    let node = construct!(Node {
        position: (10., 10.),
        visible: true
    });
    assert_eq!(node.position.0, 10.);
    assert_eq!(node.visible, true);
}
```

<a name="1-3">1.3</a> **Skipping Fields**: You can skip the declaration of non-required fields. You can also use bool-aliases:
- `field` is an alias for `field: true`
- `!field` is and alias for `field: false`

```rust
fn create_node() {
    let node = construct!(Node {
        !visible
    });
    assert_eq!(node.position.0, 0.);
}
```

<a name="1-4">1.4</a> **Required Fields**: Non-default fields must be marked with `#[required]` to avoid compilation errors.

```rust
#[derive(Construct)]
#[construct(Reference -> Nothing)]
struct Reference {
    #[required]
    target: Entity,
    count: usize,
}
```

<a name="1-5">1.5</a> **Passing Required Fields**: You must pass all required fields to `construct!(..)` to avoid compilation errors.

```rust
fn create_reference() {
    let reference = construct!(Reference {
        target: Entity(23)
    });
    assert_eq!(reference.target.0, 23);
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
        position: (10., 10.),
        size: (10., 10.),
        // You can skip fields, no `visible` here, for example
    });
    assert_eq!(rect.size.0, 10.);
    assert_eq!(node.position.1, 10.);
    assert_eq!(node.visible, true);
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
    let rect_entity = Entity(20);
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
    let (button, input, rect) = construct!(Button {
        disabled: true
    });
    assert_eq!(button.pressed, false);
    assert_eq!(input.disabled, true);
    assert_eq!(rect.size.0, 100.);
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
    let btn = Entity(42);
    design!(Button).focus(btn);
}
```

### Custom Constructors

<a name="4-1">4.1</a> **Custom Constructors**: Sometimes you may want to implement Construct for a foreign type or provide a custom constructor. You can use `derive_construct!` for this purpose:

```rust
pub struct ProgressBar {
    min: f32,
    val: f32,
    max: f32,
}

derive_construct! {
    ProgressBar -> Rect

    // Params with default values
    (min: f32 = 0., max: f32 = 1., val: f32 = 0.)
    
    // Custom constructor
    {
        if max < min {
            max = min;
        }
        val = val.min(max).max(min);
        Self { min, val, max }
    }
}
```

<a name="4-2">4.2</a> **Using Custom Constructors**: The provided constructor will be called when creating instances:

```rust
fn create_progress_bar() {
    let (range, _, _) = construct!(ProgressBar { val: 100. });
    assert_eq!(range.min, 0.);
    assert_eq!(range.max, 1.);
    assert_eq!(range.val, 1.);
}
```

<a name="4-3">4.3</a> **Deriving Segments**: You can derive Segments in a similar way:

```rust
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

fn create_slider() {
    let (_slider, range, _, _) = construct!(Slider {
        val: 10.
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

The `constructivism_macro` is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

This means you can select the license you prefer!
This dual-licensing approach is the de-facto standard in the Rust ecosystem and there are [very good reasons](https://github.com/bevyengine/bevy/issues/2373) to include both.
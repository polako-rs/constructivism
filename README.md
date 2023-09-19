### Definition

This is theory & practice section that describes `constructivim`. You can see complete example [here](./examples/tutorial.rs). You can start with 

```rust
use constructivism::*;
```

1. `constructivism` constist of `Construct`s. `Construct` can be declared only in front of the another `Construct`. `constructivism` provide only `Nothing` construct. To define new `Constrcut` you can:

```rust
#[derive(Construct)]
#[construct(Node -> Nothing)]
pub struct Node {
    // You can provide custom default values.
    #[default(true)]
    visible: bool,
    position: (f32, f32),
}
```

2. You can `construct!` the `Construct`:
```rust
fn def_02() {
    let node = construct!(Node {
        position: (10., 10.),
        visible: true
    });
    assert_eq!(node.position.0, 10.);
    assert_eq!(node.visible, true);
}
```

3. You can skip declaration of default fields
```rust
fn def_03() {
    let node = construct!(Node {
        visible: false
    });
    assert_eq!(node.position.0, 0.)
}
```

4. You have to mark non-default required fields with `#[required]` or you get compilation error.
```rust
pub struct Entity(usize);

#[derive(Construct)]
#[construct(Reference -> Nothing)]
struct Reference {
    #[required]
    target: Entity,
    count: usize,
}
```

5. You have to pass all required fiels to `construct!(..)` or you get compilation error
```rust
fn def_05() {
    let reference = construct!(Reference {
        target: Entity(23)
    });
    assert_eq!(reference.target.0, 23);
    assert_eq!(reference.count, 0);
}
```

6. The `Construct`'s relation (`Node -> Nothing`) called `Sequence` in constructivism. You define only local `Sequence` to the next `Construct`:
```rust
#[derive(Construct)]
#[construct(Rect -> Node)]
pub struct Rect {
    #[default((100., 100.))]
    size: (f32, f32)
}
```

7. The `Sequence` for `Rect` becomes `Rect -> Node -> Nothing`. The unwrapped `#[derive(Construct)]` for Rect looks something like this:
```rust
impl Construct for Rect {
    type Sequence = (Rect, Node, Nothing)
    /* other Construct types and methods */
}
```

8. You can `construct!` the whole `Sequence` within the single call:
```rust
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
```

9. Every construct have it's own `Design`. You can implement methods for construct's design:

```rust
impl NodeDesign {
    pub fn move_to(&self, entity: Entity, position: (f32, f32)) { }
}
impl RectDesign {
    pub fn expand_to(&self, entity: Entity, size: (f32, f32)) { }
}
```

10. You can call methods on construct's design. Method resolves within the `Sequence` order:
```rust
fn def_10() {
    let rect_entity = Entity(20);
    design!(Rect).expand_to(rect_entity, (10., 10.));
    design!(Rect).move_to(rect_entity, (10., 10.));
}
```

11. You can define and insert `Segment` into construct's `Sequence`:
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

12. The `Sequence` for `Button` becomes `Button -> Input -> Rect -> Node -> Nothing`. You still can `construct!` the whole `Sequence` within the single call:
```rust
fn def_12() {
    let (button, input, rect, node) = construct!(Button {
        disabled: true
    });
    assert_eq!(button.pressed, false);
    assert_eq!(input.disabled, true);
    assert_eq!(rect.size.0, 100.);
    assert_eq!(node.position.0, 0.);
}
```

13. `Segment` has its own `Design` as well. And the method call resolves within the `Sequence` order as well:
```rust
impl InputDesign {
    fn focus(&self, entity: Entity) { }
}

fn def_13() {
    let btn = Entity(42);
    design!(Button).focus(entity);
}
```


### Upcoming features

- docstring bypassing

- `mixable! { ... }`, just like `constructable! { ... }`

- union props

- generics (generic structs not supported yet)

- nested results, like 
```rust
let (radio, base) = construct!(*Radio { ... });
```

- expose constructivism_macro as macro-library, so it can be injected into your own namespace

- doc-based bindgen for third-parti libraries (not right now)

- nested construct inference (looks like possible):
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

### Limitations:
- only public structs (or enums with `constructable!`)
- no generics supported yet (looks very possible)
- limited number of params for the whole inheritance tree (current version compiles with 16, tested with 64)
- only static structs/enums (no lifetimes)

### Cost:
I didn't perform any stress-tests. It should run pretty fast: there is no heap allocations, only some deref calls per `construct!` per defined prop per depth level. Cold compilation time grows with number of params limit (1.5 mins for 64), but the size of the binary doesn't changes.

> TODO: Provide stress testing and results

### License

The `constructivism_macro` is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

This means you can select the license you prefer!
This dual-licensing approach is the de-facto standard in the Rust ecosystem and there are [very good reasons](https://github.com/bevyengine/bevy/issues/2373) to include both.
#!#[rustfmt::skip]
#![allow(unused_variables)]

use std::marker::PhantomData;

// You've got to start somewhere
use constructivism::*;

// ### Constructs and Sequences

// 1.1  **Constructs**: Constructivism revolves around the concept of Constructs.

#[derive(Construct)]
pub struct Node {
    hidden: bool,
    position: (f32, f32),
}

pub struct TypeRef<C: Construct + 'static>(PhantomData<C>);

pub fn construct_node<F, P,  const I: u8>(func: F) -> <<Node as Construct>::NestedSequence as Flattern>::Output
where
    P: ExtractParams<
        I, <Node as Construct>::MixedParams,
        Value = <<Node as Construct>::MixedParams as Extractable>::Output,
        Rest = <<<<Node as Construct>::Base as Construct>::ExpandedParams as Extractable>::Input as AsParams>::Defined
    >,
    F: FnOnce(
        &<Node as Construct>::Params,
        <<<Node as Construct>::ExpandedParams as Extractable>::Input as AsParams>::Undefined
    ) -> P
{
    construct_inferred::<Node, F, P, I>(func)
}

// 1.2  **Constructing**: You can use the `construct!` macro to create instances of Constructs.
//      Please ***note*** the dots at the beginning of the each param, they are required and you
//      will find this syntax quite useful.
fn construct_inferred<C: Construct + 'static, F, P, const I: u8>(func: F) -> <C::NestedSequence as Flattern>::Output
where
    P: ExtractParams<
        I, C::MixedParams,
        Value = <C::MixedParams as Extractable>::Output,
        Rest = <<<C::Base as Construct>::ExpandedParams as Extractable>::Input as AsParams>::Defined
    >,
    F: FnOnce(
        &<C as Construct>::Params,
        <<<C as Construct>::ExpandedParams as Extractable>::Input as AsParams>::Undefined
    ) -> P
{
    let fields = <<C as Construct>::Params as Singleton>::instance();
    let params = <<C as Construct>::ExpandedParams as Extractable>::as_params();
    let defined = func(fields, params);
    <C as Construct>::construct(defined).flattern()
}

fn create_node() {
    let node = construct_node(|fields, params| {
        let param: &::constructivism::Param<_, _> = &fields.position;
        let field = param.field();
        let value = params
            .field(&field)
            .define(param.value(((10., 10.)).into()));
        let params = params + value;
        let param: &::constructivism::Param<_, _> = &fields.hidden;
        let field = param.field();
        let value = params.field(&field).define(param.value((true).into()));
        let params = params + value;
        params.defined()
    });
    // let node =
    // // construct!(Node {
    // //     .position: (10., 10.),
    // //     .hidden: true,
    // // });
    // {
    //     use ::constructivism::traits::*;
    //     let fields =  <<Node as ::constructivism::Construct> ::Params as ::constructivism::Singleton> ::instance();
    //     let params =  <<Node as ::constructivism::Construct> ::ExpandedParams as ::constructivism::Extractable> ::as_params();
    //     let param: &::constructivism::Param<_, _> = &fields.position;
    //     let field = param.field();
    //     let value = params
    //         .field(&field)
    //         .define(param.value(((10., 10.)).into()));
    //     let params = params + value;
    //     let param: &::constructivism::Param<_, _> = &fields.hidden;
    //     let field = param.field();
    //     let value = params.field(&field).define(param.value((true).into()));
    //     let params = params + value;
    //     let defined_params = params.defined();
    //     <Node as ::constructivism::Construct>::construct(defined_params).flattern()
    // };
    assert_eq!(node.position.0, 10.);
    assert_eq!(node.hidden, true);
}

// 1.3  **Sequences**: A Construct can be declared only in front of another Construct.
//      `constructivism` comes with only Nothing, () construct. The `Self -> Base` relation
//      called Sequence in `constructivism`. You can omit the Sequence declaration,
//      `Self -> Nothing` used in this case. If you want to derive Construct on the top of another
//      meaningful Construct, you have to specify Sequence directly using
//      `#[construct(/* Sequence */)]` attribute.

#[derive(Construct)]
#[construct(Rect -> Node)]
pub struct Rect {
    size: (f32, f32),
}

// 1.4  **Constructing Sequences**: The Sequence for the Rect  in example above becomes
//      `Rect -> Node -> Nothing`. You can `construct!` the entire sequence within a single call:

fn create_sequence() {
    let (rect, node /* nothing */) = construct!(Rect {
        .hidden,                        // You can write just `.hidden` instead of `.hidden: true`
        .position: (10., 10.),
        .size: (10., 10.),
    });
    assert_eq!(rect.size.0, 10.);
    assert_eq!(node.position.1, 10.);
    assert_eq!(node.hidden, true);
}

// 1.5  **Params**: There are different kind of Params (the things you passing to `construct!(..)`):
//      - Common: use `Default::default()` if not passed to `construct!(..)`
//      - Default: use provided value if not passed to `construct!(..)`
//      - Required: must be passed to `construct!(..)`
//      - Skip: can't be passed to `construct!(..)`, use Default::default() or provided value
//      You configure behavior using `#[param]` attribute when deriving:

#[derive(Construct)]
#[construct(Follow -> Node)]
pub struct Follow {
    offset: (f32, f32), // Common, no #[param]

    #[param(required)] // Required
    target: Entity,

    #[param(default = Anchor::Center)] // Default
    anchor: Anchor,

    #[param(skip)] // Skip with Default::default()
    last_computed_distance: f32,

    #[param(skip = FollowState::None)] // Skip with provided value
    state: FollowState,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Entity;

pub enum Anchor {
    Left,
    Center,
    Right,
}

pub enum FollowState {
    None,
    Initialized(f32),
}

// 1.6  **Passing params**: When passing params to `construct!(..)` you have to pass all required
//      for Sequence params, or you will get the compilation error. You can omit non-required params.

fn create_elements() {
    // omit everything, default param values will be used
    let (rect, node /* nothing */) = construct!(Rect);
    assert_eq!(node.hidden, false);
    assert_eq!(rect.size.0, 0.);

    // you have to pass target to Follow, the rest can be omitted..
    let (follow, node) = construct!(Follow {
        .target: Entity
    });
    assert_eq!(follow.offset.0, 0.);
    assert_eq!(node.hidden, false);

    // ..or specified:
    let (follow, node) = construct!(Follow {
        .hidden,
        .target: Entity,
        .offset: (10., 10.),

        // last_computed_distance param is skipped, uncommenting
        // the next line will result in compilation error
        // error: no field `last_computed_distance` on type `&follow_construct::Params`

        // .last_computed_distance: 10.
    });
    assert_eq!(follow.offset.0, 10.);
    assert_eq!(node.hidden, true);
}

// ### Design and Methods

// 2.1  **Designs and Methods**: Every Construct has its own Design. You can implement methods for
//      a Construct's design:
impl NodeDesign {
    pub fn move_to(&self, entity: Entity, position: (f32, f32)) {}
}

impl RectDesign {
    pub fn expand_to(&self, entity: Entity, size: (f32, f32)) {}
}

// 2.2  **Calling Methods**: You can call methods on a Construct's design. Method resolution
//      follows the sequence order:
fn use_design() {
    let rect_entity = Entity;
    design!(Rect).expand_to(rect_entity, (10., 10.));
    design!(Rect).move_to(rect_entity, (10., 10.)); // move_to implemented for NodeDesign
}

// ### Segments

// 3.1  **Segments**: Segments allow you to define and insert segments into a Construct's
//      sequence:
#[derive(Segment)]
pub struct Input {
    disabled: bool,
}

#[derive(Construct)]
#[construct(Button -> Input -> Rect)]
pub struct Button {
    pressed: bool,
}

// 3.2  **Sequence with Segments**: The Sequence for Button becomes
//      `Button -> Input -> Rect -> Node -> Nothing`. You can instance the entire sequence of a
//      Construct containing segments within a single `construct!` call:
fn create_button() {
    let (button, input, rect, node) = construct!(Button {
        .disabled: true,
    });
    assert_eq!(button.pressed, false);
    assert_eq!(input.disabled, true);
    assert_eq!(rect.size.0, 0.);
    assert_eq!(node.position.0, 0.);
}

// 3.3  **Segment Design**: Segment has its own Design as well. And the method call resolves
//      within the Sequence order as well. Segment's designs has one generic parameter - the next
//      segment/construct, so you have to respect it when implement Segment's Design:
impl<T> InputDesign<T> {
    fn focus(&self, entity: Entity) {
        /* do the focus stuff */
    }
}

fn focus_button() {
    let btn = Entity;
    design!(Button).focus(btn);
}

// ### Props

// 4.1  **Props**: By deriving Constructs or Segments you also get the ability to set and get
//      properties on items with respect of Sequence:

fn button_props() {
    let (mut button, mut input, mut rect, mut node) = construct!(Button);

    // You can access to props knowing only the top-level Construct
    // Prop<Node, (f32, f32)>
    let pos = prop!(Button.position);
    // Prop<Rect, (f32, f32)>
    let size = prop!(Button.size);
    // Prop<Input, bool>
    let disabled = prop!(Button.disabled);
    // Prop<Button, bool>
    let pressed = prop!(Button.pressed);

    // You can read props. You have to pass exact item to the get()
    let x = pos.get(&node).as_ref().0;
    let w = size.get(&rect).as_ref().0;
    let is_disabled = *disabled.get(&input).as_ref();
    let is_pressed = *pressed.get(&button).as_ref();
    assert_eq!(0., x);
    assert_eq!(0., w);
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

// 4.2 **Expand props**: If you have field with Construct type, you can access this fields props as well:

#[derive(Construct, Default)]
pub struct Vec2 {
    x: f32,
    y: f32,
}

#[derive(Construct)]
pub struct Node2d {
    #[prop(construct)] // You have to mark expandable props with #[prop(construct)]
    position: Vec2,
}

fn modify_position_x() {
    let mut node = construct!(Node2d);
    assert_eq!(node.position.x, 0.);
    assert_eq!(node.position.y, 0.);

    // Prop<Node2d, f32>
    let x = prop!(Node2d.position.x);

    x.set(&mut node, 100.);
    assert_eq!(node.position.x, 100.);
    assert_eq!(node.position.y, 0.);
}

// ### Custom Constructors

// 5.1  **Custom Constructors**: Sometimes you may want to derive Construct for a foreign type
//      or provide a custom constructor/props for your type. You can use `derive_construct!` for
//      this purpose:
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

    // Constructor, all params with default values
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

// 5.2  **Using Custom Constructors**: The provided constructor will be called when creating
//      instances:
fn create_progress_bar() {
    let (pb, _, _) = construct!(ProgressBar { .val: 100. });
    assert_eq!(pb.min, 0.);
    assert_eq!(pb.max, 1.);
    assert_eq!(pb.val, 1.);
}

// 5.3  **Custom Construct Props**: In the example above `derive_construct!` declares props using
//      getters and setters. This setters and getters are called when you use `Prop::get`
//      and `Prop::set`
fn modify_progress_bar() {
    let (mut pb, _, _) = construct!(ProgressBar);
    let min = prop!(ProgressBar.min);
    let val = prop!(ProgressBar.val);
    let max = prop!(ProgressBar.max);

    assert_eq!(pb.val, 0.);

    val.set(&mut pb, 2.);
    assert_eq!(pb.val, 1.0); //because default for max = 1.0

    min.set(&mut pb, 5.);
    max.set(&mut pb, 10.);
    assert_eq!(pb.min, 5.);
    assert_eq!(pb.val, 5.);
    assert_eq!(pb.max, 10.);
}

// 5.3  **Deriving Segments**: You can derive Segments in a similar way:
pub struct Range {
    min: f32,
    max: f32,
    val: f32,
}
derive_segment! {
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

fn main() {
    create_node();
    create_elements();
    create_sequence();
    use_design();
    create_button();
    button_props();
    modify_position_x();
    focus_button();
    create_progress_bar();
    modify_progress_bar();
    create_slider();
}

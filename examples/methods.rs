use std::marker::PhantomData;

use constructivist::*;
use constructivist_core::Construct;
use constructivist_core::Methods;
use constructivist_core::MutableMethod;
pub struct World;

pub struct Entity(usize);

#[derive(Construct)]
pub struct Node {
    position: (f32, f32),
}

impl Node {
    pub fn spawn_child(world: &mut World, this: Entity) -> Entity {
        Entity(this.0 + 1)
    }

    pub fn translate(&mut self, tr: (f32, f32)) {
        self.position.0 += tr.0;
        self.position.1 += tr.1;
    }
}

pub trait NodeMethods {
    fn spawn_child(&self) -> StaticMethod<Node, (&mut World, Entity), Entity>;
    fn translate(&self) -> MutableMethod<Node, ((f32, f32),), ()>;
}

impl<M: Methods<Node>> NodeMethods for M {
    fn spawn_child(&self) -> StaticMethod<Node, (&mut World, Entity), Entity> {
        StaticMethod(PhantomData, |(world, this)| Node::spawn_child(world, this))
    }
    fn translate(&self) -> MutableMethod<Node, ((f32, f32),), ()> {
        MutableMethod(|this, (tr,)| this.translate(tr))
    }
}

#[derive(Construct)]
#[extends(Node)]
pub struct Rect {
    size: (f32, f32),
}

impl Rect {
    pub fn grow(&mut self, add: (f32, f32)) {
        self.size.0 += add.0;
        self.size.1 += add.1;
    }
}

pub trait RectMethods {
    fn grow(&self) -> MutableMethod<Rect, ((f32, f32),), ()>;
}

impl<M: Methods<Rect>> RectMethods for M {
    fn grow(&self) -> MutableMethod<Rect, ((f32, f32),), ()> {
        MutableMethod(|this, (add,)| this.grow(add))
    }
}

fn main() {
    let (mut rect, mut node) = construct!(Rect { size: (10., 10.) });

    let methods = <<Rect as Construct>::Methods as Singleton>::instance();
    let _ = methods.spawn_child().call((&mut World, Entity(23)));
    methods.translate().call(&mut node, ((1., 1.),));
    methods.grow().call(&mut rect, ((1., 1.),));
}

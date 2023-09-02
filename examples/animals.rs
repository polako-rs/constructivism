use std::marker::PhantomData;

use constructivist_core::construct;
use constructivist_core::constructall;
use constructivist_core::new;
use constructivist_core;

use constructivist_macro_support::Construct;


#[derive(Default)]
struct Name(String);
#[derive(Default)]
struct Age(usize);
#[derive(Default)]
struct Field<T>(PhantomData<T>);
#[derive(Default)]
struct Fields {
    name: Field<Name>,
    age: Field<Age>,
}

impl Fields {
    pub fn instance() -> &'static Fields {
        &Fields {
            name: Field(PhantomData),
            age: Field(PhantomData),
        }
    }
}

pub struct Content<T>(T);
pub trait IntoContent<T> {
    fn into_content(self) -> Content<T>;
}

#[derive(Construct)]
pub struct Div {

}

impl IntoContent<Entity> for Div {
    fn into_content(self) -> Content<Entity> {
        Content(Entity(23))
    }
}

impl IntoContent<Entity> for (Div, ()) {
    fn into_content(self) -> Content<Entity> {
        Content(Entity(23))
    }
}

impl<T, C: IntoContent<Entity>> IntoContent<Entity> for (T, C) {
    fn into_content(self) -> Content<Entity> {
        self.1.into_content()
    }
}


#[derive(Construct)]
#[wraps(Div)]
pub struct Slider {

}

fn t() {
    let obj = (Slider { }, (Div { }, ()));
    let e = obj.into_content();
}






#[derive(Construct)]
pub struct Animal {
    #[required]
    name: String,
}

#[derive(Construct)]
#[wraps(Animal)]
pub struct Duck {
    #[default(1)]
    volume: u8,
}

pub struct Entity(usize);

#[derive(Default)]
pub struct Vec2 { pub x: f32, pub y: f32 }

#[derive(Construct)]
#[wraps(Duck)]
pub struct Follow {
    #[required]
    target: Entity,
    offset: Vec2,
}



fn main() {
    let (follow, base) = new!(Follow {
        name: "bob",
        target: Entity(20),        
    });

    let (duck, animal) = constructall!(Duck {
        name: "bob",
        volume: 22,
    });
    assert_eq!(animal.name, "bob".to_string());
    assert_eq!(duck.volume, 22);


    let (follow, duck, animal) = constructall!(Follow {
        target: Entity(20),
        name: "Bill",
    });
    assert_eq!(follow.target.0, 20);
    assert_eq!(follow.offset.x, 0.);
    assert_eq!(follow.offset.y, 0.);
    assert_eq!(duck.volume, 1);
    assert_eq!(animal.name, "Bill".to_string());

    let follow = construct!(Follow {
        target: Entity(10),
    });
    assert_eq!(follow.target.0, 10);
}
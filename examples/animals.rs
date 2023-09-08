use constructivist::*;

use constructivist_macro_support::Construct;

#[derive(Construct)]
pub struct Animal {
    #[required]
    name: String,
}

#[derive(Construct)]
#[extends(Animal)]
pub struct Duck {
    #[default(1)]
    volume: u8,
}

pub struct Entity(usize);

#[derive(Default)]
pub struct Vec2 { pub x: f32, pub y: f32 }

#[derive(Construct)]
#[extends(Duck)]
pub struct Follow {
    #[required]
    target: Entity,
    offset: Vec2,
}



fn main() {
    let (duck, animal) = construct!(Duck {
        name: "bob",
        volume: 22,
    });
    assert_eq!(animal.name, "bob".to_string());
    assert_eq!(duck.volume, 22);


    let (follow, duck, animal) = construct!(Follow {
        target: Entity(20),
        name: "Bill",
    });
    assert_eq!(follow.target.0, 20);
    assert_eq!(follow.offset.x, 0.);
    assert_eq!(follow.offset.y, 0.);
    assert_eq!(duck.volume, 1);
    assert_eq!(animal.name, "Bill".to_string());

    // TODO: provide better compilation errro, name is required here:
    // let (follow, _, _) = construct!(Follow {
    //     target: Entity(10),
    // });
    // assert_eq!(follow.target.0, 10);
}
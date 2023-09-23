use constructivism::{*, Prop};

use std::marker::PhantomData;


fn main() {

}

macro_rules! get {
    ($root:ident $(.$item:ident)+) => {
        $root.getters()$(.$item())+.unwrap()
    };
}

#[derive(Construct, Default)]
#[construct(Color -> Nothing)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

#[derive(Construct)]
#[construct(Div -> Nothing)]
pub struct Div {
    #[prop(construct)]
    background: Color
}


#[derive(Construct)]
#[construct(Label -> Div)]
pub struct Label {
    #[prop(construct)]
    text_color: Color
}

impl Label {
    
}


pub struct Pr<Host, Target> {
    get: fn(&Host) -> Value<Target>,
    set: fn(&mut Host, Target),
}

struct Model<T: Construct>(PhantomData<T>);

// impl<T: Construct> Model<T> {
//     fn props(&self) {
//         let props = <<T as Construct>::Props as Singleton>::instance();


//     }
// }

fn t() {

    // let text_color_r = Pr {
    //     get: |c| <<Label as Construct>::Props as Singleton>::instance()
    //         .text_color_getters(c)
    //         .text_color()
    //         .r(),
    //     set: |c, val| <<Label as Construct>::Props as Singleton>::instance()
    //         .text_color_setters(c)
    //         .text_color()
    //         .set_r(val)
    // };
    // let background_r = Pr {
    //     get: |c| <<Label as Construct>::Props as Singleton>::instance()
    //         .background_getters(c)
    //         .background()
    //         .r(),
    //     set: |c, val| <<Label as Construct>::Props as Singleton>::instance()
    //         .background_setters(c)
    //         .background()
    //         .set_r(val)
    // };
    let _text_color_r = prop!(Label.text_color.r);
    let _background_r = prop!(Label.background.a);
    let _text_color = prop!(Label.text_color);
    let _background = prop!(Label.background);
    // let 
    // let c = construct
    // let l = Label {
    //     text_color: Color { r: 1. }
    // };
    // let v = get!(l.text_color.r);
    // let g = LabelGetters(&l);
    // let v = g.text_color().r().unwrap();

    // let p = Prop {
    //     get: |c: &Label| { c.getters().text_color().r().unwrap() },
    // //     get: |c: &Color| { c.r_value() },
    //     set: |c: &mut Label, value| { }
    // };
}
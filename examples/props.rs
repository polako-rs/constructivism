use constructivism::*;


fn main() {

}

#[derive(Default)]
#[derive(Construct)]
#[construct(Color -> Nothing)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    #[prop(a, set_a)]
    a: f32,
}

impl Color {
    pub fn a(&self) -> f32 {
        self.a
    }
    pub fn set_a(&mut self, a: f32) {
        self.a = a
    }
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
// impl<T: Construct> Model<T> {
//     fn props(&self) {
//         let props = <<T as Construct>::Props as Singleton>::instance();


//     }
// }

fn _t() {
    let _p = <<Label as Construct>::Props<Lookup> as Singleton>::instance();
    
    // let _x = prop!(Label.background.);
    let _text_color_r = prop!(Label.background.r);
    let _background_r = prop!(Label.background.a);
    let _text_color = prop!(Label.text_color);
    let _background = prop!(Label.background);
}
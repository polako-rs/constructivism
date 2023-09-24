use constructivism::*;


fn main() {

}

#[derive(Default)]
#[derive(Construct)]
#[construct(Color -> Nothing)]
pub struct Color {
    /// Red channel
    r: f32,
    /// Green channel
    g: f32,
    /// Blue channel
    b: f32,
    /// Alpha channel
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

    /// The background color for the Div, transparent by default.
    #[prop(construct)]
    background: Color
}


#[derive(Construct)]
#[construct(Label -> Div)]
pub struct Label {
    /// The text coklor, black by default.
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
    // let prop = prop!(Label.text_color.r);
}
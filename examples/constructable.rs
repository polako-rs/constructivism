use constructivist_macro_support::{Construct, constructable};

#[allow(dead_code)]
#[derive(Construct)]
pub struct Div {
    width: f32,
    height: f32
}



#[allow(dead_code)]
pub struct Slider {
    min: f32,
    max: f32,
    val: f32,
}

constructable! { Slider extends Div (
    min: f32 = 0.,
    max: f32 = 1.,
    val: f32 = 0.,
){
    if max < min {
        max = min;
    }
    val = val.min(max).max(min);
    Self { min, max, val }
}}


fn main() {
    
}
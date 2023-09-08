use constructivist_core::{new, Join, AsFlatProps};
use constructivist_macro_support::{Construct, constructable};
use constructivist_core::traits::*;

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
    let x = <Join<(slider_construct::min, slider_construct::max, slider_construct::val), ()>>::as_props();
    let y = <Join<(div_construct::width, div_construct::height), ()>>::as_props();
    let a = <(div_construct::width, div_construct::height) as AsFlatProps>::as_flat_props();
    let b = <(slider_construct::min, slider_construct::max, slider_construct::val) as AsFlatProps>::as_flat_props();
    let z = <Join<
        (slider_construct::min, slider_construct::max, slider_construct::val),
        (div_construct::width, div_construct::height)
    >>::as_flat_props();
    

    let _ = new!(Slider {
        width: 23.,
        max: 10.,
        val: 5.,
        // uncomment next line to get 'expected function, found `PropRedefined<val>`'
        // val: 5.,
    });
}
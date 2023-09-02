use std::marker::PhantomData;

use constructivist_core::new;
use constructivist_macro_support::{Construct, constructable};

#[derive(Construct)]
pub struct Div {
    width: f32,
    height: f32
}

pub struct Slider {
    min: f32,
    max: f32,
    val: f32,
}

constructable! { Slider extends Div (
    min: f32 = 0.,
    max: f32 = 1.,
    val: f32 = 0.
){
    if max < min {
        max = min;
    }
    val = val.min(max).max(min);
    Self { min, max, val }
}}

struct UnionPropsConflict<N, T>(PhantomData<(N, T)>);
impl<N, T> UnionPropsConflict<N, T> {
    fn validator(self) -> Self {
        self
    }
}
mod fields {
 pub struct abs;
 pub struct rel;

}

fn main() {
    let name = UnionPropsConflict::<fields::abs, fields::rel>(PhantomData);
    // name.validator()();
    let x = new!(Slider {
        width: 23.,
        max: 10.,
        val: 5.,
        // val: 5.,
    });


}
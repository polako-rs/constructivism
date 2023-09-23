pub mod proc;
pub mod context;
pub mod derive;
mod exts;
pub mod genlib;
pub mod throw;

pub mod prelude {
    pub use constructivism_macro_gen::implement_constructivism_macro;
    pub use crate::proc::{Construct, Prop};
    pub use crate::context::Context;
    pub use crate::derive::{DeriveConstruct, DeriveSegment};
    pub use crate::genlib;
}

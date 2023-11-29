pub mod context;
pub mod derive;
pub mod exts;
pub mod genlib;
pub mod proc;
pub mod throw;

pub mod prelude {
    pub use crate::context::Context;
    pub use crate::derive::{DeriveConstruct, DeriveSegment};
    pub use crate::genlib;
    pub use crate::proc::{Construct, Prop, Ref};
    pub use constructivism_macro_gen::implement_constructivism_macro;
}

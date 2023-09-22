## About

`constructivist` is the way to deliver `constructivism` into your crate.

## The Problem

At some point you will realise that `construtivism` is fucken awesome. And you want to use it in your crate. Let's call this crate `polako`, just for example. And it also could happen that your crate is about to work with another crate (or framework, or engine), let's say it is about to work with `bevy` (for example of course). You will meet one irresistible problem in this case: you can't implement a foreign trait for a foreign type. You can't implement `constructivism::Construct` for `bevy::Sprite` from your thouthend-times-amazing `polako` crate.

This is how you become a `constructivist`.

`constructivis`*`t`* allows you to inline the `constructivis`*`m`* into your crate. But. You are supposed to complete some steps.

## The Solution

I assume that you are using workspace and your crates live inside the `crates` folder. There is also required naming convention with a single constraint: if your main crate is called `awesomecrate`, then your constructivism crate have to be called `awesomecrate_constructivism`. You can inspect how the real `polako` inlines constructivism here: [https://github.com/jkb0o/polako/tree/eml](https://github.com/jkb0o/polako/tree/eml)

#### Step 1: Create your macro crate

1. You have to create (or use existed) your own macro crate to implement your version of `constructivism_macro`. This is how you can create your `polako_macro` crate:

```bash
# in your favorite terminal:
cd crates
mkdir polako_macro
cd polako_macro
cargo init --lib
cargo add syn proc_macro2 quote constructivist
```

2. If it is new macro crate, you have to edit `Cargo.toml` of this crate, and make this crate `proc_macro` crate:
```toml
# crates/polako_macro/Cargo.toml
[lib]
proc-macro = true
```

3. You have to inline `constructivism_macro` into your crate:
```rust
// crates/polako_macro/src/lib.rs
implement_constructivism_macro!("polako");
```

4. You can use all the power of `constructivism_macro` in your crate.

#### Step 2: Create your constructivism crate

1. You need to inline all traits and implementations of `constructivism` in your crate:

```bash
# we are in constructivism_macro dir for now, go back to crates
cd ../
mkdir polako_constructivism
cargo init --lib
cargo add --path ../polako_macro

# you most probably want to have your third-parti crate as dependency
cargo add bevy
```

2. At this point, you can use constructivism derives and proc macros in you crate. It meansyou can implement constructivism now:

```rust
// crates/polako_constructivism/src/lib.rs
pub use polako_macro::*;
// 32 is the maximum params limit, see (TODO: link the explanation)
implement_constructivism!(32);
```

#### Step 3: Add bindings to all your needs

1. From now, you are working with crate that defines & implements constructivism structs & traits. An this is the point where you can bridge the third-party crate. `bevy` in our example. Add the bridge mod to your crate:

```rust
// crates/polako_constructivism/src/lib.rs
pub use polako_macro::*;
implement_constructivism!(32);

// add bridge mod:
mod bridge
```

2. Provide implementations for the third-party crate:

```rust
// crates/polako_constructivism/src/bridge.rs
use bevy::prelude::*;
use polaco_macro::*;

derive_construct! {
    NodeBundle -> Nothing () {
        NodeBundle::default()
    }
}
```

#### Step 4: Add constructivism mod to your crate root

1. As you already guessed, the name of the root crate in this tutorial is `polako`. So, you HAVE to add `constructivism` mod to your root crate to make it all work everywhere:

```rust
// src/lib.rs
pub mod constructivism {
    // this is required:
    pub use polako_constructivism::*;
    pub use polako_macro::*;

    // this is optional (but nice):
    pub mod prelude {
        pub use polako_constructivism::prelude::*;
        pub use polako_macro::*;
    }
}
```

#### Step 5: Give the Feedback

1. All the stuff you done won't compile in most cases (becouse I've tested only single case). 
2. Go to github and write an [issue](https://github.com/jkb0o/polako/issues) about how life is hard without constructivism and cry about this damn tutorial that just won't work as expected.
3. (Optional) Find the source of the problem and provide a *beautiful* PR.

#### Step 6: Overcome the Suffering

1. You followed this tutorial and implemented constructivism more then once.
2. You wonder - why there is no tools that automate all of these steps?
3. You realize - it is becouse nobody wrote these tools yet.
4. You implement `cargo-bootstrap-constructivism` (ask me how), provide astonished PR, and make this crate even better.

## License (boring)

The `constructivist` is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

This means you can select the license you prefer!
This dual-licensing approach is the de-facto standard in the Rust ecosystem and there are [very good reasons](https://github.com/bevyengine/bevy/issues/2373) to include both.
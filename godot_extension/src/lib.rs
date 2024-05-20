use godot::prelude::*;

struct GdExtension;

#[gdextension]
unsafe impl ExtensionLibrary for GdExtension {}

pub mod macro_solver_interface;

use crate::shared::{PtrMagic, module::{ModuleCallback, ModuleVariable}};

/// A Class in the PixelScript runtime.
/// 
/// A class will compile directly to whatever the target language expects.
/// 
/// In LUA - A tree
/// In Python/JS/easyjs - A class
/// 
/// Works similar to a module, in the sense that the caller never needs to know the builtin structure.
/// Just keeps a pointer alive until you are ready to add it to the runtime.
/// 
/// example:
/// ```C
/// void* class_ptr = pixelscript_new_class("Math");
/// pixelscript_class_add_callbacl("add", add_wrapper);
/// pixelscript_class_add_variable("x", var_obj);
/// pixelscript_add_class(class_ptr);
/// // Now the pointer is no longer valid because the runtime owns it.
/// ```
/// 
/// Depending on intentions, a class is better than a module if you need it in global / __main__ context.
/// A module is better for memory and optomization. So choose a class if you need it globally.
/// Choose a module if it is optional. And combine the both as you please.
/// 
/// Although main difference between a class and a module is that a class is ideally created using raw literals and then executed.
pub struct Class {
    /// The name of the class.
    pub name: String,
    /// Callbacks inside the class
    pub callbacks: Vec<ModuleCallback>,
    /// Variables inside the class
    pub vars: Vec<ModuleVariable>,
}

impl PtrMagic for Class {}

unsafe impl Send for Class {}
unsafe impl Sync for Class {}
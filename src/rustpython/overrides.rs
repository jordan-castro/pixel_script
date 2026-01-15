// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
// Methods to override builtins:

use rustpython_vm::{VirtualMachine, scope::Scope};

use crate::{shared::read_file};

/// Add the _pixelscript_load_pymodule function
fn add_pixelscript_load_pymodule(vm: &VirtualMachine) {
    // let pyfunc = vm.new_function(
    //     "_pixelscript_load_pymodule",
    //     |name: String, vm: &VirtualMachine| {
    //         println!("Trying to load func {name}");
    //         // Add .py just in case.
    //         let name = if !name.ends_with(".py") {
    //             format!("{name}.py").to_string()
    //         } else {
    //             name
    //         };
    //         let contents = read_file(name.as_str());
    //         if contents.len() == 0 {
    //             vm.ctx.none()
    //         } else {
    //             vm.ctx.new_str(contents).into()
    //         }
    //     },
    // );
    // vm.builtins
    //         .set_attr("_pixelscript_load_pymodule", pyfunc, vm)
    //         .expect("Could not reach builtins");
}

/// Makes it possible to import user written modules.
pub(super) fn override_import_loader(vm: &VirtualMachine) {
//     add_pixelscript_load_pymodule(vm);

// // 1. Define the PixelFinder and a Mock Spec class
//     let setup_code = r#"
// import sys
// import types

// class MockSpec:
//     def __init__(self, name, loader, origin):
//         self.name = name
//         self.loader = loader
//         self.origin = origin
//         self.submodule_search_locations = None 
//         self.has_location = True
//         self.cached = None
//         self._initializing = False

// class PixelFinder:
//     @classmethod
//     def find_spec(cls, fullname, path=None, target=None):
//         # Trigger the Rust helper
//         source = _pixelscript_load_pymodule(fullname)
//         if source is None:
//             return None
        
//         # Return our mock spec since importlib.machinery.ModuleSpec is missing
//         return MockSpec(fullname, cls, f"{fullname}.py")

//     @classmethod
//     def create_module(cls, spec):
//         # Returning None tells the VM to create a default empty module
//         return None

//     @classmethod
//     def exec_module(cls, module):
//         source = _pixelscript_load_pymodule(module.__name__)
//         if source:
//             code = compile(source, module.__name__ + ".py", 'exec')
//             # Essential: set the __file__ attribute manually
//             module.__file__ = module.__name__ + ".py"
//             exec(code, module.__dict__)
// "#;

//     // 2. Run the definition in a persistent system scope
//     let system_scope = vm.new_scope_with_builtins();
//     vm.run_code_string(system_scope.clone(), setup_code, "<pixelscript_internal>".to_owned())
//         .expect("Failed to define PixelFinder internally");

//     // 3. FORCE into sys.meta_path using the Rust List API
//     let sys_module = vm.sys_module.clone();
//     let meta_path = sys_module.get_attr("meta_path", vm).unwrap();
//     let pixel_finder = system_scope.locals.get_item("PixelFinder", vm).unwrap();

//     vm.call_method(&meta_path, "insert", vec![
//         vm.ctx.new_int(0).into(),
//         pixel_finder
//     ]).expect("Could not insert pixelfinder");

//     println!("Success: PixelFinder injected at index 0 of sys.meta_path");

}

/// Helper to inject a bundled .py file into the VM's module cache
fn inject_bundled_module(vm: &VirtualMachine, name: &str, code: &str) {
    // let module_dict = vm.ctx.new_dict();
    // let module = vm.new_module(name, module_dict.clone(), None);
    // // Set package attributes so relative imports (from . import ...) work
    // if name.contains('.') || name == "collections" {
    //     let parent = if name == "collections" {
    //         name.to_string()
    //     } else {
    //         name.split('.').next().unwrap().to_string()
    //     };
    //     // Tell Python this is a package
    //     module_dict
    //         .set_item("__package__", vm.ctx.new_str(parent).into(), vm)
    //         .unwrap();
    //     module_dict
    //         .set_item(
    //             "__path__",
    //             vm.ctx
    //                 .new_list(vec![vm.ctx.new_str("virtual").into()])
    //                 .into(),
    //             vm,
    //         )
    //         .unwrap();
    // }

    // let scope = Scope::with_builtins(None, module_dict, vm);
    // // Insert into sys modules first
    // let sys_modules = vm.sys_module.get_attr("modules", vm).unwrap();
    // sys_modules.set_item(name, module.into(), vm).unwrap();
    // // Now run
    // if let Err(err) = vm.run_code_string(scope, code, format!("<bundled_{}>", name)) {
    //     // This will print the ACTUAL Python error (like ModuleNotFoundError)
    //     vm.print_exception(err);
    //     panic!("Failed to initialize bundled module: {}", name);
    // }
}

/// Add the lib/rustpython folder
pub(super) fn add_bundled_stdlib(vm: &VirtualMachine) {
//     // let dict = scope.clone().downcast::<rustpython::vm::builtins::PyDict>().expect("Could not downcast to Dict, Python.");
//     // let scope = Scope::with_builtins(None, dict, vm);

//     // Get each file under lib/rustpython. Also get /collections
//     // encodings
//     inject_bundled_module(vm, "encodings", "def search_function(name): return None");
//     inject_bundled_module(vm, "types", include_str!("../../lib/rustpython/types.py"));
//     inject_bundled_module(
//         vm,
//         "_weakrefset",
//         include_str!("../../lib/rustpython/_weakrefset.py"),
//     );

//     // Add reprlib
//     inject_bundled_module(
//         vm,
//         "reprlib",
//         include_str!("../../lib/rustpython/reprlib.py"),
//     );

//     // Essentials next
//     // _py_abc before abc
//     inject_bundled_module(
//         vm,
//         "_py_abc",
//         include_str!("../../lib/rustpython/_py_abc.py"),
//     );
//     // abc before _collections_abc
//     inject_bundled_module(vm, "abc", include_str!("../../lib/rustpython/abc.py"));
//     inject_bundled_module(
//         vm,
//         "_collections_abc",
//         include_str!("../../lib/rustpython/_collections_abc.py"),
//     );
//     inject_bundled_module(
//         vm,
//         "_sitebuiltins",
//         include_str!("../../lib/rustpython/_sitebuiltins.py"),
//     );

//     // Keyword
//     inject_bundled_module(
//         vm,
//         "keyword",
//         include_str!("../../lib/rustpython/keyword.py"),
//     );
//     inject_bundled_module(
//         vm,
//         "operator",
//         include_str!("../../lib/rustpython/operator.py"),
//     );

//     // collections
//     inject_bundled_module(
//         vm,
//         "collections._defaultdict",
//         include_str!("../../lib/rustpython/collections/_defaultdict.py"),
//     );

//     inject_bundled_module(
//         vm,
//         "collections",
//         include_str!("../../lib/rustpython/collections/__init__.py"),
//     );
//     inject_bundled_module(
//         vm,
//         "collections.abc",
//         include_str!("../../lib/rustpython/collections/abc.py"),
//     );

//     // Collections fix
//     vm.run_code_string(
//         vm.new_scope_with_builtins(),
//         r#"
// import sys, collections
// collections._defaultdict = sys.modules['collections._defaultdict']
// collections.abc = sys.modules['collections.abc']
//     "#,
//         "<collections_fix>".to_owned(),
//     )
//     .unwrap();

//     // Modules for modders
//     // functools before enum
//     inject_bundled_module(
//         vm,
//         "functools",
//         include_str!("../../lib/rustpython/functools.py"),
//     );
//     inject_bundled_module(vm, "enum", include_str!("../../lib/rustpython/enum.py"));
//     inject_bundled_module(vm, "typing", include_str!("../../lib/rustpython/typing.py"));

//     // Path/IO logic
//     inject_bundled_module(vm, "io", include_str!("../../lib/rustpython/io.py"));
//     inject_bundled_module(vm, "stat", include_str!("../../lib/rustpython/stat.py"));

//     println!("Finished overrides");
}

#[cfg(test)]
mod tests {
    use std::{ffi::c_void, ptr, sync::Arc};

    use pixel_script::{python::PythonScripting, shared::{PixelScript, PtrMagic, func::get_function_lookup, object::PixelObject, var::Var}, *
    };

    struct Person {
        name: String,
    }

    impl Person {
        pub fn new(n_name: String) -> Self {
            Person { name: n_name }
        }

        pub fn set_name(&mut self, n_name: String) {
            self.name = n_name;
        }

        pub fn get_name(&self) -> String {
            self.name.clone()
        }
    }

    impl PtrMagic for Person {}

    pub extern "C" fn free_person(ptr: *mut c_void) {
        let _ = unsafe { Person::from_borrow(ptr as *mut Person) };
    }

    pub extern "C" fn set_name(argc: usize, argv: *mut *mut Var, _opaque: *mut c_void) -> *mut Var {
        unsafe {
            let args = Var::slice_raw(argv, argc);
            // Get ptr
            let pixel_object_var = Var::from_borrow(args[1]);
            let host_ptr = pixel_object_var.get_host_ptr();
            let p = Person::from_borrow(host_ptr as *mut Person);

            // Check if first arg is self or nme
            let name = {
            let first_arg = Var::from_borrow(args[2]);
            if first_arg.is_string() {
                first_arg
            } else {
                Var::from_borrow(args[3])
            }
            };

            p.set_name(name.get_string().unwrap().clone());

            Var::into_raw(Var::new_null())
        }
    }

    pub extern "C" fn get_name(argc: usize, argv: *mut *mut Var, _opaque: *mut c_void) -> *mut Var {
        unsafe {
            let args = Var::slice_raw(argv, argc);
            // Get ptr
            let pixel_object_var = Var::from_borrow(args[1]);
            let host_ptr = pixel_object_var.get_host_ptr();
            let p = Person::from_borrow(host_ptr as *mut Person);

            Var::new_string(p.get_name().clone()).into_raw()
        }
    }

    pub extern "C" fn new_person(
        argc: usize,
        argv: *mut *mut Var,
        opaque: *mut c_void,
    ) -> *mut Var {
        unsafe {
            let args = std::slice::from_raw_parts(argv, argc);
            let p_name = Var::from_borrow(args[1]);
            let p_name = p_name.get_string().unwrap();
            let p = Person::new(p_name.clone());

            let ptr = Person::into_raw(p) as *mut c_void;
            let mut pixel_object = PixelObject::new(ptr, free_person, "PersonT");
            
            // Save callbacks
            let mut function_lookup = get_function_lookup();
            let set_name_idx = function_lookup.add_function("set_name", set_name, opaque);
            let get_name_idx = function_lookup.add_function("get_name", get_name, opaque);

            pixel_object.add_callback("set_name", "", set_name_idx);
            pixel_object.add_callback("get_name", "", get_name_idx);

            // Save...
            let var = pixelscript_var_newhost_object(pixel_object.into_raw());

            var
        }
    }

    // Testing callbacks
    pub extern "C" fn print_wrapper(
        argc: usize,
        argv: *mut *mut Var,
        _opaque: *mut c_void,
    ) -> *mut Var {
        unsafe {
            let args = std::slice::from_raw_parts(argv, argc);

            let var_ptr = Var::from_borrow(args[0]);

            if let Ok(msg) = var_ptr.get_string() {
                println!("Lua sent: {}", msg);
            }
        }

        Var::new_null().into_raw()
    }

    pub extern "C" fn add_wrapper(
        argc: usize,
        argv: *mut *mut Var,
        _opaque: *mut c_void,
    ) -> *mut Var {
        // Assumes n1 and n2
        unsafe {
            let args = std::slice::from_raw_parts(argv, argc);

            let n1 = Var::from_borrow(args[0]);
            let n2 = Var::from_borrow(args[1]);

            Var::new_i64(n1.value.i64_val + n2.value.i64_val).into_raw()
        }
    }

    #[test]
    fn test_add_variable() {
        PythonScripting::add_variable("name", &Var::new_string(String::from("Jordan")));
    }

    #[test]
    fn test_add_callback() {
        // Create fn idx
        let mut function_lookup = get_function_lookup();
        let idx = function_lookup.add_function("println", print_wrapper, ptr::null_mut());
        PythonScripting::add_callback("println", idx);
    }

    #[test]
    fn test_add_module() {
        let mut module = shared::module::Module::new("cmath".to_string());

        // Save methods
        let mut function_lookup = get_function_lookup();
        let add_idx = function_lookup.add_function("m_add", add_wrapper, ptr::null_mut());

        module.add_callback("add", "", add_idx);
        module.add_variable("n1", &Var::new_i64(1));
        module.add_variable("n2", &Var::new_i64(2));

        let module_arc = Arc::new(module);

        PythonScripting::add_module(Arc::clone(&module_arc));
    }

    #[test]
    fn test_add_object() {
        let mut function_lookup = get_function_lookup();
        let person_idx = function_lookup.add_function("Person", new_person, ptr::null_mut());
        PythonScripting::add_callback("Person", person_idx);
    }

    #[test]
    fn test_execute() {
        pixelscript_initialize();

        test_add_variable();
        test_add_callback();
        test_add_module();
        test_add_object();

        let py_code = r#"
import cmath

msg = "Welcome " + name
println(msg)

result = cmath.add(cmath.n1, cmath.n2)
println(f"Module result: {result}")

if result != 3:
    raise "Math, Expected 3, got " + str(result)

person = Person("Jordan")
println(person.get_name())
person.set_name("Jordan Castro")
println(person.get_name())
        "#;
        let err = PythonScripting::execute(py_code, "<test>");
        assert!(err.is_empty(), "Python Error is not empty: {}", err);

        pixelscript_finalize();
    }
}

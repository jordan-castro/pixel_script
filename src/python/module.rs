use std::sync::Arc;

use rustpython::vm::{PyObjectRef, VirtualMachine, convert::ToPyObject};

use crate::{python::create_function, shared::module::Module};

fn create_internal_module(vm: &VirtualMachine, module: &Module, parent_path: Option<&str>) -> PyObjectRef {
    let m_dict = vm.ctx.new_dict();
    let module_name = match parent_path{
        Some(path) => format!("{}.{}", path, module.name),
        None => module.name.clone(),
    };

    m_dict.set_item("__name__", vm.ctx.new_str(module_name.clone()).into(), vm).unwrap();
    m_dict.set_item("__package__", vm.ctx.none(), vm).unwrap();
    m_dict.set_item("__loader__", vm.ctx.none(), vm).unwrap();
    m_dict.set_item("__spec__", vm.ctx.none(), vm).unwrap();

    // Variables
    for variable in module.variables.iter() {
        m_dict.set_item(&variable.name, variable.var.clone().to_pyobject(vm), vm).expect("Could not set a variable in Python module.");
    }

    // Callbacks
    for callback in module.callbacks.iter() {
        let func = create_function(vm, &callback.name, callback.idx);
        m_dict.set_item(&callback.name, func, vm).expect("Can not define method in Python module.");
    }

    // Inner modules
    let modules = vm.sys_module.get_attr("modules", vm).expect("Could not get inner sys Modules Python");
    for inner_m in module.modules.iter() {
        let m = create_internal_module(vm, inner_m, Some(&module_name));
        let m_name = format!("{}.{}", module_name, inner_m.name.clone());
        // Set in sys modules
        modules.set_item(m_name.as_str(), m.clone(), vm).expect("Could not add internal Python module");
        m_dict.set_item(&inner_m.name, m, vm).expect("Can not define inner module in Python module.");
    }

    vm.new_module(&module.name, m_dict.into(), vm.ctx.new_str("").into()).into()
}

/// Create a Python module.
pub(super) fn create_module(vm: &VirtualMachine, module: Arc<Module>) {
    let m = create_internal_module(vm, module.as_ref(), None);
    let sys_modules = vm.sys_module.get_attr("modules", vm).expect("Could not get Sys Modules Python.");
    sys_modules.set_item(&module.name, m, vm).expect("Could not add Python module.");
}
use std::sync::Arc;

use rustpython::vm::{PyObjectRef, VirtualMachine, convert::ToPyObject};

use crate::{python::create_function, shared::module::Module};

fn create_internal_module(vm: &VirtualMachine, module: &Module, parent_path: Option<&str>) -> PyObjectRef {
    let m_dict = vm.ctx.new_dict();
    let module_name = match parent_path{
        Some(path) => format!("{}.{}", path, module.name),
        None => module.name.clone(),
    }  ;

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
    for inner_m in module.modules.iter() {
        let m = create_internal_module(vm, inner_m, Some(&module_name));
        let m_name = format!("{}.{}", module_name, inner_m.name.clone());
        // Set in sys modules
        let pystr = vm.ctx.new_str(m_name);
        vm.sys_module.set_attr(&pystr, m.clone(), vm).expect("Could not Add internal Python module.");

        m_dict.set_item(&inner_m.name, m, vm).expect("Can not define inner module in Python module.");
    }

    vm.new_module(&module.name, m_dict.into(), vm.ctx.new_str("").into()).into()
}

/// Create a Python module.
pub(super) fn create_module(vm: &VirtualMachine, module: Arc<Module>) {
    let m = create_internal_module(vm, module.as_ref(), None);
    let m_str = vm.ctx.new_str(module.name.clone());
    vm.sys_module.set_attr(&m_str, m, vm).expect("Could not add Python module");
}
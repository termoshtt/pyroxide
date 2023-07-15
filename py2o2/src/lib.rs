pub mod codegen;
pub mod inspect;
pub mod parser;

use anyhow::Result;
use inspect::get_inspect_json;

pub fn generate(python_module_name: &str, bare: bool) -> Result<String> {
    let interface = inspect::Interface::from_py_module(python_module_name)?;
    let generated = codegen::generate(python_module_name, &interface, bare)?;
    Ok(generated)
}

pub fn inspect(python_module_name: &str) -> Result<String> {
    get_inspect_json(python_module_name)
}

#![allow(dead_code, unused_imports)]

use anyhow::Result;
use py2o2_runtime::Enum2;
use pyo3::{prelude::*, types::*};

pub mod example;
pub mod type_aliases;

const PYTHON_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../python/");

#[test]
fn example() -> Result<()> {
    std::env::set_var("PYTHONPATH", PYTHON_ROOT);

    Python::with_gil(|py| {
        // No return value
        example::a1(py)?;
        example::a2(py, 57)?;
        example::a3(py, "homhom", 3.0)?;

        // With return values
        dbg!(example::a4(py)?);
        dbg!(example::a5(py, 33)?);
        dbg!(example::a6(py)?);
        dbg!(example::a7(py, 112)?);
        Ok(())
    })
}

#[test]
fn type_aliases() -> Result<()> {
    std::env::set_var("PYTHONPATH", PYTHON_ROOT);

    Python::with_gil(|py| {
        let out = type_aliases::scale(py, 2.0, PyList::new(py, [1.0, 2.0, 3.0]))?;
        dbg!(out);

        let id = type_aliases::UserId(124);
        let out = type_aliases::get_user_name(py, id)?;
        assert_eq!(out.to_str()?, "ID = 124");

        Ok(())
    })
}

pub trait OneOf: IntoPy<PyObject> {}

impl OneOf for i64 {}
impl OneOf for &'_ str {}

fn union_f_new<'py>(py: Python<'py>, a: impl OneOf) -> PyResult<Enum2<i64, String>> {
    Ok(py
        .import("union")?
        .getattr("f_new")?
        .call((a.into_py(py),), None)?
        .extract()?)
}

#[test]
fn union() -> Result<()> {
    std::env::set_var("PYTHONPATH", PYTHON_ROOT);
    Python::with_gil(|py| {
        let out = union_f_new(py, 42)?;
        assert_eq!(out, Enum2::Item1(42));

        let out = union_f_new(py, "homhom")?;
        assert_eq!(out, Enum2::Item2("homhom".to_string()));
        Ok(())
    })
}

pub use pyo3;

use pyo3::{
    conversion::{FromPyObject, IntoPy},
    exceptions::PyTypeError,
    types::PyType,
    Py, PyAny, PyErr, PyResult, Python,
};
use std::{any::TypeId, collections::BTreeMap};

#[derive(Debug, PartialEq, Clone)]
pub enum Enum2<T1, T2> {
    Item1(T1),
    Item2(T2),
}

static mut TYPE_MAPPING: BTreeMap<TypeId, Py<PyType>> = BTreeMap::new();

fn get_py_type<'py, T>(py: Python<'py>) -> &'py PyType
where
    T: 'static + IntoPy<Py<PyAny>> + Default,
{
    // This must be called from the thread which has Python's GIL.
    unsafe { TYPE_MAPPING.entry(TypeId::of::<T>()) }
        .or_insert_with(|| {
            let value = T::default().into_py(py);
            value.as_ref(py).get_type().extract().unwrap()
        })
        .as_ref(py)
}

impl<'s, T1, T2> FromPyObject<'s> for Enum2<T1, T2>
where
    T1: 'static + FromPyObject<'s> + IntoPy<Py<PyAny>> + Default,
    T2: 'static + FromPyObject<'s> + IntoPy<Py<PyAny>> + Default,
{
    fn extract(ob: &'s PyAny) -> PyResult<Self> {
        let ty = ob.get_type();
        Python::with_gil(|py| {
            if ty.is(get_py_type::<T1>(py)) {
                return Ok(Enum2::Item1(T1::extract(ob)?));
            }
            if ty.is(get_py_type::<T2>(py)) {
                return Ok(Enum2::Item2(T2::extract(ob)?));
            }
            Err(PyErr::from_value(
                PyTypeError::new_err(format!(
                    "None of {:?} or {:?}",
                    TypeId::of::<T1>(),
                    TypeId::of::<T2>()
                ))
                .value(py),
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn convert() -> Result<()> {
        Python::with_gil(|py| -> Result<()> {
            let v1: i32 = 42;
            let v2: f32 = 2.123;

            let p1: Py<PyAny> = v1.into_py(py);
            let e: Enum2<i32, f32> = p1.extract(py)?;
            assert_eq!(e, Enum2::Item1(v1));

            let p2: Py<PyAny> = v2.into_py(py);
            let e: Enum2<i32, f32> = p2.extract(py)?;
            assert_eq!(e, Enum2::Item2(v2));

            let p3: Py<PyAny> = "test".into_py(py);
            assert!(p3.extract::<Enum2<i32, f32>>(py).is_err());

            Ok(())
        })?;
        Ok(())
    }
}

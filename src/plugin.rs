use color_eyre::eyre::Result;
use pyo3::{types::PyModule, Py, PyAny, PyResult, Python};
use std::{fs::read_to_string, path::Path, sync::Arc};

#[derive(Debug, Clone)]
pub struct Plugin {
    py_impl: Py<PyAny>,
}

impl Plugin {
    pub fn load(path: impl AsRef<Path>, config: impl AsRef<str>) -> Result<Self> {
        let code = read_to_string(path)?;
        let py_impl = Python::with_gil(|py| -> PyResult<Py<PyAny>> {
            Ok(PyModule::from_code(py, &code, "", "")?
                .getattr("Plugin")?
                .call1((config.as_ref(),))?
                .into())
        })?;

        Ok(Self { py_impl })
    }

    pub fn exec(&self, data: &[u8]) -> Result<()> {
        Python::with_gil(|py| -> PyResult<()> {
            self.py_impl.getattr(py, "exec")?.call1(py, (data,))?;

            Ok(())
        })?;

        Ok(())
    }
}

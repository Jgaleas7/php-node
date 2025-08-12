use std::path::{Path, PathBuf};

use lang_handler::{Handler, Request, Response, ResponseBuilder};
use pyo3::prelude::*;

/// Embed a Python script environment to handle HTTP requests.
pub struct Embed {
  docroot: PathBuf,
}

impl Embed {
  /// Create a new Python embed with the given document root.
  pub fn new<C: AsRef<Path>>(docroot: C) -> Self {
    Self {
      docroot: docroot.as_ref().to_path_buf(),
    }
  }
}

impl Handler for Embed {
  type Error = String;

  fn handle(&self, request: Request) -> Result<Response, Self::Error> {
    // Determine the path to the python script relative to docroot
    let path = self.docroot.join(&request.url().path()[1..]);
    let code = std::fs::read_to_string(&path)
      .map_err(|e| format!("failed to read script: {e}"))?;

    Python::with_gil(|py| -> PyResult<Response> {
      let sys = py.import("sys")?;
      let io = py.import("io")?;
      let buffer: Py<PyAny> = io.getattr("StringIO")?.call0()?.into();
      sys.setattr("stdout", buffer.bind(py))?;
      py.run(&code, None, None)?;
      let output: String = buffer.bind(py).call_method0("getvalue")?.extract()?;
      let resp = ResponseBuilder::new().status(200).body(output.as_bytes()).build();
      Ok(resp)
    }).map_err(|e| e.to_string())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use lang_handler::{MockRootBuilder, RequestBuilder};

  #[test]
  fn runs_python_script() {
    let docroot = MockRootBuilder::default()
      .file("hello.py", "print('Hello, Python!')")
      .build()
      .unwrap();
    let embed = Embed::new(&*docroot);
    let request = RequestBuilder::new()
      .method("GET")
      .url("http://localhost/hello.py")
      .build()
      .unwrap();
    let response = embed.handle(request).unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.body(), "Hello, Python!\n");
  }
}

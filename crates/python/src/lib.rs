use std::path::{Component, Path, PathBuf};

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
    let url_path = request.url().path();
    let req_path = Path::new(url_path);

    if req_path
      .components()
      .any(|c| matches!(c, Component::ParentDir))
    {
      return Err("path traversal attempt".to_string());
    }

    let rel_path = req_path.strip_prefix("/").unwrap_or(req_path);

    let docroot = self
      .docroot
      .canonicalize()
      .map_err(|e| format!("failed to access docroot: {e}"))?;

    let full_path = docroot.join(rel_path);
    let full_path = full_path
      .canonicalize()
      .map_err(|e| format!("failed to read script: {e}"))?;

    if !full_path.starts_with(&docroot) {
      return Err("path traversal attempt".to_string());
    }

    let mut code =
      std::fs::read_to_string(&full_path).map_err(|e| format!("failed to read script: {e}"))?;
    if !code.ends_with('\n') {
      code.push('\n');
    }

    Python::with_gil(|py| -> PyResult<Response> {
      let sys = py.import_bound("sys")?;
      let io = py.import_bound("io")?;
      let buffer: Py<PyAny> = io.getattr("StringIO")?.call0()?.into();
      let old_stdout: Py<PyAny> = sys.getattr("stdout")?.into();
      sys.setattr("stdout", buffer.bind(py))?;
      let run_result = py.run_bound(&code, None, None);
      let output_result: PyResult<String> = buffer
        .bind(py)
        .call_method0("getvalue")
        .and_then(|o| o.extract());
      sys.setattr("stdout", old_stdout.bind(py))?;
      run_result?;
      let output = output_result?;
      let resp = ResponseBuilder::new()
        .status(200)
        .body(output.as_bytes())
        .build();
      Ok(resp)
    })
    .map_err(|e| e.to_string())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use lang_handler::{MockRootBuilder, RequestBuilder};
  use pyo3::Python;

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

  #[test]
  fn multiple_invocations_do_not_interfere() {
    let docroot = MockRootBuilder::default()
      .file("hi.py", "print('hi')")
      .build()
      .unwrap();
    let embed = Embed::new(&*docroot);
    let request = RequestBuilder::new()
      .method("GET")
      .url("http://localhost/hi.py")
      .build()
      .unwrap();
    let response1 = embed.handle(request.clone()).unwrap();
    let response2 = embed.handle(request).unwrap();
    assert_eq!(response1.body(), "hi\n");
    assert_eq!(response2.body(), "hi\n");
    Python::with_gil(|py| {
      let sys = py.import_bound("sys").unwrap();
      let stdout = sys.getattr("stdout").unwrap();
      let orig = sys.getattr("__stdout__").unwrap();
      assert!(stdout.is(&orig));
    });
  }

  #[test]
  fn blocks_path_traversal() {
    let docroot = MockRootBuilder::default()
      .file("safe.py", "print('safe')")
      .build()
      .unwrap();

    let outside = docroot.parent().unwrap().join("secret.py");
    std::fs::write(&outside, "print('secret')").unwrap();

    let embed = Embed::new(&*docroot);
    let request = RequestBuilder::new()
      .method("GET")
      .url("http://localhost/../secret.py")
      .build()
      .unwrap();

    assert!(embed.handle(request).is_err());
  }
}

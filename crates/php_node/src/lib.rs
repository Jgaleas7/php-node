//#[macro_use]
//extern crate napi_derive;

/*mod headers;
mod request;
mod response;
mod rewriter;
mod runtime;

pub use headers::PhpHeaders;
pub use request::PhpRequest;
pub use response::PhpResponse;
pub use rewriter::PhpRewriter;
pub use runtime::PhpRuntime;
*/
#![deny(clippy::all)]
#[macro_use]
extern crate napi_derive;
use napi::{Env, Result, Error, Status};
use once_cell::sync::OnceCell;
use python::Embed;

static EMBED: OnceCell<Embed> = OnceCell::new();

#[napi]
fn run_python_script(script_path: String, docroot: String) -> Result<String> {
    let embed = EMBED.get_or_try_init(|| {
        Embed::new(&docroot).map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
    })?;

    match embed.handle(&script_path) {
        Ok(output) => Ok(output),
        Err(e) => Err(Error::new(Status::GenericFailure, e.to_string())),
    }
}

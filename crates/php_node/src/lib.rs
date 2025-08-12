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
use python::Embed;

// Use a lazy_static or similar for the Python embed to avoid re-initializing it
static EMBED: once_cell::sync::OnceCell<Embed> = once_cell::sync::OnceCell::new();

#[napi]
fn run_python_script(env: Env, script_path: String, docroot: String) -> Result<String> {
    let embed = EMBED.get_or_init(|| {
        Embed::new(&docroot).unwrap()
    });

    match embed.handle(&script_path) {
        Ok(output) => Ok(output),
        Err(e) => Err(Error::new(Status::GenericFailure, e.to_string())),
    }
}

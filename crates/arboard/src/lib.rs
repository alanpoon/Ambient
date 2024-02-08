use std::result::{Result};
use thiserror::Error;
use std::borrow::Cow;
pub struct Clipboard{

}
#[derive(Error)]
#[non_exhaustive]
pub enum Error {
	/// The clipboard contents were not available in the requested format.
	/// This could either be due to the clipboard being empty or the clipboard contents having
	/// an incompatible format to the requested one (eg when calling `get_image` on text)
	#[error("The clipboard contents were not available in the requested format or the clipboard is empty.")]
	ContentNotAvailable,
}
impl std::fmt::Debug for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use Error::*;
		macro_rules! kind_to_str {
			($( $e: pat ),*) => {
				match self {
					$(
						$e => stringify!($e),
					)*
				}
			}
		}
		let name = kind_to_str!(
			ContentNotAvailable
		);
		f.write_fmt(format_args!("{} - \"{}\"", name, self))
	}
}
impl Clipboard {
	/// Creates an instance of the clipboard.
	///
	/// # Errors
	///
	/// On some platforms or desktop environments, an error can be returned if clipboards are not
	/// supported. This may be retried.
	pub fn new() -> Result<Self,  Error> {
		Ok(Clipboard { })
	}
    pub fn set_text<'a, T: Into<Cow<'a, str>>>(&mut self, text: T) -> Result<(),Error> {
		Ok(())
	}
    pub fn get_text(&mut self) -> Result<String, Error> {
		Ok(String::from(""))
	}
}
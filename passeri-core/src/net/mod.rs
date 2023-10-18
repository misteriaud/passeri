use std::error::Error;
#[doc(hidden)]
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
#[doc(hidden)]
pub mod receiver;
#[doc(inline)]
pub use receiver::Receiver;
#[doc(hidden)]
pub mod sender;
#[doc(inline)]
pub use sender::Sender;

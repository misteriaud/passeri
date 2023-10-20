use std::error::Error;
#[doc(hidden)]
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
/// Define a set of enums and thread trait to work with [Receiver] bridge
pub mod receiver;
pub use receiver::Receiver;
/// Define a set of enums and thread trait to work with [Sender] bridge
pub mod sender;
pub use sender::Sender;

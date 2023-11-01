mod error;
pub mod reqwest_client;

pub use error::Error;
pub use reqwest;
use reqwest::Client;
pub use reqwest::Method;
pub use reqwest::Error as ReqwestError;

pub fn client() -> Option<&'static Client> {
    reqwest_client::reqwest_client(true)
}

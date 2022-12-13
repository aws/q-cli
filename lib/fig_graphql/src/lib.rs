#[cfg(feature = "dispatch")]
pub mod dispatch;
pub mod scalar;

pub use graphql_client::GraphQLQuery;
use scalar::*;

include!(concat!(env!("OUT_DIR"), "/queries.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn plugin() {
        let plugin = plugin!(name: "ohmyzsh").await;
        let authors = plugin.unwrap().plugin.unwrap().authors;
        println!("{:?}", authors);
    }

    #[ignore]
    #[tokio::test]
    async fn user() {
        let user = user!().await;
        dbg!(&user);
    }
}

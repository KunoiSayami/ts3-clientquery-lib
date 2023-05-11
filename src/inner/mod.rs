pub mod client;
pub mod connection;
pub mod server;
pub mod types;

mod from_query_string {
    pub trait FromQueryString: for<'de> serde::de::Deserialize<'de> {
        fn from_query(data: &str) -> Result<Self, serde_teamspeak_querystring::Error>
        where
            Self: Sized,
        {
            serde_teamspeak_querystring::from_str(data)
        }
    }

    impl FromQueryString for () {
        fn from_query(_data: &str) -> Result<Self, serde_teamspeak_querystring::Error>
        where
            Self: Sized,
        {
            Ok(())
        }
    }

    impl FromQueryString for String {
        fn from_query(data: &str) -> Result<Self, serde_teamspeak_querystring::Error>
        where
            Self: Sized,
        {
            Ok(data.to_string())
        }
    }
}

pub use from_query_string::FromQueryString;

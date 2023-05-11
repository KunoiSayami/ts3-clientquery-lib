mod schandler_id {
    use super::FromQueryString;
    use serde_derive::Deserialize;
    #[derive(Copy, Clone, Debug, Deserialize)]
    pub struct SchandlerId {
        #[serde(rename = "schandlerid")]
        schandler_id: i64,
    }
    impl SchandlerId {
        pub fn schandler_id(&self) -> i64 {
            self.schandler_id
        }
    }
    impl FromQueryString for SchandlerId {}
}

mod notifies {
    use super::FromQueryString;
    use serde_derive::Deserialize;

    #[derive(Clone, Debug, Deserialize)]
    pub struct NotifyTextMessage {
        #[serde(rename = "targetmode", default)]
        target_mode: i8,
        msg: String,
        #[serde(rename = "invokerid", default)]
        invoker_id: i64,
        #[serde(rename = "invokername", default)]
        invoker_name: String,
        #[serde(rename = "invokeruid", default)]
        invoker_uid: String,
    }

    impl NotifyTextMessage {
        pub fn msg(&self) -> &str {
            &self.msg
        }
        pub fn invoker_name(&self) -> &str {
            &self.invoker_name
        }
        pub fn target_mode(&self) -> i8 {
            self.target_mode
        }
        pub fn invoker_id(&self) -> i64 {
            self.invoker_id
        }
        pub fn invoker_uid(&self) -> &str {
            &self.invoker_uid
        }
    }

    impl FromQueryString for NotifyTextMessage {}
}

mod query_status {
    use crate::error::{QueryError, QueryResult};
    use serde_derive::Deserialize;

    #[derive(Clone, Debug, Deserialize)]
    pub struct QueryStatus {
        id: i32,
        msg: String,
    }

    impl Default for QueryStatus {
        fn default() -> Self {
            Self {
                id: 0,
                msg: "ok".to_string(),
            }
        }
    }

    impl QueryStatus {
        pub fn id(&self) -> i32 {
            self.id
        }
        pub fn msg(&self) -> &String {
            &self.msg
        }

        pub fn into_err(self) -> QueryError {
            QueryError::from(self)
        }

        pub fn into_result<T>(self, ret: T) -> QueryResult<T> {
            if self.id == 0 {
                return Ok(ret);
            }
            Err(self.into_err())
        }
    }

    impl TryFrom<&str> for QueryStatus {
        type Error = QueryError;

        fn try_from(value: &str) -> Result<Self, Self::Error> {
            let (_, line) = value
                .split_once("error ")
                .ok_or_else(|| QueryError::split_error(value))?;
            serde_teamspeak_querystring::from_str(line)
                .map_err(|e| QueryError::parse_error(e, line))
        }
    }
}

pub mod prelude {
    pub use super::NotifyTextMessage;
    pub use super::QueryStatus;
    pub use super::SchandlerId;
}

use crate::inner::FromQueryString;
pub use notifies::NotifyTextMessage;
pub use query_status::QueryStatus;
pub use schandler_id::SchandlerId;

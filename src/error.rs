mod query {
    use crate::error::ErrorKind::OK;
    use crate::inner::types::QueryStatus;
    use std::fmt::{Display, Formatter};
    use std::io::Error;

    #[derive(Clone, Default, Debug)]
    pub struct QueryError {
        code: i32,
        message: String,
    }

    impl QueryError {
        pub fn code(&self) -> i32 {
            self.code
        }
        pub fn message(&self) -> &str {
            &self.message
        }

        pub fn static_empty_response() -> Self {
            Self {
                code: -1,
                message: "Expect result but none found.".to_string(),
            }
        }

        pub fn send_message_error(data: String) -> Self {
            Self {
                code: -2,
                message: format!("Unable to send message, raw data => {}", data),
            }
        }

        pub fn decode_error(data: &str) -> Self {
            Self {
                code: -3,
                message: format!("Decode result error: {}", data),
            }
        }

        pub fn length_mismatch(payload: &str, size: usize) -> Self {
            Self {
                code: -4,
                message: format!(
                    "Error payload size mismatch! expect {} but {} found. payload: {:?}",
                    payload.as_bytes().len(),
                    size,
                    payload
                ),
            }
        }

        pub fn except_data_not_found() -> Self {
            Self {
                code: -5,
                message: "Except data but not found".to_string(),
            }
        }

        pub fn except_data_not_found_payload(payload: &str) -> Self {
            Self {
                code: -5,
                message: format!("Except data but not found, payload => {:?}", payload),
            }
        }

        pub fn parse_error(e: serde_teamspeak_querystring::Error, line: &str) -> Self {
            Self {
                code: -7,
                message: format!("ParseError {:?} {:?}", line, e),
            }
        }

        pub fn split_error(line: &str) -> Self {
            Self {
                code: -7,
                message: format!("SplitError: {:?}", line),
            }
        }

        pub fn deserialize_error(e: serde_teamspeak_querystring::Error) -> Self {
            Self {
                code: -7,
                message: format!("DeserializeError: {:?}", e),
            }
        }
    }

    impl From<Error> for QueryError {
        fn from(value: Error) -> Self {
            Self {
                code: -6,
                message: format!("IOError: {:?}", value),
            }
        }
    }

    impl From<serde_teamspeak_querystring::Error> for QueryError {
        fn from(value: serde_teamspeak_querystring::Error) -> Self {
            QueryError::deserialize_error(value)
        }
    }

    #[non_exhaustive]
    #[derive(Clone, Debug)]
    pub enum ErrorKind {
        EmptyResponse,
        SendMessageError,
        DecodeError,
        LengthMismatch,
        EmptyResultResponse,
        IOError,
        DeserializeError,
        TeamSpeakError,
        OK,
    }

    impl From<QueryError> for ErrorKind {
        fn from(value: QueryError) -> Self {
            match value.code() {
                0 => OK,
                -1 => ErrorKind::EmptyResponse,
                -2 => ErrorKind::SendMessageError,
                -3 => ErrorKind::DecodeError,
                -4 => ErrorKind::LengthMismatch,
                -5 => ErrorKind::EmptyResultResponse,
                -6 => ErrorKind::IOError,
                -7 => ErrorKind::DeserializeError,
                1.. => ErrorKind::TeamSpeakError,
                _ => unreachable!("Should add more error kind to this enum"),
            }
        }
    }

    impl Display for QueryError {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}({})", self.message, self.code)
        }
    }

    impl std::error::Error for QueryError {}

    impl From<QueryStatus> for QueryError {
        fn from(status: QueryStatus) -> Self {
            Self {
                code: status.id(),
                message: status.msg().clone(),
            }
        }
    }

    /*impl From<dyn std::error::Error + Sized> for QueryError {
        fn from(s: Box<dyn std::error::Error>) -> Self {
            Self {
                code: -2,
                message: s.to_string(),
            }
        }
    }*/
}

pub use query::{ErrorKind, QueryError};
pub type QueryResult<T> = Result<T, QueryError>;

mod ts_socket {
    use crate::error::{QueryError, QueryResult};
    use crate::inner::connection::{decode_status, decode_status_with_result};
    use crate::inner::types::prelude::*;
    use crate::inner::FromQueryString;
    #[cfg(feature = "log")]
    use log::{error, trace, warn};
    use serde_teamspeak_querystring::escape;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt, Interest};
    use tokio::net::TcpStream;

    pub const BUFFER_SIZE: usize = 512;

    pub struct TeamspeakConnection {
        conn: TcpStream,
    }

    impl TeamspeakConnection {
        // credit by: Coelacanthus
        pub async fn wait_readable(&mut self) -> Result<bool, tokio::io::Error> {
            Ok(self.conn.ready(Interest::READABLE).await?.is_readable())
        }

        pub async fn read_data(&mut self) -> Result<Option<String>, tokio::io::Error> {
            let mut buffer = [0u8; BUFFER_SIZE];
            let mut ret = String::new();
            loop {
                let size = if let Ok(data) =
                    tokio::time::timeout(Duration::from_secs(2), self.conn.read(&mut buffer)).await
                {
                    match data {
                        Ok(size) => size,
                        Err(e) => return Err(e),
                    }
                } else {
                    return Ok(None);
                };

                ret.push_str(&String::from_utf8_lossy(&buffer[..size]));
                if size < BUFFER_SIZE || (ret.contains("error id=") && ret.ends_with("\n\r")) {
                    break;
                }
            }
            #[cfg(feature = "log")]
            trace!("receive => {:?}", &ret);
            Ok(Some(ret))
        }

        async fn write_data(&mut self, payload: &str) -> Result<(), tokio::io::Error> {
            debug_assert!(payload.ends_with("\n\r"));
            #[cfg(feature = "log")]
            trace!("send => {:?}", payload);
            self.conn.write(payload.as_bytes()).await.map(|size| {
                #[cfg(feature = "log")]
                if size != payload.as_bytes().len() {
                    error!(
                        "Error payload size mismatch! expect {} but {} found. payload: {:?}",
                        payload.as_bytes().len(),
                        size,
                        payload
                    )
                }
            })?;
            Ok(())
        }

        pub async fn keep_alive(&mut self) -> QueryResult<bool> {
            let line = self.query_one_non_error::<String>("whoami\n\r").await?;
            Ok(line.contains("clid=") && line.contains("cid="))
        }

        async fn basic_operation(&mut self, payload: &str) -> QueryResult<()> {
            let data = self.write_and_read(payload).await?;
            decode_status(data).map(|_| ())
        }

        async fn write_and_read(&mut self, payload: &str) -> QueryResult<String> {
            self.write_data(payload).await?;
            self.read_data()
                .await?
                .ok_or_else(QueryError::except_data_not_found)
        }

        pub async fn connect(server: &str, port: u16) -> Result<Self, tokio::io::Error> {
            let conn = TcpStream::connect(format!("{}:{}", server, port)).await?;

            let mut self_ = Self { conn };

            tokio::time::sleep(Duration::from_millis(10)).await;

            let content = self_.read_data().await?;

            #[cfg(feature = "log")]
            if content.is_none() {
                warn!("Read none data.");
            }

            Ok(self_)
        }

        pub async fn register_event(&mut self, event: &str) -> QueryResult<()> {
            self.basic_operation(&format!(
                "clientnotifyregister schandlerid=0 event={}\n\r",
                event
            ))
            .await
        }

        pub async fn login(&mut self, api_key: &str) -> QueryResult<()> {
            let payload = format!("auth apikey={}\n\r", api_key);
            self.basic_operation(payload.as_str()).await
        }

        async fn send_text_message(
            &mut self,
            mode: i64,
            server_id: i64,
            client_id: i64,
            text: &str,
        ) -> QueryResult<()> {
            let payload = format!(
                "sendtextmessage schandlerid={server_id} targetmode={mode} target={client_id} msg={text}\n\r",
                server_id = server_id,
                mode = mode,
                client_id = client_id,
                text = escape(text)
            );
            let data = self.write_and_read(&payload).await.map(|s| {
                if s.contains("\n\r") {
                    s.split_once("\n\r").unwrap().0.to_string()
                } else {
                    s
                }
            })?;
            if data.starts_with("notifytextmessage") {
                let (_, data) = data
                    .split_once("notifytextmessage ")
                    .ok_or_else(|| QueryError::send_message_error(data.clone()))?;
                let r = NotifyTextMessage::from_query(data)
                    .map_err(|_| QueryError::decode_error(data))?;
                //trace!("{:?} {:?}", r.msg(), text);
                if !r.msg().eq(text) {
                    let result = Err(QueryError::send_message_error(
                        "None (No equal)".to_string(),
                    ));
                    return result;
                }
            } else {
                return decode_status(data).map(|_| ());
            }
            Ok(())
        }

        pub async fn send_private_message(
            &mut self,
            server_id: i64,
            client_id: i64,
            text: &str,
        ) -> QueryResult<()> {
            self.send_text_message(1, server_id, client_id, text).await
        }

        pub async fn send_channel_message(
            &mut self,
            server_id: i64,
            text: &str,
        ) -> QueryResult<()> {
            self.send_text_message(2, server_id, 0, text).await
        }

        async fn query_operation_non_error<T: FromQueryString + Sized>(
            &mut self,
            payload: &str,
        ) -> QueryResult<Vec<T>> {
            let data = self.write_and_read(payload).await?;
            let ret = decode_status_with_result(data)?;
            Ok(ret.ok_or_else(|| QueryError::except_data_not_found_payload(payload))?)
        }

        async fn query_one_non_error<T: FromQueryString + Sized>(
            &mut self,
            payload: &str,
        ) -> QueryResult<T> {
            self.query_operation_non_error(payload)
                .await
                .map(|mut v| v.swap_remove(0))
        }

        // TODO: Need test in no connection
        pub async fn get_current_server_tab(&mut self) -> QueryResult<SchandlerId> {
            self.query_one_non_error("currentschandlerid\n\r").await
        }
    }
}

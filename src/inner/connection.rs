use crate::error::{QueryError, QueryResult};
use crate::inner::types::QueryStatus;
use crate::inner::FromQueryString;

pub(crate) fn decode_status(content: String) -> QueryResult<String> {
    for line in content.lines() {
        if line.trim().starts_with("error ") {
            let status = QueryStatus::try_from(line)?;

            return status.into_result(content);
        }
    }
    Err(QueryError::static_empty_response())
}

pub(crate) fn decode_status_with_result<T: FromQueryString + Sized>(
    data: String,
) -> QueryResult<Option<Vec<T>>> {
    let content = decode_status(data)?;

    for line in content.lines() {
        if !line.starts_with("error ") {
            let mut v = Vec::new();
            for element in line.split('|') {
                v.push(T::from_query(element)?);
            }
            return Ok(Some(v));
        }
    }
    Ok(None)
}

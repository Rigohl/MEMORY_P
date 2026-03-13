//! MCP Protocol Implementation
//! Handles JSON-RPC 2.0 protocol messages

#[allow(dead_code)]
pub struct Request {
    pub method: String,
    pub params: Option<String>,
}

#[allow(dead_code)]
pub struct Response {
    pub result: String,
}

#[allow(dead_code)]
pub fn parse_request(data: &[u8]) -> crate::error::Result<Request> {
    Ok(Request {
        method: String::from_utf8_lossy(data).to_string(),
        params: None,
    })
}

#[allow(dead_code)]
pub fn encode_response(response: &Response) -> crate::error::Result<Vec<u8>> {
    Ok(response.result.as_bytes().to_vec())
}

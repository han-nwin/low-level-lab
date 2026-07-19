// POST /users?id=42&active=true HTTP/1.1
// Host: localhost:7878
// User-Agent: curl/8.7.1
// Accept: application/json
// Content-Type: application/json
// Content-Length: 30
// Connection: close
//
// {"name":"Han","role":"admin"}
//
//
// Request {
//     method: Method::Post,
//     path: "/users",
//     query: Some("id=42&active=true"),
//     version: Version::Http11,
//     headers: vec![
//         Header {
//             name: "Host",
//             value: "localhost:7878",
//         },
//         Header {
//             name: "User-Agent",
//             value: "curl/8.7.1",
//         },
//         Header {
//             name: "Accept",
//             value: "application/json",
//         },
//         Header {
//             name: "Content-Type",
//             value: "application/json",
//         },
//         Header {
//             name: "Content-Length",
//             value: "30",
//         },
//         Header {
//             name: "Connection",
//             value: "close",
//         },
//     ],
//     body: Some(b"{\"name\":\"Han\",\"role\":\"admin\"}"),
// }

// assert_eq!(
//     b"{\"name\":\"Han\"}",
//     br#"{"name":"Han"}"#
// );
enum Method {
    POST,
    GET,
}

enum Version {
    HTTP1_0,
    HTTP1_1,
}

#[derive(Debug)]
pub enum ParseError {
    MissingCRLF,
    InvalidRequest,
    InvalidMethod,
    InvalidTarget,
    InvalidVersion,
    InvalidHeader,
}

struct Header {
    name: String,
    value: String,
}

pub struct Request<'a> {
    pub method: Method,
    pub path: &'a str,
    pub query: Option<&'a str>,
    pub version: Version,
    pub headers: Vec<Header>,
    pub body: Option<&'a [u8]>, // byte array -> represent byte string of the body
}

impl<'a> Request<'a> {
    pub fn parse(raw_request_bytes: &'a [u8]) -> Result<Request, ParseError> {
        let mut remaining_bytes = raw_request_bytes;

        let line_end = remaining_bytes
            .windows(2) // similar to .iter() but do 2 elements at a time
            .position(|pair| pair == b"\r\n")
            .ok_or(ParseError::MissingCRLF)?;

        let request_line = &raw_request_bytes[..line_end];

        // 1. Request line
        let request_line_words: Vec<_> = request_line
            .split(|byte| byte.is_ascii_whitespace())
            .collect();

        // 1.1 method
        // > Match the slice as exactly three elements. If it
        // > doesn’t match, execute else and leave the
        // > function.
        let [method_bytes, target_bytes, version_bytes] = request_line_words.as_slice() else {
            return Err(ParseError::InvalidRequest);
        };

        let method_str = str::from_utf8(method_bytes).map_err(|_| ParseError::InvalidMethod)?;
        let method = match method_str {
            "GET" => Method::GET,
            "POST" => Method::POST,
            _ => return Err(ParseError::InvalidMethod),
        };

        let version_str = str::from_utf8(version_bytes).map_err(|_| ParseError::InvalidVersion)?;
        let version = match version_str {
            "HTTP/1.0" => Version::HTTP1_0,
            "HTTP/1.1" => Version::HTTP1_1,
            _ => return Err(ParseError::InvalidVersion),
        };

        // split at most 2
        let mut target_iter = target_bytes.splitn(2, |byte| *byte == 63);
        let path =
            str::from_utf8(target_iter.next().unwrap()).map_err(|_| ParseError::InvalidTarget)?;

        //  The type changes happen like this:
        // Option<&[u8]> (from iter.next())
        //     // .map(str::from_utf8)
        // Option<Result<&str, Utf8Error>>
        //     // .transpose()
        // Result<Option<&str>, Utf8Error>
        //     // .map_err(...)?
        // Option<&str>
        let query: Option<&str> = target_iter
            .next()
            .map(str::from_utf8)
            .transpose()
            .map_err(|_| ParseError::InvalidTarget)?;

        // TODO: 2. header
        // TODO: 3. body
        Ok(Request {
            method,
            path,
            query,
            version,
            headers: vec![],
            body: None,
        })
    }
}

use std::fmt;
use std::str::FromStr;

use anterofit::hyper::error::{Result, Error};
use anterofit::net::header::{Header, HeaderFormat};

#[derive(Clone, PartialEq, Debug)]
pub struct UserAgentHeader(pub String);

impl Header for UserAgentHeader {
    fn header_name() -> &'static str {
        return "User-Agent";
    }

    fn parse_header(raw: &[Vec<u8>]) -> Result<UserAgentHeader> {
        // if raw bytes are malformed or illegal, bot is going to panic
        let header = String::from_utf8(raw[0].clone()).unwrap();

        // see FromStr implementation
        return header.parse::<UserAgentHeader>();
    }
}

impl HeaderFormat for UserAgentHeader {
    fn fmt_header(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        return formatter.write_str(&self.0);
    }
}

impl FromStr for UserAgentHeader {
    type Err = Error;

    fn from_str(header: &str) -> Result<UserAgentHeader> {
        let user_agent: Vec<&str> = header.split(':').collect();

        // designed to crash in case if incoming header is malformed
        return Ok(UserAgentHeader(user_agent[1].trim().to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::UserAgentHeader;
    use anterofit::hyper::header::{Header, Headers};

    #[test]
    fn test_user_agent() {
        let mut headers = Headers::new();
        headers.set(UserAgentHeader("hexocat-bot".to_owned()));

        assert_eq!(headers.to_string(), "User-Agent: hexocat-bot\r\n".to_owned());
    }

    #[test]
    fn test_user_agent_no_agent() {
        let mut headers = Headers::new();
        headers.set(UserAgentHeader("".to_owned()));

        assert_eq!(headers.to_string(), "User-Agent: \r\n".to_owned());
    }

    #[test]
    fn test_user_agent_parse() {
        let header: UserAgentHeader = Header::parse_header(
            &[b"User-Agent: hexocat-bot".to_vec()]).unwrap();

        assert_eq!(header.0, "hexocat-bot");
    }

    #[test]
    fn test_user_agent_parse_no_agent() {
        let header: UserAgentHeader = Header::parse_header(
            &[b"User-Agent: ".to_vec()]).unwrap();

        assert_eq!(header.0, "");
    }
}

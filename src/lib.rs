use url::Url;
use std::io::prelude::*;

pub struct Request {
    host: String,
    path: String,
    content_length: usize,
    data: Option<String>
}

impl Request {
    pub fn from_url(parsed_url: Url) -> Result<Request, &'static str> {
	if parsed_url.scheme() != "spartan" {
	    return Err("Not a spartan URL")
	}

	match parsed_url.host_str() {
	    None => Err("No hostname"),
	    Some(unparsed_host) => {
		let (data, content_length) = match parsed_url.query() {
		    None => (None, 0),
		    Some(s) => {
			let decoded_data = urlencoding::decode(s).unwrap().into_owned();
			let decoded_len = decoded_data.chars().count();
			(Some(decoded_data), decoded_len)
		    }
		};

		let host = unparsed_host.to_string();
		let path = parsed_url.path().to_string();
		
		Ok(Request { host, path, data, content_length })
	    }
	}
    }
}


impl std::fmt::Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
	write!(f, "{} {} {}\r\n{}", self.host, self.path, self.content_length, match &self.data {
	    None => String::new(),
	    Some(d) => d.to_string()
	})
    }
}

pub fn get(r: String) -> Result<String, &'static str> {
    match Url::parse(&r) {
	Err(_) => Err("Cannot parse url"),
	Ok(url) => {
	    let port = url.port_or_known_default().unwrap(); // patched version of `uri` DOES have a default
	    let request = Request::from_url(url)?;
	    let mut connection = std::net::TcpStream::connect(format!("{}:{}", request.host, port)).unwrap();
	    let request_string = request.to_string();
	    
	    match connection.write_all(request_string.as_bytes()) {
		Err(_) => Err("Error writing to socket"),
		Ok(()) => {
		    let mut buffer = String::new();
		    match connection.read_to_string(&mut buffer) {
			Err(_) => Err("Unable to read response"),
			Ok(_) => Ok(buffer)
		    }
		}
	    }
	}
    }
}

#[cfg(test)]
mod tests {
    use crate::Request;

    /// taken from 5.1 of the spec
    
    #[test]
    fn basic_url_mapping_without_slash() {
	let request = Request::from_url(url::Url::parse("spartan://example.com").unwrap()).unwrap();
	assert_eq!("example.com / 0\r\n", format!("{}", request))
    }

    #[test]
    fn basic_url_mapping_with_slash() {
	let request = Request::from_url(url::Url::parse("spartan://example.com/").unwrap()).unwrap();
	assert_eq!("example.com / 0\r\n", format!("{}", request))
    }

    #[test]
    fn basic_url_mapping_with_port() {
	let request = Request::from_url(url::Url::parse("spartan://example.com:3000/").unwrap()).unwrap();
	assert_eq!("example.com / 0\r\n", format!("{}", request))
    }

    #[test]
    fn basic_url_mapping_with_anchor() {
	let request = Request::from_url(url::Url::parse("spartan://example.com/#about").unwrap()).unwrap();
	assert_eq!("example.com / 0\r\n", format!("{}", request))
    }

    #[test]
    fn basic_url_mapping_with_user() {
	let request = Request::from_url(url::Url::parse("spartan://anon@example.com/").unwrap()).unwrap();
	assert_eq!("example.com / 0\r\n", format!("{}", request))
    }

    #[test]
    fn url_mapping_from_ip_address() {
	let request = Request::from_url(url::Url::parse("spartan://127.0.0.1/").unwrap()).unwrap();
	assert_eq!("127.0.0.1 / 0\r\n", format!("{}", request))
    }

    #[test]
    fn url_mapping_from_ip6_address() {
	let request = Request::from_url(url::Url::parse("spartan://[::1]/").unwrap()).unwrap();
	assert_eq!("[::1] / 0\r\n", format!("{}", request))
    }

    #[test]
    fn url_mapping_with_punycode() {
	let request = Request::from_url(url::Url::parse("spartan://examplé.com/").unwrap()).unwrap();
	assert_eq!("xn--exampl-gva.com / 0\r\n", format!("{}", request))
    }

    #[test]
    fn url_mapping_with_urlencoded_path() {
	let request = Request::from_url(url::Url::parse("spartan://example.com/my%20file.txt").unwrap()).unwrap();
	assert_eq!("example.com /my%20file.txt 0\r\n", format!("{}", request))
    }

    #[test]
    fn url_mapping_with_unicode_path() {
	let request = Request::from_url(url::Url::parse("spartan://example.com/café.txt").unwrap()).unwrap();
	assert_eq!("example.com /caf%C3%A9.txt 0\r\n", format!("{}", request))
    }

    #[test]
    fn url_mapping_with_data() {
	let request = Request::from_url(url::Url::parse("spartan://example.com?a=1&b=2").unwrap()).unwrap();
	assert_eq!("example.com / 7\r\na=1&b=2", format!("{}", request))
    }
    
    #[test]
    fn url_mapping_with_urlencoded_data() {
	let request = Request::from_url(url::Url::parse("spartan://example.com?hello%20world").unwrap()).unwrap();
	assert_eq!("example.com / 11\r\nhello world", format!("{}", request))
    }
    
}

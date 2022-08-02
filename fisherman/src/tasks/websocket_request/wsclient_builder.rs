use crate::tasks::toanyhowerror;
use anyhow::{Error};


use hyper::buffer::BufReader;
use hyper::header::{Authorization, Basic};
pub use hyper::header::{
    Connection, ConnectionOption, Headers, Host, Protocol, ProtocolName, Upgrade,
};
use hyper::http::h1::{parse_response, Incoming};
use hyper::http::RawStatus;
use hyper::status::StatusCode;
use hyper::version::HttpVersion;
use log::trace;
use std::borrow::Cow;

use std::net::TcpStream;
use unicase::UniCase;
use websocket::header::{WebSocketAccept, WebSocketKey, WebSocketVersion};
use websocket::native_tls::{TlsConnector, TlsStream};
use websocket::result::{WSUrlErrorKind, WebSocketOtherError};
use websocket::stream::Stream;
use websocket::sync::Client;
use websocket::url::{ParseError, Position, Url};


#[derive(Clone, Debug)]
pub struct WebSocketClientBuilder<'u> {
    url: Cow<'u, Url>,
    version: HttpVersion,
    headers: Headers,
    version_set: bool,
    key_set: bool,
}

impl<'u> WebSocketClientBuilder<'u> {
    pub fn new(address: &str) -> Result<Self, ParseError> {
        let url = Url::parse(address)?;
        Ok(WebSocketClientBuilder::init(Cow::Owned(url)))
    }
    fn init(url: Cow<'u, Url>) -> Self {
        let mut instance = WebSocketClientBuilder {
            url,
            version: HttpVersion::Http11,
            version_set: false,
            key_set: false,
            headers: Headers::new(),
        };
        instance.set_default_header();
        instance
    }
    pub fn establish_tcp(&self) -> Result<TcpStream, Error> {
        self.url
            .with_default_port(|url| match url.scheme() {
                "wss" => Ok(443),
                _ => Ok(80),
            })
            .and_then(|host_and_port| TcpStream::connect(host_and_port))
            .map_err(toanyhowerror)
    }
    fn wrap_ssl(
        &self,
        tcp_stream: TcpStream,
        connector: Option<TlsConnector>,
    ) -> Result<TlsStream<TcpStream>, Error> {
        let (host, connector) = self.extract_host_ssl_conn(connector)?;
        let ssl_stream = connector.connect(host, tcp_stream).map_err(toanyhowerror)?;
        Ok(ssl_stream)
    }
    fn extract_host_ssl_conn(
        &self,
        connector: Option<TlsConnector>,
    ) -> Result<(&str, TlsConnector), Error> {
        let host = match self.url.host_str() {
            Some(h) => h,
            None => {
                return Err(WebSocketOtherError::WebSocketUrlError(
                    WSUrlErrorKind::NoHostName,
                ))
                .map_err(toanyhowerror);
            }
        };
        let connector = match connector {
            Some(c) => c,
            None => TlsConnector::builder().build().map_err(toanyhowerror)?,
        };
        Ok((host, connector))
    }
    pub fn custom_headers(mut self, custom_headers: &Headers) -> Self {
        self.headers.extend(custom_headers.iter());
        self
    }
    pub fn connect_secure(
        &mut self,
        ssl_config: Option<TlsConnector>,
    ) -> Result<Client<TlsStream<TcpStream>>, Error> {
        let tcp_stream = self.establish_tcp()?;

        let ssl_stream = self.wrap_ssl(tcp_stream, ssl_config)?;

        self.connect_on(ssl_stream)
    }
    pub fn connect_insecure(&mut self) -> Result<Client<TcpStream>, Error> {
        let tcp_stream = self.establish_tcp()?;
        self.connect_on(tcp_stream)
    }
    pub fn connect_on<S>(&mut self, mut stream: S) -> Result<Client<S>, anyhow::Error>
    where
        S: Stream,
    {
        // send request
        let resource = self.url[Position::BeforePath..Position::AfterQuery].to_owned();
        let data = format!("GET {} {}\r\n{}\r\n", resource, self.version, self.headers);
        trace!("Data to send in connect request {:?}", &data);
        stream.write_all(data.as_bytes())?;

        // wait for a response
        let mut reader = BufReader::new(stream);
        let response = parse_response(&mut reader).map_err(toanyhowerror)?;

        // validate
        self.validate(&response)?;

        Ok(Client::unchecked(reader, response.headers, true, false))
    }
    fn set_default_header(&mut self) {
        //enter host if available (unix sockets don't have hosts)
        if let Some(host) = self.url.host_str() {
            self.headers.set(Host {
                hostname: host.to_string(),
                port: self.url.port(),
            });
        }

        // handle username/password from URL
        if !self.url.username().is_empty() {
            self.headers.set(Authorization(Basic {
                username: self.url.username().to_owned(),
                password: match self.url.password() {
                    Some(password) => Some(password.to_owned()),
                    None => None,
                },
            }));
        }

        self.headers
            .set(Connection(vec![ConnectionOption::ConnectionHeader(
                UniCase("Upgrade".to_string()),
            )]));

        self.headers.set(Upgrade(vec![Protocol {
            name: ProtocolName::WebSocket,
            version: None,
        }]));

        if !self.version_set {
            self.headers.set(WebSocketVersion::WebSocket13);
        }

        if !self.key_set {
            self.headers.set(WebSocketKey::new());
        }
    }
    fn validate(&self, response: &Incoming<RawStatus>) -> Result<(), Error> {
        let status = StatusCode::from_u16(response.subject.0);

        if status != StatusCode::SwitchingProtocols {
            return Err(WebSocketOtherError::StatusCodeError(status)).map_err(toanyhowerror);
        }

        let key = self
            .headers
            .get::<WebSocketKey>()
            .ok_or(WebSocketOtherError::RequestError(
                "Request Sec-WebSocket-Key was invalid",
            ))?;

        if response.headers.get() != Some(&(WebSocketAccept::new(key))) {
            return Err(WebSocketOtherError::ResponseError(
                "Sec-WebSocket-Accept is invalid",
            ))
            .map_err(toanyhowerror);
        }

        if response.headers.get()
            != Some(
                &(Upgrade(vec![Protocol {
                    name: ProtocolName::WebSocket,
                    version: None,
                }])),
            )
        {
            return Err(WebSocketOtherError::ResponseError(
                "Upgrade field must be WebSocket",
            ))
            .map_err(toanyhowerror);
        }

        if self.headers.get()
            != Some(
                &(Connection(vec![ConnectionOption::ConnectionHeader(UniCase(
                    "Upgrade".to_string(),
                ))])),
            )
        {
            return Err(WebSocketOtherError::ResponseError(
                "Connection field must be 'Upgrade'",
            ))
            .map_err(toanyhowerror);
        }

        Ok(())
    }
}

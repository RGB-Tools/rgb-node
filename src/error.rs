// RGB standard library
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

// TODO: Move parts of this file to common daemon modules (LNP/BP)

use std::collections::HashMap;
use std::io;
use tokio::task::JoinError;

// FIXME: Replace this error type with ServiceError
/// Error used to communicate across FFI & WASM calls
#[derive(Clone, Debug, Display)]
#[display_from(Debug)]
pub struct InteroperableError(pub String);

impl<T> From<T> for InteroperableError
where
    T: std::error::Error,
{
    fn from(err: T) -> Self {
        Self(format!("{}", err))
    }
}

#[derive(Debug, Display, Error, From)]
#[display_from(Debug)]
pub enum BootstrapError {
    TorNotYetSupported,

    #[derive_from]
    IoError(io::Error),

    #[derive_from]
    ArgParseError(String),

    #[derive_from]
    ZmqSocketError(zmq::Error),

    #[derive_from]
    MultithreadError(JoinError),

    MonitorSocketError(Box<dyn std::error::Error>),

    Other,
}

impl From<BootstrapError> for String {
    fn from(err: BootstrapError) -> Self {
        format!("{}", err)
    }
}

impl From<&str> for BootstrapError {
    fn from(err: &str) -> Self {
        BootstrapError::ArgParseError(err.to_string())
    }
}

use lnpbp::bitcoin::hashes::hex;
use std::num::{ParseFloatError, ParseIntError};

#[derive(Clone, Copy, Debug, Display, Error)]
#[display_from(Debug)]
pub struct ParseError;

impl From<ParseFloatError> for ParseError {
    fn from(err: ParseFloatError) -> Self {
        Self
    }
}

impl From<ParseIntError> for ParseError {
    fn from(err: ParseIntError) -> Self {
        Self
    }
}

impl From<hex::Error> for ParseError {
    fn from(err: hex::Error) -> Self {
        Self
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Display, Error)]
#[display_from(Debug)]
pub enum RuntimeError {
    Zmq(ServiceSocketType, String, zmq::Error),
}

impl RuntimeError {
    pub fn zmq_request(socket: &str, err: zmq::Error) -> Self {
        Self::Zmq(ServiceSocketType::Request, socket.to_string(), err)
    }

    pub fn zmq_reply(socket: &str, err: zmq::Error) -> Self {
        Self::Zmq(ServiceSocketType::Request, socket.to_string(), err)
    }

    pub fn zmq_publish(socket: &str, err: zmq::Error) -> Self {
        Self::Zmq(ServiceSocketType::Request, socket.to_string(), err)
    }

    pub fn zmq_subscribe(socket: &str, err: zmq::Error) -> Self {
        Self::Zmq(ServiceSocketType::Request, socket.to_string(), err)
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Display, Error)]
#[display_from(Debug)]
pub enum RoutedError {
    Global(RuntimeError),
    RequestSpecific(ServiceError),
}

#[derive(Clone, PartialEq, Eq, Debug, Display, Error, From)]
#[display_from(Debug)]
pub enum ServiceErrorDomain {
    Io,
    Storage,
    Index,
    Cache,
    Multithreading,
    P2pwire,
    #[derive_from]
    Api(ApiErrorType),
    Monitoring,
    Bifrost,
    BpNode,
    LnpNode,
    Bitcoin,
    Lightning,
    Schema,
    Internal,
}

#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display_from(Debug)]
pub enum ServiceErrorSource {
    Broker,
    Stash,
    Contract(String),
}

#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display_from(Debug)]
pub enum ServiceSocketType {
    Request,
    Reply,
    Publish,
    Subscribe,
}

#[derive(Clone, PartialEq, Eq, Debug, Display, Error)]
#[display_from(Debug)]
pub enum ApiErrorType {
    MalformedRequest { request: String },
    UnknownCommand { command: String },
    UnimplementedCommand,
    MissedArgument { request: String, argument: String },
    UnknownArgument { request: String, argument: String },
    MalformedArgument { request: String, argument: String },
}

#[derive(Clone, PartialEq, Eq, Debug, Display, Error)]
#[display_from(Debug)]
pub struct ServiceError {
    pub domain: ServiceErrorDomain,
    pub service: ServiceErrorSource,
}

impl ServiceError {
    pub fn contract(domain: ServiceErrorDomain, contract_name: &str) -> Self {
        Self {
            domain,
            service: ServiceErrorSource::Contract(contract_name.to_string()),
        }
    }
}

#[derive(Debug, Display, Error)]
#[display_from(Debug)]
pub struct ServiceErrorRepresentation {
    pub domain: String,
    pub service: String,
    pub name: String,
    pub description: String,
    pub info: HashMap<String, String>,
}
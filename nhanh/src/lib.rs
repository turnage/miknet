//! An api for reliable udp connections.
//!
//! ## Concepts
//!
//! These api is composed of the following concepts. Implementation details
//! will vary across implementers.
//!
//! ### Connection
//!
//! A connection is some maintained communication between two UDP sockets.
//! Implementers may build this on UDP in different ways, but all are expected
//! to ensure that the peer is present for the lifetime of the connection.
//!
//! ### Endpoint
//!
//! An endpoint is a client of some implementation of this api. A connection
//! is between exactly two endpoints.
//!
//! ### Datagram
//!
//! A datagram is a block of bytes communicated from one endpoint to another.
//! Datagrams may be any size. There is a clear distinction for a receiver
//! between one datagram and the next.
//!
//! Datagrams "surface" at an endpoint with the api client learns about the
//! datagram.
//!
//! ### Stream
//!
//! A stream is a logical series of datagrams on a connection. Many streams
//! can be multiplexed on a single connection. There is no cost to a stream
//! if no traffic is sent on it.
//!
//! There are three kinds of streams an endpoint can use to send a datagram.
//! They differ in their gaurantees against the risks of UDP. Raw UDP datagrams
//! may suffer any of the following undesired outcomes:
//!
//! * Not arriving at the endpoint
//! * Arriving at the endpoint multiple times
//! * Arriving at the endpoint in an order other than the order they were sent
//!
//! **Ordered** streams gaurantee that datagrams surface at an endpoint exactly
//! once, in order, for example: (1, 2, 3, 4, 5, ...).
//!
//! **Sequenced** streams gaurantee that datagrams surface at an endpont
//! at most once, in order, for example: (1, 2, 6, 7, 10, ...). Datagrams may
//! never surface if a newer packet arrives first.
//!
//! The **unordered** stream gaurantees that datagrams will surface at an
//! endpoint at most once. This stream is a singleton on a connection.
//!
//! Streams buffer independently. For example, one ordered stream waiting on an
//! older datagram before surfacing new ones should have no effect on other
//! ordered streams.

use futures::stream::{FusedStream, Stream};
use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// An identifier for a stream.
///
/// Stream identifiers are unique on a connection.
///
/// Different stream types have independent id spaces. Datagrams
/// can be sent on an ordered stream identified by `StreamId(9)` and other
/// datagrams can be sent on a sequenced stream identified by `StreamId(9)`
/// without conflict.
#[derive(
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    Eq,
    PartialOrd,
    Ord,
    PartialEq,
    Hash,
)]
pub struct StreamId(pub u8);

/// A position of a datagram in a stream.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct StreamPosition {
    /// An identifier for the stream in which the datagram arrived.
    pub stream_id: StreamId,
    pub index: StreamIndex,
}

/// A position in a datagram stream.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum StreamIndex {
    /// The ordinal number of a datagram which arrived in a strictly
    /// ordered stream. Ordinal indices are gauranteed to count up
    /// by steps of `1`.
    Ordinal(u32),
    /// The sequence number of a datagram which arrived in a
    /// sequenced stream. Sequential indices are gauranteed to be
    /// larger than preceding indices.
    Sequence(u32),
}

/// The stream on which to deliver a datagram.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum DeliveryMode {
    /// Delivers a datagram on the ordered stream identified by `StreamId`.
    ///
    /// Datagrams delivered in this mode are gauranteed to surface at the
    /// receiving endpoint exactly once, in order.
    ReliableOrdered(StreamId),
    /// Delivers a datagram on the sequenced stream identified by `StreamId`.
    ///
    /// Datagrams delivered in this mode are gauranteed to surface at the
    /// receiving endpoint at most once, and not after any newer datagrams.
    ///
    /// Datagrams are guaranteed to arrive, but are not gauranteed to surface.
    ReliableSequenced(StreamId),
    /// Delivers a datagram on the connection's unordered stream.
    ///
    /// Datagrams delivered in this mode are gauranteed to surface at the
    /// receiving endpoint exactly once.
    ReliableUnordered,
    /// Delivers a datagram on the sequenced stream identified by `StreamId`.
    ///
    /// Datagrams delivered in this mode are gauranteed to surface at the
    /// receiving endpoint at most once, and not after any newer datagrams.
    UnreliableSequenced(StreamId),
    /// Delivers a datagram on the connection's unordered stream.
    ///
    /// Datagrams delivered in this mode are gauranteed to surface at the
    /// receiving endpoint at most once.
    UnreliableUnordered,
}

/// A block of bytes received from the connected endpoint.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Datagram {
    /// The position of the datagram in the stream in which it was delivered. This
    /// is present if the datagram was sent in a sequenced or ordered stream. If
    /// the datagram was sent on the unordered stream, this will be `None`.
    pub stream_position: Option<StreamPosition>,
    /// The bytes the other endpoint sent.
    pub data: Vec<u8>,
}

/// An api for a bound port, waiting to receive connections.
///
/// The bound port is a stream of new connections. The stream will emit an
/// error and immediately end if there is an error operating the socket.
pub trait Server<Connection: crate::Connection>:
    Stream<Item = Result<Connection>> + FusedStream
{
}

/// An api for communicating with the remote endpoint on a connection.
///
/// `Connection` is a stream of the datagrams from the remote endpoint, and an
/// api for sending datagrams.
///
/// The stream *must be polled* for the connection to make progress and send
/// datagrams.
///
/// If the connection closes, the stream of datagrams will end. An error will
/// be emitted from the stream before close if the disconnection was not
/// correct according to the implementer's protocol.
pub trait Connection: Stream<Item = Result<Datagram>> + FusedStream {
    /// Sends a datagram to the remote endpoint.
    ///
    /// The actual send is performed asynchronously.
    fn send(&mut self, data: &dyn std::io::Read, delivery_mode: DeliveryMode);
}

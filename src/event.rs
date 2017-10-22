//! This module descibes information that is sent between the client and server.
//! It corresponds to some of the ASCII messages that Craft uses.

use std::error::Error;
use std::net::SocketAddr;
use std::num::{ParseFloatError, ParseIntError};
use std::fmt::{self, Display};
use client;

/// A struct that can store both events and their senders.
#[derive(Debug)]
pub struct IdEvent {
    pub id: client::Id,
    pub peer: SocketAddr,
    pub event: Event,
}

/// An enum that can store all kinds of events.
#[derive(Debug)]
pub enum Event {
    /*
    /// An event used purely to wake the server event thread. This can tell the server
    /// that a client's object state has changed. For example, a client could leave the server
    /// while it is running. This only changes state of the *receiving* end of the client,
    /// so the server can be sent this event to be notified of this change
    /// and perform the necessary logic.
    Empty,
    */

    /// Informs the server event thread that the client has left.
    Disconnected,

    /// Represents a player's change in position.
    Position(PositionEvent),

    /// Represents a chat message sent from a client.
    Talk(TalkEvent),

    /// Represents a block placed (or mined) on a client.
    Block(BlockEvent),

    /// Represents a request for chunk data from a client.
    ChunkRequest(ChunkRequestEvent),
}

/// Describes errors that occur parsing messages.
#[derive(Debug)]
pub enum MessageParseError {
     InvalidLength,
     IntError(ParseIntError),
     FloatError(ParseFloatError),
     EmptyMessageError,
}

impl Display for MessageParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for MessageParseError {
    fn description(&self) -> &str {
        match *self {
            MessageParseError::InvalidLength => "The message had an invalid number of payload elements",
            MessageParseError::IntError(ref e) => e.description(),
            MessageParseError::FloatError(ref e) => e.description(),
            MessageParseError::EmptyMessageError => "The message had no content",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            MessageParseError::InvalidLength => None,
            MessageParseError::IntError(ref e) => Some(e),
            MessageParseError::FloatError(ref e) => Some(e),
            MessageParseError::EmptyMessageError => None,
        }
    }
}

/// Corresponds to `P` position messages.
#[derive(Debug)]
pub struct PositionEvent {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rx: f32,
    pub ry: f32,
}

impl PositionEvent {
    /// Create a new position event information structure from an encoded payload.
    pub fn new(payload: &str) -> Result<PositionEvent, MessageParseError> {
        let pieces: Vec<&str> = payload.split(|c| c == ',' || c == '\n').collect();

        if pieces.len() != 5 {
            Self::warn_invalid();
            return Err(MessageParseError::InvalidLength);
        }/* else {
            println!("pieces: {:?}", pieces);
        }*/

        match Self::parse_all(&pieces) {
            Ok(v) => Ok(v),
            Err(e) => {
                Self::warn_invalid();
                Err(MessageParseError::FloatError(e))
            },
        }
    }

    fn parse_all(pieces: &Vec<&str>) -> Result<PositionEvent, ParseFloatError> {
        let x = pieces[0].parse()?;
        let y = pieces[1].parse()?;
        let z = pieces[2].parse()?;
        let rx = pieces[3].parse()?;
        let ry = pieces[4].parse()?;

        Ok(PositionEvent {
            x,
            y,
            z,
            rx,
            ry,
        })
    }

    fn warn_invalid() {
        println!("Warning: invalid position packet.");
    }
}

/// Corresponds to `T` chat messages.
#[derive(Debug)]
pub struct TalkEvent {
    pub text: String,
}

impl TalkEvent {
    /// Create a new chat event information structure from an encoded payload.
    pub fn new(payload: &str) -> Result<TalkEvent, MessageParseError> {
        Ok(TalkEvent {
            text: payload.to_string(),
        })
    }
}

/// Corresponds to `B` block messages.
#[derive(Debug)]
pub struct BlockEvent {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub w: u8,
}

impl BlockEvent {
    /// Create a new block event information structure from an encoded payload.
    pub fn new(payload: &str) -> Result<BlockEvent, MessageParseError> {
        let pieces: Vec<&str> = payload.split(|c| c == ',' || c == '\n').collect();

        if pieces.len() != 4 {
            Self::warn_invalid();
            return Err(MessageParseError::InvalidLength);
        }

        match Self::parse_all(&pieces) {
            Ok(v) => Ok(v),
            Err(e) => {
                Self::warn_invalid();
                Err(MessageParseError::IntError(e))
            },
        }
    }

    fn parse_all(pieces: &Vec<&str>) -> Result<BlockEvent, ParseIntError> {
        let x = pieces[0].parse()?;
        let y = pieces[1].parse()?;
        let z = pieces[2].parse()?;
        let w = pieces[3].parse()?;

        Ok(BlockEvent {
            x,
            y,
            z,
            w,
        })
    }

    fn warn_invalid() {
        println!("Warning: invalid block packet.");
    }
}

/// Corresponds to `C` chunk data request messages.
#[derive(Debug)]
pub struct ChunkRequestEvent {
    pub p: i32,
    pub q: i32,
    //pub key: i32,
}

impl ChunkRequestEvent {
    /// Create a new chunk data request event information structure from an encoded payload.
    pub fn new(payload: &str) -> Result<ChunkRequestEvent, MessageParseError> {
        let pieces: Vec<&str> = payload.split(|c| c == ',' || c == '\n').collect();

        if pieces.len() != 3 {
            Self::warn_invalid();
            return Err(MessageParseError::InvalidLength);
        }

        match Self::parse_all(&pieces) {
            Ok(v) => Ok(v),
            Err(e) => {
                Self::warn_invalid();
                Err(MessageParseError::IntError(e))
            },
        }
    }

    fn parse_all(pieces: &Vec<&str>) -> Result<ChunkRequestEvent, ParseIntError> {
        let p = pieces[0].parse()?;
        let q = pieces[1].parse()?;
        //let key = pieces[2].parse()?;

        Ok(ChunkRequestEvent {
            p,
            q,
            //key,
        })
    }

    fn warn_invalid() {
        println!("Warning: invalid chunk data request packet.");
    }
}

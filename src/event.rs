//! This module descibes information that is sent between the client and server.
//! It corresponds to some of the ASCII messages that Craft uses.

use std::error::Error;
use std::num::ParseFloatError;
use std::fmt::{self, Display};
use client;

/// A struct that can store both events and their senders.
#[derive(Debug)]
pub struct IdEvent {
    pub sender: client::Id,
    pub event: Event,
}

/// An enum that can store all kinds of events.
#[derive(Debug)]
pub enum Event {
    Position(PositionEvent),
}

/// Describes errors that occur parsing messages.
#[derive(Debug)]
pub enum MessageParseError {
     InvalidLength,
     FloatError(ParseFloatError),
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
            MessageParseError::FloatError(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            MessageParseError::InvalidLength => None,
            MessageParseError::FloatError(ref e) => Some(e),
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
    pub fn new(payload: &str) -> Result<PositionEvent, MessageParseError> {
        let pieces: Vec<&str> = payload.split(|c| c == ',' || c == '\n').collect();

        if pieces.len() != 6 {
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

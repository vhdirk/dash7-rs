use std::sync::Arc;

use deku::no_std_io;

use crate::file::{DefaultFileRegistry, FileRegistry};

/// Operations used to build the ALP Actions
pub mod operation;

pub mod interface;

/// ALP basic Actions used to build Commands
pub mod action;

pub mod query;

pub mod command;

#[cfg(feature = "_wizzilab")]
mod interface_final;

// pub trait Parser {
//     fn parse_stream<S>(&self, stream: &S) -> Action
//         where S: no_std_io::Read + no_std_io::Seek;
// }



// pub struct DefaultParser<F: FileRegistry = DefaultFileRegistry> {
//     pub file_registry: Arc<F>,
// }

// impl<F> DefaultParser<F > where F: FileRegistry {



// }


// impl<F> Parser for DefaultParser<F> where F: FileRegistry {
//     fn parse_stream<S>(&self, stream: &S) -> Action
//         where S: no_std_io::Read + no_std_io::Seek {
//             Action::from_reader_with_ctx()
//     }

// }
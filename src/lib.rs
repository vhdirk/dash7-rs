// //! Implementation of a [Dash7](https://dash7-alliance.org/) ALP protocol parser
// //!
// //! The protocol
// //! ==============================================================================
// //! The protocol specifies ALP Commands that can be sent to another system to communicate.
// //! Each command is an aggregation of ALP Actions.
// //!
// //! The protocol is based on the fact that each communicating party hold a Dash7 filesystem.
// //! Each request toward an other device is then composed as an array of simple filesystem operation
// //! (ALP actions).
// //!
// //! Notes
// //! ==============================================================================
// //! Group
// //! ------------------------------------------------------------------------------
// //! Many ALP action have a group flag. This allows those to be grouped.
// //!
// //! This means that:
// //! - If any action of this group fails, the next actions are skipped.
// //! - A query before the group will apply to the whole group (to defined
// //! whether it will be executed).
// //! - If the group contains queries, a prior Logical action will determine how they
// //! are composed between them (OR, XOR, NOR, NAND). Without any Logical action, the
// //! queries are AND'ed.

use mutually_exclusive_features::exactly_one_of;
exactly_one_of!("spec_v1_2", "subiot_v0", "wizzilab_v5_3");

/// Application layer (ALP)
pub mod app;

/// Transport layer
pub mod transport;

/// Network layer
pub mod network;

/// Session lauyer
pub mod session;

/// Physycal layer
pub mod physical;

/// Data/Filesystem layer
pub mod data;

/// Datalink layer
pub mod link;

/// Utility functions
pub(crate) mod utils;

/// reusable types
pub mod types;

#[cfg(test)]
mod test_tools;


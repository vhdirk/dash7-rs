// //! Implementation of a [Dash7](https://dash7-alliance.org/) ALP protocol parser from its
// //! public specification.
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
// //! About this library
// //! ==============================================================================
// //! The goal of this library is to implement a specification with an emphasis on correctness, then
// //! on usability. Performance and memory usage are currently considered a secondary objective.
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

/// ALP
pub mod alp;

#[cfg(test)]
mod test_tools;


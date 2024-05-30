use deku::bitvec::{BitSlice, BitVec, Msb0};
use deku::prelude::*;

use super::operation::{Status, StatusOperation};
use crate::session::{InterfaceFinalStatus, InterfaceTxStatus};
use crate::utils::{read_length_prefixed, write_length_prefixed};

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct InterfaceFinalStatusOperation {
    pub interface_id: u8,

    #[deku(
        reader = "InterfaceFinalStatusOperation::from_reader_with_ctx(deku::reader, *interface_id)",
        writer = "InterfaceFinalStatusOperation::write(deku::writer, &self.status, self.interface_id)"
    )]
    pub status: InterfaceFinalStatus,
}

impl From<InterfaceFinalStatus> for InterfaceFinalStatusOperation {
    fn from(status: InterfaceFinalStatus) -> Self {
        Self {
            interface_id: status.deku_id().unwrap().deku_id().unwrap(),
            status,
        }
    }
}

impl Into<StatusOperation> for InterfaceFinalStatusOperation {
    fn into(self) -> StatusOperation {
        Status::InterfaceFinal(self).into()
    }
}

impl InterfaceFinalStatusOperation {
    pub fn read<'a>(
        rest: &'a BitSlice<u8, Msb0>,
        interface_id: u8,
    ) -> Result<(&'a BitSlice<u8, Msb0>, InterfaceFinalStatus), DekuError> {
        return read_length_prefixed(rest, interface_id);
    }

    pub fn write(
        output: &mut BitVec<u8, Msb0>,
        status: &InterfaceFinalStatus,
        interface_id: u8,
    ) -> Result<(), DekuError> {
        let vec_size = match status {
            InterfaceFinalStatus::Other(val) => val.len() as u32,
            _ => 0,
        };
        return write_length_prefixed(output, status, interface_id, vec_size);
    }
}

#[derive(DekuRead, DekuWrite, Clone, Copy, Debug, PartialEq)]
#[deku(bits = 2, id_type = "u8")]
pub enum TxStatusType {
    #[deku(id = "1")]
    Interface,
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct TxStatusOperation {
    #[deku(update = "self.status.deku_id().unwrap()", pad_bits_after = "6")]
    status_type: TxStatusType,

    #[deku(ctx = "*status_type")]
    pub status: TxStatus,
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(ctx = "status_type: TxStatusType", id = "status_type")]
pub enum TxStatus {
    #[deku(id = "TxStatusType::Interface")]
    Interface(InterfaceTxStatusOperation),
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct InterfaceTxStatusOperation {
    pub interface_id: u8,

    #[deku(
        reader = "InterfaceTxStatusOperation::from_reader_with_ctx(deku::reader, *interface_id)",
        writer = "InterfaceTxStatusOperation::write(deku::writer, &self.status, self.interface_id)"
    )]
    pub status: InterfaceTxStatus,
}

impl From<InterfaceTxStatus> for InterfaceTxStatusOperation {
    fn from(status: InterfaceTxStatus) -> Self {
        Self {
            interface_id: status.deku_id().unwrap().deku_id().unwrap(),
            status,
        }
    }
}

impl InterfaceTxStatusOperation {
    pub fn read<'a>(
        rest: &'a BitSlice<u8, Msb0>,
        interface_id: u8,
    ) -> Result<(&'a BitSlice<u8, Msb0>, InterfaceTxStatus), DekuError> {
        return read_length_prefixed(rest, interface_id);
    }

    pub fn write(
        output: &mut BitVec<u8, Msb0>,
        status: &InterfaceTxStatus,
        interface_id: u8,
    ) -> Result<(), DekuError> {
        let vec_size = match status {
            InterfaceTxStatus::Other(val) => val.len() as u32,
            _ => 0,
        };
        return write_length_prefixed(output, status, interface_id, vec_size);
    }
}

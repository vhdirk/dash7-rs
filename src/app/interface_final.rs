use deku::bitvec::{BitSlice, BitVec, Msb0};
use deku::prelude::*;

use super::operand::{Status, StatusOperand};
use crate::session::{InterfaceFinalStatus, InterfaceTxStatus};
use crate::utils::{read_length_prefixed, write_length_prefixed};

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct InterfaceFinalStatusOperand {
    pub interface_id: u8,

    #[deku(
        reader = "InterfaceFinalStatusOperand::read(deku::rest, *interface_id)",
        writer = "InterfaceFinalStatusOperand::write(deku::output, &self.status, self.interface_id)"
    )]
    pub status: InterfaceFinalStatus,
}

impl From<InterfaceFinalStatus> for InterfaceFinalStatusOperand {
    fn from(status: InterfaceFinalStatus) -> Self {
        Self {
            interface_id: status.deku_id().unwrap().deku_id().unwrap(),
            status,
        }
    }
}

impl Into<StatusOperand> for InterfaceFinalStatusOperand {
    fn into(self) -> StatusOperand {
        Status::InterfaceFinal(self).into()
    }
}

impl InterfaceFinalStatusOperand {
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
#[deku(bits = 2, type = "u8")]
pub enum TxStatusType {
    #[deku(id = "1")]
    Interface,
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct TxStatusOperand {
    #[deku(update = "self.status.deku_id().unwrap()", pad_bits_after = "6")]
    status_type: TxStatusType,

    #[deku(ctx = "*status_type")]
    pub status: TxStatus,
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(ctx = "status_type: TxStatusType", id = "status_type")]
pub enum TxStatus {
    #[deku(id = "TxStatusType::Interface")]
    Interface(InterfaceTxStatusOperand),
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct InterfaceTxStatusOperand {
    pub interface_id: u8,

    #[deku(
        reader = "InterfaceTxStatusOperand::read(deku::rest, *interface_id)",
        writer = "InterfaceTxStatusOperand::write(deku::output, &self.status, self.interface_id)"
    )]
    pub status: InterfaceTxStatus,
}

impl From<InterfaceTxStatus> for InterfaceTxStatusOperand {
    fn from(status: InterfaceTxStatus) -> Self {
        Self {
            interface_id: status.deku_id().unwrap().deku_id().unwrap(),
            status,
        }
    }
}

impl InterfaceTxStatusOperand {
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

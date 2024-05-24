use deku::prelude::*;

use crate::{app::command::Command, types::VarInt};

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
#[deku(bits = 2, id_type = "u8")]
pub enum GroupCondition {
    /// <, =, > (always true)
    #[default]
    #[deku(id = "0")]
    Any,
    /// <, >
    #[deku(id = "1")]
    NotEqual,
    /// =
    #[deku(id = "2")]
    Equal,
    /// >
    #[deku(id = "3")]
    GreaterThan,
}

// TODO: make these names more readable
#[derive(DekuRead, DekuWrite, Clone, Debug, PartialEq, Default)]
pub struct Control {
    #[deku(bits = 1)]
    pub is_dialog_start: bool,
    #[deku(bits = 1)]
    pub has_listen_timeout: bool,
    #[deku(bits = 1)]
    pub has_execution_delay_timeout: bool,
    #[deku(bits = 1)]
    pub is_ack_requested: bool,
    #[deku(bits = 1)]
    pub is_ack_not_void: bool,
    #[deku(bits = 1)]
    pub is_ack_record_requested: bool,
    #[deku(bits = 1)]
    pub has_agc: bool,
}

#[derive(DekuRead, DekuWrite, Clone, Debug, PartialEq, Default)]
pub struct AckTemplate {
    pub transaction_id_start: u8,
    pub transaction_id_stop: u8,
}

#[derive(DekuRead, DekuWrite, Clone, Debug, PartialEq, Default)]
#[deku(ctx = "command_length: u32", ctx_default = "u32::MAX")]
pub struct Frame {
    pub control: Control,

    pub dialog_id: u8,
    pub transaction_id: u8,

    #[deku(cond = "control.has_agc")]
    pub target_rx_level_i: Option<u8>,

    #[deku(cond = "control.has_listen_timeout")]
    pub listen_timeout: Option<VarInt>,

    /// Execution Delay Timeout
    /// For every Request, upper layer provides an Execution Delay Timeout Te for the transaction. If Te > Tt , the Requester
    /// provides the Execution Delay Timeout in Compressed Format. If the Te field in CTRL is set and the segment is not
    /// filtered (8.3.3), the Responders delay their responses by the decompressed value contained in the corresponding byte
    /// starting from the end date of the Request segment reception.
    /// SPEC: 8.2.7
    #[deku(cond = "control.has_execution_delay_timeout")]
    pub execution_delay_timeout: Option<VarInt>,

    // TODO currently we have no way to know if Tc is present or not
    /// Tc is present when control.is_ack_requested AND when we are requester,
    /// while responders copy this flag but do NOT provide a Tc.
    /// When parsing single frames without knowledge of dialogs we cannot determine this.
    /// We use control.is_dialog_start for now but this will break when we start supporting multiple transactions per dialog
    #[deku(cond = "control.is_ack_requested && control.is_dialog_start")]
    pub congestion_timeout: Option<VarInt>,

    #[deku(cond = "control.is_ack_not_void")]
    pub ack_template: Option<AckTemplate>,

    // TODO: is this really the command length or rather the length of the entire message?
    #[deku(ctx = "command_length")]
    pub command: Command,
}

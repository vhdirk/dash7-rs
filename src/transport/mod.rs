use deku::prelude::*;

use crate::{app::command::Command, types::VarInt};

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
#[deku(bits = 2, type = "u8")]
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
    is_dialog_start: bool,
    #[deku(bits = 1)]
    has_tl: bool,
    #[deku(bits = 1)]
    has_te: bool,
    #[deku(bits = 1)]
    is_ack_requested: bool,
    #[deku(bits = 1)]
    is_ack_not_void: bool,
    #[deku(bits = 1)]
    is_ack_record_requested: bool,
    #[deku(bits = 1)]
    has_agc: bool,
}

#[derive(DekuRead, DekuWrite, Clone, Debug, PartialEq, Default)]
pub struct AckTemplate {
    transaction_id_start: u8,
    transaction_id_stop: u8,
}

#[derive(DekuRead, DekuWrite, Clone, Debug, PartialEq, Default)]
#[deku(ctx = "command_length: u32", ctx_default = "u32::MAX")]
pub struct Frame {
    control: Control,

    dialog_id: u8,
    transaction_id: u8,

    #[deku(cond = "control.has_agc")]
    target_rx_level_i: Option<u8>,

    #[deku(cond = "control.has_tl")]
    tl: Option<VarInt>,

    #[deku(cond = "control.has_te")]
    te: Option<VarInt>,

    // TODO currently we have no way to know if Tc is present or not
    // Tc is present when control.is_ack_requested AND when we are requester,
    // while responders copy this flag but do NOT provide a Tc.
    // When parsing single frames without knowledge of dialogs we cannot determine this.
    // We use control.is_dialog_start for now but this will break when we start supporting multiple transactions per dialog
    #[deku(cond = "control.is_ack_requested && control.is_dialog_start")]
    tc: Option<VarInt>,

    #[deku(cond = "control.is_ack_not_void")]
    ack_template: Option<AckTemplate>,

    #[deku(ctx = "command_length")]
    command: Command,
}

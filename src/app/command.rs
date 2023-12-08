use super::action::Action;
use super::operand::{RequestTag, ResponseTag};
use deku::prelude::*;

#[derive(DekuRead, DekuWrite, Clone, Debug, PartialEq, Default)]
#[deku(ctx = "command_length: u32", ctx_default = "u32::MAX")]
pub struct Command {
    // we cannot process an indirect forward without knowing the interface type, which is stored in the interface file
    // as identified by the indirectforward itself
    // As such, we HAVE to bail here
    #[deku(
        until = "|action: &Action| { action.deku_id().unwrap() == OpCode::IndirectForward }",
        bytes_read = "command_length"
    )]
    pub actions: Vec<Action>,
}

impl Command {
    pub fn new(actions: Vec<Action>) -> Self {
        // TODO: validate actions
        Self { actions }
    }

    pub fn request_id(&self) -> Option<u8> {
        for action in self.actions.iter() {
            if let Action::RequestTag(RequestTag { id, .. }) = action {
                return Some(*id);
            }
        }
        None
    }

    pub fn response_id(&self) -> Option<u8> {
        for action in self.actions.iter() {
            if let Action::ResponseTag(ResponseTag { id, .. }) = action {
                return Some(*id);
            }
        }
        None
    }

    pub fn is_last_response(&self) -> bool {
        for action in self.actions.iter() {
            if let Action::ResponseTag(ResponseTag { eop, .. }) = action {
                return *eop;
            }
        }
        false
    }
}

#[cfg(test)]
mod test {

    use hex_literal::hex;

    use crate::{
        app::operand::{ActionHeader, FileOffset, Nop, ReadFileData},
        test_tools::test_item,
    };

    use super::*;

    #[test]
    fn test_command() {
        let cmd = Command {
            actions: vec![
                Action::RequestTag(RequestTag { id: 66, eop: true }),
                Action::ReadFileData(ReadFileData {
                    header: ActionHeader {
                        response: true,
                        group: false,
                    },
                    offset: FileOffset {
                        file_id: 0,
                        offset: 0u32.into(),
                    },
                    length: 8u32.into(),
                }),
                Action::ReadFileData(ReadFileData {
                    header: ActionHeader {
                        response: false,
                        group: true,
                    },
                    offset: FileOffset {
                        file_id: 4,
                        offset: 2u32.into(),
                    },
                    length: 3u32.into(),
                }),
                Action::Nop(Nop {
                    header: ActionHeader {
                        response: true,
                        group: true,
                    },
                }),
            ],
        };
        let data = &hex!("B4 42 41 00 00 08 81 04 02 03 C0");

        test_item(cmd, data);
    }

    #[test]
    fn test_command_request_id() {
        assert_eq!(
            Command {
                actions: vec![
                    Action::RequestTag(RequestTag { eop: true, id: 66 }),
                    Action::Nop(Nop {
                        header: ActionHeader {
                            group: true,
                            response: true
                        }
                    })
                ]
            }
            .request_id(),
            Some(66)
        );
        assert_eq!(
            Command {
                actions: vec![
                    Action::Nop(Nop {
                        header: ActionHeader {
                            group: true,
                            response: false
                        }
                    }),
                    Action::RequestTag(RequestTag { eop: true, id: 44 }),
                ]
            }
            .request_id(),
            Some(44)
        );
        assert_eq!(
            Command {
                actions: vec![
                    Action::Nop(Nop {
                        header: ActionHeader {
                            group: true,
                            response: false
                        }
                    }),
                    Action::Nop(Nop {
                        header: ActionHeader {
                            group: true,
                            response: false
                        }
                    })
                ]
            }
            .request_id(),
            None
        );
    }

    #[test]
    fn test_command_response_id() {
        assert_eq!(
            Command {
                actions: vec![
                    Action::ResponseTag(ResponseTag {
                        eop: true,
                        error: true,
                        id: 66
                    }),
                    Action::Nop(Nop {
                        header: ActionHeader {
                            group: true,
                            response: true
                        }
                    })
                ]
            }
            .response_id(),
            Some(66)
        );
        assert_eq!(
            Command {
                actions: vec![
                    Action::Nop(Nop {
                        header: ActionHeader {
                            group: true,
                            response: false
                        }
                    }),
                    Action::ResponseTag(ResponseTag {
                        eop: true,
                        error: true,
                        id: 44
                    }),
                ]
            }
            .response_id(),
            Some(44)
        );
        assert_eq!(
            Command {
                actions: vec![
                    Action::Nop(Nop {
                        header: ActionHeader {
                            group: true,
                            response: false
                        }
                    }),
                    Action::Nop(Nop {
                        header: ActionHeader {
                            group: true,
                            response: false
                        }
                    })
                ]
            }
            .response_id(),
            None
        );
    }

    #[test]
    fn test_command_is_last_response() {
        assert!(Command {
            actions: vec![
                Action::ResponseTag(ResponseTag {
                    eop: true,
                    error: true,
                    id: 66
                }),
                Action::Nop(Nop {
                    header: ActionHeader {
                        group: true,
                        response: true
                    }
                })
            ]
        }
        .is_last_response());

        assert!(!Command {
            actions: vec![
                Action::ResponseTag(ResponseTag {
                    eop: false,
                    error: false,
                    id: 66
                }),
                Action::Nop(Nop {
                    header: ActionHeader {
                        group: true,
                        response: true
                    }
                })
            ]
        }
        .is_last_response());

        assert!(!Command {
            actions: vec![
                Action::ResponseTag(ResponseTag {
                    eop: false,
                    error: true,
                    id: 44
                }),
                Action::ResponseTag(ResponseTag {
                    eop: true,
                    error: true,
                    id: 44
                }),
            ]
        }
        .is_last_response());

        assert!(!Command {
            actions: vec![
                Action::Nop(Nop {
                    header: ActionHeader {
                        group: true,
                        response: false
                    }
                }),
                Action::Nop(Nop {
                    header: ActionHeader {
                        group: true,
                        response: false
                    }
                })
            ]
        }
        .is_last_response());
    }
}

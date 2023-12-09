#[cfg(feature = "std")]
use std::fmt;
use std::fmt::Display;

#[cfg(not(feature = "std"))]
use alloc::fmt;

use deku::{
    bitvec::{BitSlice, BitVec, BitView, Msb0},
    prelude::*,
};

use crate::utils::pad_rest;

use super::action::Action;
use super::operand::{RequestTag, ResponseTag};

#[derive(DekuRead, DekuWrite, Clone, Debug, PartialEq, Default)]
#[deku(ctx = "command_length: u32")]
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

/// Stub implementation so we can implement DekuContainerRead
impl<'a> DekuRead<'a, ()> for Command {
    fn read(_: &'a BitSlice<u8, Msb0>, _: ()) -> Result<(&'a BitSlice<u8, Msb0>, Self), DekuError>
    where
        Self: Sized,
    {
        unreachable!("This should not have been called")
    }
}

impl DekuWrite<()> for Command {
    fn write(&self, _: &mut BitVec<u8, Msb0>, _: ()) -> Result<(), DekuError> {
        unreachable!("This should not have been called")
    }
}

impl Command {
    pub fn new(actions: Vec<Action>) -> Self {
        // TODO: validate actions
        Self { actions }
    }

    pub fn request_tag(&self) -> Option<&RequestTag> {
        for action in self.actions.iter() {
            if let Action::RequestTag(operand) = action {
                return Some(operand);
            }
        }
        None
    }

    pub fn request_id(&self) -> Option<u8> {
        self.request_tag().map(|t| t.id)
    }

    pub fn response_tag(&self) -> Option<&ResponseTag> {
        for action in self.actions.iter() {
            if let Action::ResponseTag(operand) = action {
                return Some(operand);
            }
        }
        None
    }

    pub fn response_id(&self) -> Option<u8> {
        self.response_tag().map(|t| t.id)
    }

    pub fn tag_id(&self) -> Option<u8> {
        self.request_id().or(self.response_id())
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

impl<'a> DekuContainerRead<'a> for Command {
    fn from_bytes(input: (&'a [u8], usize)) -> Result<((&'a [u8], usize), Self), DekuError> {
        let input_bits = input.0.view_bits::<Msb0>();
        let size = (input_bits.len() - input.1) as u32 / u8::BITS;
        let (rest, value) = Self::read(&input_bits[input.1..], size)?;

        Ok((pad_rest(input_bits, rest), value))
    }
}

/// Stub implementation so we can implement DekuContainerWrite
impl DekuContainerWrite for Command {
    fn to_bytes(&self) -> Result<Vec<u8>, DekuError> {
        let output = self.to_bits()?;
        Ok(output.into_vec())
    }

    fn to_bits(&self) -> Result<BitVec<u8, Msb0>, DekuError> {
        let mut output: BitVec<u8, Msb0> = BitVec::new();
        self.write(&mut output, u32::MAX)?;
        Ok(output)
    }
}

impl TryFrom<&'_ [u8]> for Command {
    type Error = DekuError;
    fn try_from(input: &'_ [u8]) -> Result<Self, Self::Error> {
        let (rest, res) = <Self as DekuContainerRead>::from_bytes((input, 0))?;
        if !rest.0.is_empty() {
            return Err(DekuError::Parse({
                let res = fmt::format(format_args!("Too much data"));
                res
            }));
        }
        Ok(res)
    }
}

impl TryFrom<Command> for Vec<u8> {
    type Error = DekuError;
    fn try_from(input: Command) -> Result<Self, Self::Error> {
        DekuContainerWrite::to_bytes(&input)
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tag_str = self
            .tag_id()
            .map_or("".to_string(), |t| format!("with tag {} ", t));
        f.write_str(&format!("Command {} ", &tag_str))?;

        let status = if let Some(operand) = self.response_tag() {
            if operand.eop {
                if operand.error {
                    "completed, with error"
                } else {
                    "completed, without error"
                }
            } else {
                "executing"
            }
        } else {
            "executing"
        };

        f.write_str(&format!("({})", status))?;

        if self.actions.len() > 0 {
            f.write_str("\n\tactions:\n")?;

            for action in self.actions.iter() {
                f.write_str(&format!("\t\t{:?}\n", action))?;
            }
        }

        // if self.interface_status is not None:
        //   output += "\tinterface status: {}\n".format(self.interface_status)
        // return output

        Ok(())
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

use std::fmt;

use bitvec::{order::Msb0, slice::BitSlice, vec::BitVec, view::BitView};
use deku::{
    DekuContainerRead, DekuContainerWrite, DekuEnumExt, DekuError, DekuRead, DekuUpdate, DekuWrite,
};

use super::action::{Action, OpCode, RequestTag, ResponseTag};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Command {
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

impl DekuRead<'_, ()> for Command {
    fn read(
        input: &'_ BitSlice<u8, Msb0>,
        _: (),
    ) -> Result<(&'_ BitSlice<u8, Msb0>, Self), DekuError>
    where
        Self: Sized,
    {
        let mut rest = input;
        let mut actions = vec![];

        while !rest.is_empty() {
            let (new_rest, action) = <Action as DekuRead<'_, _>>::read(rest, ())?;

            let opcode = action.deku_id()?;
            actions.push(action);
            rest = new_rest;

            if opcode == OpCode::IndirectForward {
                // we cannot process an indirect forward without knowing the interface type, which is stored in the interface file
                // as identified by the indirectforward itself
                // As such, we HAVE to bail here
                return Ok((rest, Self { actions }));
            }
        }

        Ok((rest, Self { actions }))
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
impl DekuContainerRead<'_> for Command {
    fn from_bytes(input: (&'_ [u8], usize)) -> Result<((&'_ [u8], usize), Self), DekuError> {
        let input_bits = input.0.view_bits::<Msb0>();
        let (rest, value) = <Self as DekuRead>::read(&input_bits[input.1..], ())?;

        let pad = 8 * ((rest.len() + 7) / 8) - rest.len();
        let read_idx = input_bits.len() - (rest.len() + pad);
        Ok((
            (input_bits[read_idx..].domain().region().unwrap().1, pad),
            value,
        ))
    }
}

impl DekuUpdate for Command {
    fn update(&mut self) -> Result<(), DekuError> {
        Ok(())
    }
}

impl DekuWrite<()> for Command {
    fn write(&self, output: &mut BitVec<u8, Msb0>, _: ()) -> Result<(), DekuError> {
        for action in self.actions.iter() {
            DekuWrite::write(action, output, ())?;
        }
        Ok(())
    }
}

impl TryFrom<Command> for Vec<u8> {
    type Error = DekuError;
    fn try_from(input: Command) -> Result<Self, Self::Error> {
        DekuContainerWrite::to_bytes(&input)
    }
}

impl DekuContainerWrite for Command {
    fn to_bytes(&self) -> Result<Vec<u8>, DekuError> {
        let output = self.to_bits()?;
        Ok(output.into_vec())
    }

    fn to_bits(&self) -> Result<BitVec<u8, Msb0>, DekuError> {
        let mut output: BitVec<u8, Msb0> = BitVec::new();
        self.write(&mut output, ())?;
        Ok(output)
    }
}

#[cfg(test)]
mod test {

    use hex_literal::hex;

    use crate::{
        alp::{
            action::{ActionHeader, Nop, ReadFileData},
            operand::FileOffset,
        },
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
        let data = &hex!("B4 42   41 00 00 08   81 04 02 03  C0");

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
    fn test_comman_response_id() {
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

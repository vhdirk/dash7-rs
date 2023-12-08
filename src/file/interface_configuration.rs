use deku::prelude::*;

use crate::app::interface;

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct InterfaceConfiguration {
  interface_type: interface::InterfaceType,

  #[deku(ctx="*interface_type")]
  pub configuration: interface::InterfaceConfiguration
}

impl InterfaceConfiguration {
  pub fn new(configuration: interface::InterfaceConfiguration) -> Self{
    Self {
      interface_type: configuration.deku_id().unwrap(),
      configuration
    }
  }
}




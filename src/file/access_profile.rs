use deku::prelude::*;

use crate::link;

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(ctx="specifier: u8")]
pub struct AccessProfile {

  #[deku(skip, default="specifier")]
  pub specifier: u8,
  pub profile: link::AccessProfile,
}


// mod test {
//     use super::AccessProfile;

//   fn test (){

//     let p = AccessProfile::<0> { val: 2 };

//     p.
//   }
// }
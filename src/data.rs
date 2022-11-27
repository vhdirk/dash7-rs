// use crate::codec::{Codec, StdError, WithOffset, WithSize};
// #[cfg(test)]
// use crate::test_tools::test_item;
// #[cfg(test)]
// use hex_literal::hex;



// #[derive(Clone, Copy, Debug, PartialEq)]
// pub struct FileHeader {
//     /// Permissions of the file
//     pub permissions: Permissions,
//     /// Properties of the file
//     pub properties: FileProperties,
//     /// Index of the File containing the ALP Command, executed
//     /// by D7AActP. Discarded if the ACT_EN field in Properties
//     /// is set to 0.
//     pub alp_cmd_fid: u8,
//     /// Index of the File containing the Interface, on which the
//     /// result of D7AActP is sent. Discarded if the ACT_EN field
//     /// in Properties is set to 0.
//     pub interface_file_id: u8,
//     /// Current size of the file.
//     pub file_size: u32,
//     /// Size, allocated for the file in memory (appending data to
//     /// the file cannot exceed this value)
//     pub allocated_size: u32,
//     // ALP_SPEC What is the difference between file_size and allocated_size? When a file is
//     // declared, less than its size is allocated and then it grows dynamically?
// }

// #[test]
// fn test_file_header() {
//     test_item(
//         FileHeader {
//             permissions: Permissions {
//                 encrypted: true,
//                 executable: false,
//                 user: UserPermissions {
//                     read: true,
//                     write: true,
//                     run: true,
//                 },
//                 guest: UserPermissions {
//                     read: false,
//                     write: false,
//                     run: false,
//                 },
//             },
//             properties: FileProperties {
//                 act_en: false,
//                 act_cond: ActionCondition::Read,
//                 storage_class: StorageClass::Permanent,
//             },
//             alp_cmd_fid: 1,
//             interface_file_id: 2,
//             file_size: 0xDEAD_BEEF,
//             allocated_size: 0xBAAD_FACE,
//         },
//         &hex!("B8 13 01 02 DEADBEEF BAADFACE"),
//     )
// }

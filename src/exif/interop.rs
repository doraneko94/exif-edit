use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;

use super::utils::{AllList, RemoveTag, pick_v0, pick_v04};

use crate::{define_str_enum, define_struct};
use crate::components::utils::ShowValue;

define_struct!(
    InteropInfo,
    fields: {
        interop_offset: u32,
        interoperability_index: InteroperabilityIndex,
        interoperability_version: [u8; 4],
    },
    tags: {
        interop_offset: (InteropOffset, pick_v0),
        interoperability_index: (InteroperabilityIndex, |s: &str| Some(InteroperabilityIndex::from_str(s))),
        interoperability_version: (InteroperabilityVersion, pick_v04)
        ;
    }
);

define_str_enum!(
    InteroperabilityIndex {
        R03_DCFOptionFile_AdobeRGB = "R03",
        R98_DCFBasicFile_sRGB = "R98",
        THM_DCFThunmnailFile = "THM",
    }, R03_DCFOptionFile_AdobeRGB
);
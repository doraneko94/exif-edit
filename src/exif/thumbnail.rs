use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;

use super::utils::{RemoveTag, pick_v0};

use crate::define_struct;

define_struct!(
    ThumbnailInfo,
    fields: {
        thumbnail_offset: (Vec<u32>, Vec<u8>),
        thumbnail_length: u32,
    },
    tags: {
        thumbnail_length: (ThumbnailLength, pick_v0);

        // thumbnail_offset: 未実装
        thumbnail_offset: (ThumbnailOffset, |v0: &[u32], v1: &[u8]| Some((v0.to_vec(), v1.to_vec())))
    }
);
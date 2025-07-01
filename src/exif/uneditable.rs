use little_exif::ifd::ExifTagGroup;
use std::collections::HashMap;

#[derive(Clone, PartialEq)]
pub struct Uneditable {
    pub maker_note: Option<String>,
    pub unknown_dict: HashMap<(ExifTagGroup, u16), String>
}

impl Uneditable {
    pub fn new() -> Self {
        Self { maker_note: None, unknown_dict: HashMap::new() }
    }

    pub fn unknown_all(&self) -> Vec<(String, String)> {
        let mut sorted: Vec<_> = self.unknown_dict.iter().collect();
        sorted.sort_by_key(|((group, hex), _)| (*group, *hex));
        
        let mut ret = Vec::with_capacity(sorted.len());
        for (&(group, hex), content) in sorted.iter() {
            ret.push((
                format!(
                    "Unknown({}, hex:0x{:x})",
                    match group {
                        ExifTagGroup::GENERIC => "Generic",
                        ExifTagGroup::EXIF => "Exif",
                        ExifTagGroup::INTEROP => "Interop",
                        ExifTagGroup::GPS => "GPS",
                    },
                    hex
                ),
                content.to_string()
            ));
        }
        ret
    }
}
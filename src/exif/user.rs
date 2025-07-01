use encoding::all::ISO_2022_JP;
use encoding::{DecoderTrap, EncoderTrap, Encoding};
use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;

use super::utils::{RemoveTag, some_string};

use crate::define_struct;

define_struct!(
    UserInfo,
    fields: {
        image_description: String,
        artist: String,
        copyright: String,
        user_comment: UserComment
    },
    tags: {
        image_description: (ImageDescription, some_string),
        artist: (Artist, some_string),
        copyright: (Copyright, some_string),
        user_comment: (UserComment, UserComment::from_vec)
        ;
    }
);

#[derive(Clone, PartialEq)]
pub struct UserComment {
    pub code: UserCommentCode,
    pub data: Vec<u8>,
    pub decoded: String,
}

fn to_hex(v: &[u8]) -> String {
    v.iter().map(|vi| format!("0x{:02X}", vi)).collect::<Vec<String>>().join(", ")
}

impl UserComment {
    pub fn from_vec(v: &[u8]) -> Option<Self> {
        if v.len() < 8 { return None; }
        let mut v_code = [0; 8];
        for i in 0..8 { v_code[i] = v[i]; }
        let code = UserCommentCode::from_array(&v_code);
        let v8 = v[8..].to_vec();
        let decoded = match code {
            UserCommentCode::ASCII => String::from_utf8_lossy(&v8).to_string(),
            UserCommentCode::JIS => match ISO_2022_JP.decode(&v8, DecoderTrap::Strict) {
                Ok(s) => s,
                Err(_) => to_hex(&v8)
            },
            UserCommentCode::Unicode => match String::from_utf8(v8.clone()) {
                Ok(s) => s,
                Err(_) => to_hex(&v8)
            },
            UserCommentCode::Undefined(_) => to_hex(&v8)
        };
        Some(Self { code, data: v.to_vec(), decoded })
    }

    pub fn from_str(s: &str, code: &UserCommentCode) -> Self {
        let mut data = code.to_array().to_vec();
        data.extend_from_slice(
            &match code {
                UserCommentCode::JIS => match ISO_2022_JP.encode(s, EncoderTrap::Strict) {
                    Ok(v) => v,
                    Err(_) => s.as_bytes().to_vec()
                },
                _ => s.as_bytes().to_vec(),
            }
        );
        Self { code: code.clone(), data, decoded: s.to_string() }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum UserCommentCode {
    ASCII,
    JIS,
    Unicode,
    Undefined([u8; 8]),
}

impl UserCommentCode {
    pub fn to_string(&self) -> String {
        match self {
            Self::ASCII => "ASCII".to_string(),
            Self::JIS => "JIS".to_string(),
            Self::Unicode => "Unicode".to_string(),
            Self::Undefined(_) => "Undefined(...)".to_string()
        }
    }
    pub fn from_u64(value: u64) -> Self {
        match value {
            0x4153434949000000 => Self::ASCII,
            0x4A49530000000000 => Self::JIS,
            0x556E69636F646500 => Self::Unicode,
            v => Self::Undefined(v.to_be_bytes())
        }
    }
    pub fn to_u64(&self) -> u64 {
        match self {
            Self::ASCII => 0x4153434949000000,
            Self::JIS => 0x4A49530000000000,
            Self::Unicode => 0x556E69636F646500,
            Self::Undefined(v) => u64::from_be_bytes(v.clone()),
        }
    }
    pub fn from_array(v: &[u8; 8]) -> Self {
        if *v == [0x41, 0x53, 0x43, 0x49, 0x49, 0x00, 0x00, 0x00] {
            Self::ASCII
        } else if *v == [0x4A, 0x49, 0x53, 0x00, 0x00, 0x00, 0x00, 0x00] {
            Self::JIS
        } else if *v == [0x55, 0x6E, 0x69, 0x63, 0x6F, 0x64, 0x65, 0x00] {
            Self::Unicode
        } else {
            Self::Undefined(v.clone())
        }
    }

    pub fn to_array(&self) -> [u8; 8] {
        match self {
            Self::ASCII => [0x41, 0x53, 0x43, 0x49, 0x49, 0x00, 0x00, 0x00],
            Self::JIS => [0x4A, 0x49, 0x53, 0x00, 0x00, 0x00, 0x00, 0x00],
            Self::Unicode => [0x55, 0x6E, 0x69, 0x63, 0x6F, 0x64, 0x65, 0x00],
            Self::Undefined(v) => v.clone()
        }
    }

    pub fn all(&self) -> Vec<(u64, Self)> {
        let mut v = vec![
            (0x4153434949000000, Self::ASCII),
            (0x4A49530000000000, Self::JIS),
            (0x556E69636F646500, Self::Unicode),
        ];
        match self {
            Self::Undefined(val) => { v.push((u64::from_be_bytes(val.clone()), Self::Undefined(val.clone()))); }
            _ => {}
        }
        v
    }
}
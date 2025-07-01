use chrono::{Local, NaiveDateTime, Timelike};
use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;
use little_exif::rational::{iR64, uR64};

use num_traits::{ToPrimitive, NumCast};

use super::exif_capture::TimeOffset;
use super::rational::approx_frac;

#[derive(Clone, Copy, PartialEq)]
pub struct F64 {
    value: f64,
    denominator: u32
}

impl F64 {
    pub fn new(value: f64, denominator: u32) -> Self {
        Self { value, denominator }
    }

    pub fn from_ur64(u: &uR64) -> Self {
        let den = if u.denominator == 0 { 1 } else { u.denominator };
        F64::new(u.nominator as f64 / den as f64, den)
    }

    pub fn from_ir64(i: &iR64) -> Self {
        let den = if i.denominator == 0 { 1 } else { i.denominator };
        let value = i.nominator as f64 / den as f64;
        F64::new(value, den.abs() as u32)
    }

    pub fn value(&self) -> f64 { self.value }
    pub fn den(&self) -> u32 { self.denominator }

    pub fn from_f64(value: f64) -> Result<Self, ()> {
        match approx_frac(value) {
            Some((v, _, d)) => Ok(Self::new(v, d as u32)),
            None => Err(())
        }
    }

    pub fn to_ur64(self) -> Result<uR64, ()> {
        if self.value < 0.0 { Err(()) }
        else { Ok(uR64 {
            nominator: (self.value * self.denominator as f64) as u32,
            denominator: self.denominator
        }) }
    }
}

impl std::fmt::Debug for F64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl std::fmt::Display for F64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

pub fn some_string(s: &str) -> Option<String> {
    Some(s.to_string())
}

pub fn vec_ur64_to_f64(v: &[uR64]) -> Vec<F64> {
    v.iter().map(|vi| F64::from_ur64(&vi)).collect::<Vec<F64>>()
}

pub fn pick_v0<T: Clone>(v: &[T]) -> Option<T> {
    if v.len() < 1 { None }
    else { Some(v[0].clone()) }
}

pub fn pick_v0_cast<T: ToPrimitive + Copy, U: NumCast>(v: &[T]) -> Option<U> {
    if v.len() < 1 { None }
    else { NumCast::from(v[0]) }
}

pub fn pick_v0_ur64(v: &[uR64]) -> Option<F64> {
    if v.len() < 1 { None }
    else { Some(F64::from_ur64(&v[0])) }
}

pub fn pick_v0_ir64(v: &[iR64]) -> Option<F64> {
    if v.len() < 1 { None }
    else { Some(F64::from_ir64(&v[0])) }
}

macro_rules! pick_v0n {
    ($name:ident, $n:expr) => {
        pub fn $name<T: Copy>(v: &[T]) -> Option<[T; $n]> {
            let n = std::cmp::min(v.len(), $n);
            if n < 1 { return None; }
            let mut a = [v[0]; $n];
            for i in 1..n {
                a[i] = v[i];
            }
            Some(a)
        }
    };
}

macro_rules! pick_v0n_ur64 {
    ($name:ident, $n:expr) => {
        pub fn $name(v: &[uR64]) -> Option<[F64; $n]> {
            let n = std::cmp::min(v.len(), $n);
            if n < 1 { return None; }
            let mut a = [F64::from_ur64(&v[0]); $n];
            for i in 1..n {
                a[i] = F64::from_ur64(&v[i]);
            }
            Some(a)
        }
    };
}

pick_v0n!(pick_v02, 2);
pick_v0n!(pick_v04, 4);
pick_v0n!(pick_v0768, 768);

pick_v0n_ur64!(pick_v02_ur64, 2);
pick_v0n_ur64!(pick_v03_ur64, 3);
pick_v0n_ur64!(pick_v04_ur64, 4);
pick_v0n_ur64!(pick_v06_ur64, 6);

pub trait VersionAscii {
    fn to_string(&self) -> Option<String>;
    fn from_string(&mut self, s: &str);
}

impl VersionAscii for Vec<u8> {
    fn to_string(&self) -> Option<String> {
        match String::from_utf8(self.clone()) {
            Ok(s) => Some(s),
            Err(_) => None
        }
    }
    fn from_string(&mut self, s: &str) {
        *self = s.bytes().collect::<Vec<u8>>();
    }
}

pub trait RemoveTag {
    fn remove_tag(&mut self, tag: ExifTag);
}

impl RemoveTag for Metadata {
    fn remove_tag(&mut self, tag: ExifTag) {
        self.get_ifd_mut(tag.get_group(), 0).remove_tag(tag);
    }
}

pub trait AllList: Sized + Clone {
    fn to_u16(&self) -> u16 { 0 }
    fn from_u16(value: u16) -> Self;
    fn all(&self) -> Vec<(u16, Self)> {
        vec![(0, self.clone())]
    }
}

impl AllList for String { fn from_u16(_: u16) -> Self { "".to_string() } }

impl AllList for u16 { fn from_u16(value: u16) -> Self { value } }
impl AllList for u32 { fn from_u16(value: u16) -> Self { value as u32 } }

impl AllList for F64 { fn from_u16(value: u16) -> Self { Self::new(value as f64, 1) } }

impl AllList for Vec<u8> { fn from_u16(value: u16) -> Self { vec![(value % 256) as u8] } }
impl AllList for Vec<u16> { fn from_u16(value: u16) -> Self { vec![value] } }
impl AllList for Vec<u32> { fn from_u16(value: u16) -> Self { vec![value as u32] } }
impl AllList for Vec<F64> { fn from_u16(value: u16) -> Self { vec![F64::new(value as f64, 1)] } }

impl AllList for NaiveDateTime { fn from_u16(_: u16) -> Self {
    Local::now().naive_local().with_nanosecond(0).unwrap()
} }
impl AllList for TimeOffset { fn from_u16(_: u16) -> Self {
    Self::from_str("+00:00").unwrap()
} }

macro_rules! all_list_vec {
    ($type:ty, $n:expr) => {
        impl AllList for [$type; $n] {
            fn from_u16(value: u16) -> Self {
                [value as $type; $n]
            }
        }
    };
}

all_list_vec!(u8, 4);
all_list_vec!(u16, 2);
all_list_vec!(u16, 768);

macro_rules! all_list_f64 {
    ($n:expr) => {
        impl AllList for [F64; $n] {
            fn from_u16(value: u16) -> Self {
                [F64::new(value as f64, 1); $n]
            }
        }
    };
}

all_list_f64!(2);
all_list_f64!(3);
all_list_f64!(4);
all_list_f64!(6);

#[macro_export]
macro_rules! define_big_struct {
    (
        $name: ident,
        $( $field:ident: $struct:ident ),* $(,)?
    ) => {
        #[derive(Clone, PartialEq)]
        pub struct $name {
            $(pub $field: $struct,)*
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    $($field: $struct::new(),)*
                }
            }
        }
    };
}

#[macro_export]
macro_rules! define_struct {
    (
        $name:ident,
        fields: {
            $( $field_name:ident: $field_type:ty ),* $(,)?
        },
        tags: {
            $( $target_field1:ident : ($tag_variant1:ident, $func1:expr) ),* $(,)?
            ;
            $( $target_field2:ident : ($tag_variant2:ident, $func2:expr) ),* $(,)?
        }
    ) => {
        #[derive(Clone, PartialEq)]
        pub struct $name {
            $(pub $field_name: Option<$field_type>,)*
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    $($field_name: None,)*
                }
            }

            pub fn update_tag(&mut self, meta: &mut Metadata, tag: ExifTag) {
                match &tag {
                    $(
                        ExifTag::$tag_variant1(v) => {
                            self.$target_field1 = $func1(v);
                        }
                    )*
                    $(
                        ExifTag::$tag_variant2(v, u) => {
                            self.$target_field2 = $func2(v, u);
                        }
                    )*
                    _ => { return; }
                }
                meta.set_tag(tag);
            }

            pub fn delete_tag(&mut self, meta: &mut Metadata, tag: ExifTag) {
                match &tag {
                    $(
                        ExifTag::$tag_variant1(_) => {
                            self.$target_field1 = None;
                        }
                    )*
                    $(
                        ExifTag::$tag_variant2(_, _) => {
                            self.$target_field2 = None;
                        }
                    )*
                    _ => { return; }
                }
                meta.remove_tag(tag);
            }
        }
    };
}

#[macro_export]
macro_rules! define_enum {
    (
        $name: ident {
            $($variant:ident = $value:expr),* $(,)?
        },
        $unknown:expr
    ) => {
        #[derive(Clone, PartialEq, Eq)]
        #[allow(non_camel_case_types)]
        pub enum $name {
            $($variant),*,
            UnknownValue(Vec<u16>),
        }

        impl $name {
            pub fn to_vec(&self) -> Vec<u16> {
                match self {
                    $(Self::$variant => vec![$value]),*,
                    Self::UnknownValue(v) => v.clone(),
                }
            }

            pub fn from_vec(v: &[u16]) -> Self {
                if v.len() != 1 {
                    Self::UnknownValue(v.to_vec())
                } else {
                    match v[0] {
                        $($value => Self::$variant),*,
                        _ => Self::UnknownValue(v.to_vec()),
                    }
                }
            }

            pub fn from_u8_vec(v: &[u8]) -> Self {
                Self::from_vec(&v.iter().map(|&vi| vi as u16).collect::<Vec<u16>>())
            }

            pub fn to_string(&self) -> String {
                match self {
                    $(Self::$variant => stringify!($variant).to_string()),*,
                    Self::UnknownValue(v) => format!("UnknownValue{:?}", v),
                }
            }

            pub fn unknown() -> Self { Self::UnknownValue(vec![$unknown]) }
        }

        impl AllList for $name {
            fn to_u16(&self) -> u16 {
                self.to_vec()[0]
            }
            fn from_u16(value: u16) -> Self {
                Self::from_vec(&[value])
            }
            fn all(&self) -> Vec<(u16, Self)> {
                let mut v = vec![
                    $(
                        ($value, Self::$variant),
                    )*
                ];
                match self {
                    Self::UnknownValue(val) => { v.push(($unknown, Self::UnknownValue(val.clone()))); }
                    _ => {}
                }
                v
            }
        }

        impl ShowValue for $name {
            fn show_value(&self) -> String { self.to_string() }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.to_string())
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.to_string())
            }
        }

        impl Default for $name {
            fn default() -> Self { Self::unknown() }
        }
    };
}

#[macro_export]
macro_rules! define_str_enum {
    (
        $name: ident {
            $($variant:ident = $value:expr),* $(,)?
        },
        $default:ident
    ) => {
        #[derive(Clone, PartialEq, Eq)]
        #[allow(non_camel_case_types)]
        pub enum $name {
            $($variant),*,
        }

        impl $name {
            pub fn to_str(&self) -> String {
                match self {
                    $(Self::$variant => $value.to_string()),*,
                }
            }

            pub fn from_str(s: &str) -> Self {
                match s {
                    $($value => Self::$variant),*,
                    _ => Self::$default
                }
            }

            pub fn to_string(&self) -> String {
                match self {
                    $(Self::$variant => stringify!($variant).to_string()),*,
                }
            }
        }

        impl AllList for $name {
            fn to_u16(&self) -> u16 {
                let v = vec![$(Self::$variant),*,];
                if let Some(i) = v.iter().position(|x| x == self) {
                    i as u16
                } else { v.len() as u16 }
            }
            fn from_u16(value: u16) -> Self {
                let v = vec![$(Self::$variant),*,];
                let n = value as usize;
                if n >= v.len() { Self::default() }
                else { v[n].clone() }
            }
            fn all(&self) -> Vec<(u16, Self)> {
                let v = vec![
                    $(Self::$variant),*,
                ];
                v.iter().enumerate().map(|(i, vi)| (i as u16, vi.clone())).collect::<Vec<(u16, Self)>>()
            }
        }

        impl ShowValue for $name {
            fn show_value(&self) -> String { self.to_string() }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.to_string())
            }
        }

        impl Default for $name {
            fn default() -> Self { Self::$default }
        }
    };
}
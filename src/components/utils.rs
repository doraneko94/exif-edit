use chrono::NaiveDateTime;
use yew::prelude::*;

use crate::exif::ExifEditData;
use crate::exif::exif_capture::TimeOffset;
use crate::exif::utils::F64;

pub trait ShowValue: PartialEq {
    fn show_value(&self) -> String;
}

impl ShowValue for String { fn show_value(&self) -> String { self.clone() } }

impl ShowValue for u16 { fn show_value(&self) -> String { self.to_string() } }
impl ShowValue for u32 { fn show_value(&self) -> String { self.to_string() } }

impl ShowValue for F64 { fn show_value(&self) -> String { self.value().to_string() } }

impl ShowValue for NaiveDateTime { fn show_value(&self) -> String { format!("{:?}", self) } }
impl ShowValue for TimeOffset { fn show_value(&self) -> String { self.to_string() } }

macro_rules! show_value_vec {
    ($type:ty) => {
        impl ShowValue for Vec<$type> {
            fn show_value(&self) -> String {
                let v = self.iter().map(|vi| vi.to_string()).collect::<Vec<String>>();
                v.join(", ")
            }
        }
    };
}

show_value_vec!(u8);
show_value_vec!(u16);
show_value_vec!(u32);
show_value_vec!(F64);

macro_rules! show_value {
    ($type:ty, $n:expr) => {
        impl ShowValue for [$type; $n] {
            fn show_value(&self) -> String {
                let v = self.iter().map(|vi| vi.to_string()).collect::<Vec<String>>();
                v.join(", ")
            }
        }
    };
}

show_value!(u8, 4);
show_value!(u16, 2);
show_value!(u16, 768);
show_value!(F64, 2);
show_value!(F64, 3);
show_value!(F64, 4);
show_value!(F64, 6);

#[derive(Properties, PartialEq)]
pub struct InfoProps {
    pub exif: UseStateHandle<Option<ExifEditData>>
}

#[macro_export]
macro_rules! ev {
    ($($name:tt).+ , $props:ident) => {
        $props.exif.as_ref().unwrap().$($name).+.clone()
    };
}

#[macro_export]
macro_rules! ev_time {
    ($($name0:tt).+ , $($name1:tt).+ , $props:ident) => {
        if let Some(ndt) = $props.exif.as_ref().unwrap().$($name0).+ {
            if let Some(u) = $props.exif.as_ref().unwrap().$($name1).+ {
                ndt.with_nanosecond(u as u32 * 1_000_000)
            } else { Some(ndt.clone()) }
        } else { None }
    };
}

#[macro_export]
macro_rules! on_string {
    ($name:ident, $tag:ident, $nr:ident [$i:literal], $props:ident) => {
        let $name = {
            let input_ref = $nr[$i].clone();
            let exif = $props.exif.clone();
            Callback::from(move |mode: Mode| {
                let input_ref = input_ref.clone();
                let exif = exif.clone();
                Callback::from(move |_: MouseEvent| {
                    match mode {
                        Mode::Update => {
                            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                                let value = input.value();
                                if let Some(eed) = exif.as_ref() {
                                    let mut eed = eed.clone();
                                    eed.update_tag(ExifTag::$tag(value));
                                    exif.set(Some(eed));
                                }
                            }
                        }
                        Mode::Delete => {
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                let tag = ExifTag::$tag("".to_string());
                                eed.delete_tag(tag);
                                exif.set(Some(eed));
                            }
                        }
                        Mode::Create => {
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                let tag = ExifTag::$tag("".to_string());
                                eed.update_tag(tag);
                                exif.set(Some(eed));
                            }
                        }
                    }
                })
            })
        };
    };
}

#[macro_export]
macro_rules! on_int {
    ($type:ty, $name:ident, $tag:ident, $nr:ident [$i:literal], $props:ident) => {
        let $name = {
            let input_ref = $nr[$i].clone();
            let exif = $props.exif.clone();
            Callback::from(move |mode: Mode| {
                let input_ref = input_ref.clone();
                let exif = exif.clone();
                Callback::from(move |_: MouseEvent| {
                    match mode {
                        Mode::Update => {
                            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                                if let Some(eed) = exif.as_ref() {
                                    let original = eed.pick_value(ExifTag::$tag(vec![0])).unwrap();
                                    let mut eed = eed.clone();
                                    if let Ok(value) = input.value().parse::<$type>() {
                                        eed.update_tag(ExifTag::$tag(vec![value]));
                                        exif.set(Some(eed));
                                    } else {
                                        input.set_value(&original);
                                    }
                                } 
                            }
                        }
                        Mode::Delete => {
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                let tag = ExifTag::$tag(vec![0]);
                                eed.delete_tag(tag);
                                exif.set(Some(eed));
                            }
                        }
                        Mode::Create => {
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                let tag = ExifTag::$tag(vec![0]);
                                eed.update_tag(tag);
                                exif.set(Some(eed));
                            }
                        }
                    }
                })
            })
        };
    };
}

#[macro_export]
macro_rules! on_int_ref {
    ($type:ty, $type_ref:ty, $name:ident, $tag:ident, $nr:ident [$i:literal], $props:ident) => {
        let $name = {
            let input_ref = $nr[$i].clone();
            let exif = $props.exif.clone();
            Callback::from(move |mode: Mode| {
                let input_ref = input_ref.clone();
                let exif = exif.clone();
                Callback::from(move |_: MouseEvent| {
                    match mode {
                        Mode::Update => {
                            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                                if let Some(eed) = exif.as_ref() {
                                    let original = eed.pick_value(ExifTag::$tag(vec![0])).unwrap();
                                    let mut eed = eed.clone();
                                    if let (Ok(value), Ok(_)) = (input.value().parse::<$type>(), input.value().parse::<$type_ref>()) {
                                        eed.update_tag(ExifTag::$tag(vec![value]));
                                        exif.set(Some(eed));
                                    } else {
                                        input.set_value(&original);
                                    }
                                }
                            }
                        }
                        Mode::Delete => {
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                let tag = ExifTag::$tag(vec![0]);
                                eed.delete_tag(tag);
                                exif.set(Some(eed));
                            }
                        }
                        Mode::Create => {
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                let tag = ExifTag::$tag(vec![0]);
                                eed.update_tag(tag);
                                exif.set(Some(eed));
                            }
                        }
                    }
                })
            })
        };
    };
}

#[macro_export]
macro_rules! on_f64 {
    ($type:ident, $name:ident, $tag:ident, $nr:ident [$i:literal], $props:ident) => {
        let $name = {
            let input_ref = $nr[$i].clone();
            let exif = $props.exif.clone();
            Callback::from(move |mode: Mode| {
                let input_ref = input_ref.clone();
                let exif = exif.clone();
                Callback::from(move |_: MouseEvent| {
                    match mode {
                        Mode::Update => {
                            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                                if let Some(eed) = exif.as_ref() {
                                    let original = eed.pick_value(ExifTag::$tag(vec![$type::new(0, 1)])).unwrap();
                                    let mut eed = eed.clone();
                                    let mut reset_flg = false;
                                    if let Ok(value) = input.value().parse::<f64>() {
                                        if let Some((_, nom, den)) = approx_frac(value) {
                                            eed.update_tag(ExifTag::$tag(vec![$type::new(nom, den)]));
                                            exif.set(Some(eed));
                                        } else { reset_flg = true; }
                                    } else { reset_flg = true; }
                                    if reset_flg {
                                        input.set_value(&original);
                                    }
                                }
                            }
                        }
                        Mode::Delete => {
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                let tag = ExifTag::$tag(vec![$type::new(0, 1)]);
                                eed.delete_tag(tag);
                                exif.set(Some(eed));
                            }
                        }
                        Mode::Create => {
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                let tag = ExifTag::$tag(vec![$type::new(0, 1)]);
                                eed.update_tag(tag);
                                exif.set(Some(eed));
                            }
                        }
                    }
                })
            })
        };
    };
}

#[macro_export]
macro_rules! on_enum {
    ($type:ident, $name:ident, $tag:ident, $nr:ident [$i:literal], $props:ident) => {
        let $name = {
            let input_ref = $nr[$i].clone();
            let exif = $props.exif.clone();
            Callback::from(move |mode: Mode| {
                let input_ref = input_ref.clone();
                let exif = exif.clone();
                Callback::from(move |_: MouseEvent| {
                    match mode {
                        Mode::Update => {
                            if let Some(input) = input_ref.cast::<HtmlSelectElement>() {
                                let Ok(value) = input.value().parse::<String>();
                                if let Some(eed) = exif.as_ref() {
                                    let mut eed = eed.clone();
                                    eed.update_tag(ExifTag::$tag(vec![value.parse::<u16>().unwrap()]));
                                    exif.set(Some(eed));
                                }
                            }
                        }
                        Mode::Delete => {
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                let tag = ExifTag::$tag(vec![0]);
                                eed.delete_tag(tag);
                                exif.set(Some(eed));
                            }
                        }
                        Mode::Create => {
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                let tag = ExifTag::$tag(vec![$type::unknown().to_u16()]);
                                eed.update_tag(tag);
                                exif.set(Some(eed));
                            }
                        }
                    }
                })
            })
        };
    };
}

#[macro_export]
macro_rules! on_enum_u8 {
    ($type:ident, $name:ident, $tag:ident, $nr:ident [$i:literal], $props:ident) => {
        let $name = {
            let input_ref = $nr[$i].clone();
            let exif = $props.exif.clone();
            Callback::from(move |mode: Mode| {
                let input_ref = input_ref.clone();
                let exif = exif.clone();
                Callback::from(move |_: MouseEvent| {
                    match mode {
                        Mode::Update => {
                            if let Some(input) = input_ref.cast::<HtmlSelectElement>() {
                                let Ok(value) = input.value().parse::<String>();
                                if let Some(eed) = exif.as_ref() {
                                    let mut eed = eed.clone();
                                    eed.update_tag(ExifTag::$tag(vec![value.parse::<u8>().unwrap()]));
                                    exif.set(Some(eed));
                                }
                            }
                        }
                        Mode::Delete => {
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                let tag = ExifTag::$tag(vec![0]);
                                eed.delete_tag(tag);
                                exif.set(Some(eed));
                            }
                        }
                        Mode::Create => {
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                let tag = ExifTag::$tag(vec![$type::unknown().to_u16() as u8]);
                                eed.update_tag(tag);
                                exif.set(Some(eed));
                            }
                        }
                    }
                })
            })
        };
    };
}

#[macro_export]
macro_rules! on_vec {
    ($type:ty, $name:ident, $tag:ident, $nr:ident [$i:literal], $props:ident) => {
        let $name = {
            let input_ref = $nr[$i].clone();
            let exif = $props.exif.clone();
            Callback::from(move |mode: Mode| {
                let input_ref = input_ref.clone();
                let exif = exif.clone();
                Callback::from(move |_: MouseEvent| {
                    match mode {
                        Mode::Update => {
                            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                                let Ok(value) = input.value().parse::<String>();
                                if let Some(eed) = exif.as_ref() {
                                    let mut eed = eed.clone();
                                    eed.update_tag(ExifTag::$tag(value.split(",")
                                    .filter_map(|piece| {
                                        let t = piece.trim();
                                        if t.is_empty() {
                                            None
                                        } else {
                                            t.parse::<$type>().ok()
                                        }
                                    }).collect()));
                                    exif.set(Some(eed));
                                }
                            }
                        }
                        Mode::Delete => {
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                let tag = ExifTag::$tag(vec![0]);
                                eed.delete_tag(tag);
                                exif.set(Some(eed));
                            }
                        }
                        Mode::Create => {
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                let tag = ExifTag::$tag(vec![0]);
                                eed.update_tag(tag);
                                exif.set(Some(eed));
                            }
                        }
                    }
                })
            })
        };
    };
}

#[macro_export]
macro_rules! on_f64_vec {
    ($type:ident, $name:ident, $tag:ident, $nr:ident [$i:literal], $props:ident) => {
        let $name = {
            let input_ref = $nr[$i].clone();
            let exif = $props.exif.clone();
            Callback::from(move |mode: Mode| {
                let input_ref = input_ref.clone();
                let exif = exif.clone();
                Callback::from(move |_: MouseEvent| {
                    match mode {
                        Mode::Update => {
                            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                                let Ok(value) = input.value().parse::<String>();
                                if let Some(eed) = exif.as_ref() {
                                    let mut eed = eed.clone();
                                    let v = value.split(",")
                                        .filter_map(|piece| {
                                            let t = piece.trim();
                                            if t.is_empty() {
                                                None
                                            } else {
                                                match t.parse::<f64>() {
                                                    Ok(f) => {
                                                        match approx_frac(f) {
                                                            Some((_, nom, den)) => Some($type::new(nom, den)),
                                                            None => None,
                                                        }
                                                    }
                                                    Err(_) => None
                                                }
                                            }
                                        }).collect::<Vec<$type>>();
                                    eed.update_tag(ExifTag::$tag(v));
                                    exif.set(Some(eed));
                                }
                            }
                        }
                        Mode::Delete => {
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                let tag = ExifTag::$tag(vec![$type::new(0, 1)]);
                                eed.delete_tag(tag);
                                exif.set(Some(eed));
                            }
                        }
                        Mode::Create => {
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                let tag = ExifTag::$tag(vec![$type::new(0, 1)]);
                                eed.update_tag(tag);
                                exif.set(Some(eed));
                            }
                        }
                    }
                })
            })
        };
    };
}

#[macro_export]
macro_rules! on_time {
    ($name:ident, $tag0:ident, $tag1:ident, $nr:ident [$i:literal], $props:ident) => {
        let $name = {
            let input_ref = $nr[$i].clone();
            let exif = $props.exif.clone();
            Callback::from(move |mode: Mode| {
                let input_ref = input_ref.clone();
                let exif = exif.clone();
                Callback::from(move |_: MouseEvent| {
                    match mode {
                        Mode::Update => {
                            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                                let value = input.value();
                                let parts = value.split(".").collect::<Vec<&str>>();
                                if parts.len() == 2 {
                                    if let Some(eed) = exif.as_ref() {
                                        let mut eed = eed.clone();
                                        eed.update_tag(ExifTag::$tag0(parts[0].replace("T", " ").replace("-", ":")));
                                        eed.update_tag(ExifTag::$tag1(parts[1].to_string()));
                                        exif.set(Some(eed));
                                    }
                                }
                            }
                        }
                        Mode::Delete => {
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                eed.delete_tag(ExifTag::$tag0("".to_string()));
                                eed.delete_tag(ExifTag::$tag1("".to_string()));
                                exif.set(Some(eed));
                            }
                        }
                        Mode::Create => {
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                let ndt = Local::now().naive_local().with_nanosecond(0).unwrap();
                                eed.update_tag(ExifTag::$tag0(ndt.format("%Y:%m:%d %H:%M:%S").to_string()));
                                eed.update_tag(ExifTag::$tag1("0".to_string()));
                                exif.set(Some(eed));
                            }
                        }
                    }
                })
            })
        };
    };
}

#[macro_export]
macro_rules! on_offset {
    ($name:ident, $tag:ident, $nr:ident [$i:literal], $props:ident) => {
        let $name = {
            let input_ref = $nr[$i].clone();
            let exif = $props.exif.clone();
            Callback::from(move |mode: Mode| {
                let input_ref = input_ref.clone();
                let exif = exif.clone();
                Callback::from(move |_: MouseEvent| {
                    match mode {
                        Mode::Update => {
                            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                                let value = input.value();
                                if let Some(eed) = exif.as_ref() {
                                    let mut eed = eed.clone();
                                    eed.update_tag(ExifTag::$tag(value));
                                    exif.set(Some(eed));
                                }
                            }
                        }
                        Mode::Delete => {
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                let tag = ExifTag::$tag("".to_string());
                                eed.delete_tag(tag);
                                exif.set(Some(eed));
                            }
                        }
                        Mode::Create => {
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                let tag = ExifTag::$tag("+00:00".to_string());
                                eed.update_tag(tag);
                                exif.set(Some(eed));
                            }
                        }
                    }
                })
            })
        };
    };
}

#[macro_export]
macro_rules! on_ascii {
    ($name:ident, $tag:ident, $nr:ident [$i:literal], $props:ident) => {
        let $name = {
            let input_ref = $nr[$i].clone();
            let exif = $props.exif.clone();
            Callback::from(move |mode: Mode| {
                let input_ref = input_ref.clone();
                let exif = exif.clone();
                Callback::from(move |_: MouseEvent| {
                    match mode {
                        Mode::Update => {
                            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                                let value = input.value();
                                if let Some(eed) = exif.as_ref() {
                                    let mut b = value.as_bytes().to_vec();
                                    let n = b.len();
                                    if n > 4 {
                                        b = b[..4].to_vec();
                                    } else if n < 4 {
                                        for _ in n..4 {
                                            b.push(0x30);
                                        }
                                    }
                                    let mut eed = eed.clone();
                                    eed.update_tag(ExifTag::$tag(b));
                                    exif.set(Some(eed));
                                }
                            }
                        }
                        Mode::Delete => {
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                let tag = ExifTag::$tag(Vec::new());
                                eed.delete_tag(tag);
                                exif.set(Some(eed));
                            }
                        }
                        Mode::Create => {
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                let tag = ExifTag::$tag(vec![0x30, 0x30, 0x30, 0x30]);
                                eed.update_tag(tag);
                                exif.set(Some(eed));
                            }
                        }
                    }
                })
            })
        };
    };
}
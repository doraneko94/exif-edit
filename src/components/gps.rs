use chrono::{Local, NaiveDateTime, Timelike};
use little_exif::rational::uR64;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

use little_exif::exif_tag::ExifTag;

use crate::components::utils::ShowValue;
use crate::exif::rational::{approx_frac, ExifRational};
use crate::exif::utils::{AllList, F64};
use crate::{
    ev, on_f64, on_string, on_int, on_vec, on_enum
};

use crate::exif::gps::{
    DMS,
    GPSAltitudeRef, GPSStatus, GPSMeasureMode, GPSSpeedRef, NorthRef,
    GPSDestDistanceRef, GPSDifferential
};

use super::accordion::{Accordion, AccordionMode, Mode};
use super::map_component::MapComponent;
use super::utils::InfoProps;

macro_rules! on_latlng {
    (
        $name:ident,
        $lat_ref:ident, $lat:ident, $lng_ref:ident, $lng:ident, 
        $props:ident
    ) => {
        let $name = {
            let exif = $props.exif.clone();
            Callback::from(move |mode: Mode| {
                let exif = exif.clone();
                Callback::from(move |(lat, lng): (f64, f64)| {
                    let exif = exif.clone();
                    match mode {
                        Mode::Update | Mode::Create => {
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                let lat_ref = if lat >= 0.0 { "N".to_string() } else { "S".to_string() };
                                let lat = DMS::from_f64(lat, None).unwrap().to_vec();
                                let lng_ref = if lng >= 0.0 { "E".to_string() } else { "N".to_string() };
                                let lng = DMS::from_f64(lng, None).unwrap().to_vec();
                                eed.update_tag(ExifTag::$lat_ref(lat_ref));
                                eed.update_tag(ExifTag::$lat(lat));
                                eed.update_tag(ExifTag::$lng_ref(lng_ref));
                                eed.update_tag(ExifTag::$lng(lng));
                                exif.set(Some(eed));
                            }
                        }
                        Mode::Delete => {
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                eed.delete_tag(ExifTag::$lat_ref("".to_string()));
                                eed.delete_tag(ExifTag::$lat(vec![]));
                                eed.delete_tag(ExifTag::$lng_ref("".to_string()));
                                eed.delete_tag(ExifTag::$lng(vec![]));
                                exif.set(Some(eed));
                            }
                        }
                    }        
                })
            })
        };
    };
}

macro_rules! on_with_ref {
    (
        $type:ident,
        $name:ident, $enum:ident, $tag_t:ident, $tag_u:ident, 
        $nr_t:ident [$i_t:literal], $nr_u:ident [$i_u:literal], 
        $props:ident
    ) => {
        let $name = {
            let input_ref_t = $nr_t[$i_t].clone();
            let input_ref_u = $nr_u[$i_u].clone();
            let exif = $props.exif.clone();
            Callback::from(move |mode: Mode| {
                let input_ref_t = input_ref_t.clone();
                let input_ref_u = input_ref_u.clone();
                let exif = exif.clone();
                Callback::from(move |_: MouseEvent| {
                    match mode {
                        Mode::Update => {
                            if let (Some(input_t), Some(input_u)) = (
                                input_ref_t.cast::<HtmlSelectElement>(),
                                input_ref_u.cast::<HtmlInputElement>(),
                            ) {
                                if let Some(eed) = exif.as_ref() {
                                    let original = eed.pick_value(ExifTag::$tag_u(vec![$type::new(0, 1)])).unwrap();
                                    let mut eed = eed.clone();
                                    let mut reset_flg = false;

                                    let Ok(value_t) = input_t.value().parse::<String>();
                                    if let Ok(value_u) = input_u.value().parse::<f64>() {
                                        if let Some((_, nom, den)) = approx_frac(value_u) {
                                            eed.update_tag(ExifTag::$tag_u(vec![$type::new(nom, den)]));
                                            eed.update_tag(ExifTag::$tag_t(value_t));
                                            exif.set(Some(eed));
                                        } else { reset_flg = true; }
                                    } else { reset_flg = true; }
                                    if reset_flg {
                                        input_u.set_value(&original);
                                    }
                                }
                            }
                        }
                        Mode::Delete => {
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                eed.delete_tag(ExifTag::$tag_u(vec![]));
                                eed.delete_tag(ExifTag::$tag_t("".to_string()));
                                exif.set(Some(eed));
                            }
                        }
                        Mode::Create => {
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                eed.update_tag(ExifTag::$tag_u(vec![$type::new(0, 1)]));
                                eed.update_tag(ExifTag::$tag_t($enum::default().show_value()));
                                exif.set(Some(eed));
                            }
                        }
                    }
                })
            })
        };
    };
}

#[function_component(GPSInfo)]
pub fn gps_info(props: &InfoProps) -> Html {
    let input_refs = [
        use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(),
        use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(),
        use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(),
        use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(),
    ];
    on_latlng!(gps, GPSLatitudeRef, GPSLatitude, GPSLongitudeRef, GPSLongitude, props);
    on_string!(gps_map_datum, GPSMapDatum, input_refs[0], props);
    let altitude = {
        let input_ref_t = input_refs[1].clone();
        let input_ref_u = input_refs[2].clone();
        let exif = props.exif.clone();
        Callback::from(move |mode: Mode| {
            let input_ref_t = input_ref_t.clone();
            let input_ref_u = input_ref_u.clone();
            let exif = exif.clone();
            Callback::from(move |_: MouseEvent| {
                match mode {
                    Mode::Update => {
                        if let (Some(input_t), Some(input_u)) = (
                            input_ref_t.cast::<HtmlSelectElement>(),
                            input_ref_u.cast::<HtmlInputElement>(),
                        ) {
                            if let Some(eed) = exif.as_ref() {
                                let original = eed.pick_value(ExifTag::GPSAltitude(vec![uR64::new(0, 1)])).unwrap();
                                let mut eed = eed.clone();
                                let mut reset_flg = false;

                                if let (Ok(value_t), Ok(value_u)) = (input_t.value().parse::<u8>(), input_u.value().parse::<f64>()) {
                                    if let Some((_, nom, den)) = approx_frac(value_u) {
                                        eed.update_tag(ExifTag::GPSAltitude(vec![uR64::new(nom, den)]));
                                        eed.update_tag(ExifTag::GPSAltitudeRef(vec![value_t]));
                                        exif.set(Some(eed));
                                    } else { reset_flg = true; }
                                } else { reset_flg = true; }
                                if reset_flg {
                                    input_u.set_value(&original);
                                }
                            }
                        }
                    }
                    Mode::Delete => {
                        if let Some(eed) = exif.as_ref() {
                            let mut eed = eed.clone();
                            eed.delete_tag(ExifTag::GPSAltitude(vec![]));
                            eed.delete_tag(ExifTag::GPSAltitudeRef(vec![]));
                            exif.set(Some(eed));
                        }
                    }
                    Mode::Create => {
                        if let Some(eed) = exif.as_ref() {
                            let mut eed = eed.clone();
                            eed.update_tag(ExifTag::GPSAltitude(vec![uR64::new(0, 1)]));
                            eed.update_tag(ExifTag::GPSAltitudeRef(vec![GPSAltitudeRef::unknown().to_u16() as u8]));
                            exif.set(Some(eed));
                        }
                    }
                }
            })
        })
    };

    on_string!(gps_satellites, GPSSatellites, input_refs[3], props);
    on_string!(gps_status, GPSStatus, input_refs[4], props);
    on_string!(gps_measure_mode, GPSMeasureMode, input_refs[5], props);
    on_f64!(uR64, gps_dop, GPSDOP, input_refs[6], props);

    on_with_ref!(uR64, gps_speed, GPSSpeedRef, GPSSpeedRef, GPSSpeed, input_refs[7], input_refs[8], props);
    on_with_ref!(uR64, gps_track, NorthRef, GPSTrackRef, GPSTrack, input_refs[9], input_refs[10], props);
    on_with_ref!(uR64, gps_img_direction, NorthRef, GPSImgDirectionRef, GPSImgDirection, input_refs[11], input_refs[12], props);

    on_latlng!(gps_dest, GPSDestLatitudeRef, GPSDestLatitude, GPSDestLongitudeRef, GPSDestLongitude, props);
    on_with_ref!(uR64, gps_dest_bearing, NorthRef, GPSDestBearingRef, GPSDestBearing, input_refs[13], input_refs[14], props);
    on_with_ref!(uR64, gps_dest_distance, GPSDestDistanceRef, GPSDestDistanceRef, GPSDestDistance, input_refs[15], input_refs[16], props);

    on_vec!(u8, gps_processing_method, GPSProcessingMethod, input_refs[17], props);
    on_vec!(u8, gps_area_information, GPSAreaInformation, input_refs[18], props);
    let date_time = {
        let input_ref = input_refs[19].clone();
        let exif = props.exif.clone();
        Callback::from(move |mode: Mode| {
            let input_ref = input_ref.clone();
            let exif = exif.clone();
            Callback::from(move |_: MouseEvent| {
                match mode {
                    Mode::Update => {
                        if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                let value = input.value();
                                let parts = value.split("T").collect::<Vec<&str>>();
                                if parts.len() == 2 {
                                    let s_date = parts[0].replace("-", ":");
                                    let parts = parts[1].split(":").collect::<Vec<&str>>();
                                    if parts.len() == 3 {
                                        if let (Ok(h), Ok(m), Ok(s)) = (
                                            parts[0].parse::<i32>(),
                                            parts[1].parse::<i32>(),
                                            parts[2].parse::<f64>(),
                                        ) {
                                            let v_time = vec![
                                                uR64::new(h, 1),
                                                uR64::new(m, 1),
                                                uR64::new((s * 1000.0) as i32, 1000)
                                            ];
                                            eed.update_tag(ExifTag::GPSDateStamp(s_date));
                                            eed.update_tag(ExifTag::GPSTimeStamp(v_time));
                                            exif.set(Some(eed));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Mode::Delete => {
                        if let Some(eed) = exif.as_ref() {
                            let mut eed = eed.clone();
                            eed.delete_tag(ExifTag::GPSDateStamp("".to_string()));
                            eed.delete_tag(ExifTag::GPSTimeStamp(vec![]));
                            exif.set(Some(eed));
                        }
                    }
                    Mode::Create => {
                        if let Some(eed) = exif.as_ref() {
                            let mut eed = eed.clone();
                            let now = Local::now().naive_local();
                            let nd = now.date();
                            let nt = now.time();
                            eed.update_tag(ExifTag::GPSDateStamp(format!("{:?}", nd).replace("-", ":")));
                            eed.update_tag(ExifTag::GPSTimeStamp(vec![
                                uR64 { nominator: nt.hour(), denominator: 1 },
                                uR64 { nominator: nt.minute(), denominator: 1 },
                                uR64 { nominator: nt.second() * 1000, denominator: 1000 }
                            ]));
                            exif.set(Some(eed));
                        }
                    }
                }
            })
        })
    };
    on_enum!(GPSDifferential, gps_differential, GPSDifferential, input_refs[20], props);
    on_f64!(uR64, gps_h_positioning_error, GPSHPositioningError, input_refs[21], props);
    on_vec!(u8, gps_version_id, GPSVersionID, input_refs[22], props);
    on_int!(u32, gps_info, GPSInfo, input_refs[23], props);

    html! {
        <div>
            <div class="tab-content border border-top-0 p-3">
            <div class="accordion">
                <AccordionGPS
                    name={ "GPSLatitude / GPSLongitude" }
                    lead={Some("撮影場所の緯度経度")}
                    lat_ref={ev!(gps_info.location_info.gps_latitude_ref, props)}
                    lat={ev!(gps_info.location_info.gps_latitude, props)}
                    lng_ref={ev!(gps_info.location_info.gps_longitude_ref, props)}
                    lng={ev!(gps_info.location_info.gps_longitude, props)}
                    on_func={gps} />
                <Accordion<String>
                    name={ "GPSMapDatum" }
                    lead={Some("使用されている測地系")}
                    input_ref={input_refs[0].clone()}
                    value={ev!(gps_info.location_info.gps_map_datum, props)}
                    on_func={gps_map_datum} />
                <AccordionTwin<GPSAltitudeRef, F64>
                    name={ "GPSAltitude / GPSAltitudeRef" }
                    lead={Some("撮影場所の高度")}
                    mode={TwinMode::WithRef}
                    value_t={ev!(gps_info.location_info.gps_altitude_ref, props)}
                    value_u={ev!(gps_info.location_info.gps_altitude, props)}
                    input_refs={[input_refs[1].clone(), input_refs[2].clone()]}
                    on_func={altitude} />

                <Accordion<String>
                    name={ "GPSSatellites" }
                    lead={Some("使用している衛星の情報")}
                    input_ref={input_refs[3].clone()}
                    value={ev!(gps_info.location_info.gps_satellites, props)}
                    on_func={gps_satellites} />
                <Accordion<GPSStatus>
                    name={ "GPSStatus" }
                    lead={Some("GPS測位の状態")}
                    mode={AccordionMode::Dropdown}
                    input_ref={input_refs[4].clone()}
                    value={ev!(gps_info.location_info.gps_status, props)}
                    on_func={gps_status} />
                <Accordion<GPSMeasureMode>
                    name={ "GPSMeasureMode" }
                    lead={Some("2Dまたは3D測位")}
                    mode={AccordionMode::Dropdown}
                    input_ref={input_refs[5].clone()}
                    value={ev!(gps_info.location_info.gps_measure_mode, props)}
                    on_func={gps_measure_mode} />
                <Accordion<F64>
                    name={ "GPSDOP" }
                    lead={Some("測位精度 (Dilution of Precision)")}
                    input_ref={input_refs[6].clone()}
                    value={ev!(gps_info.location_info.gps_dop, props)}
                    on_func={gps_dop} />

                <AccordionTwin<GPSSpeedRef, F64>
                    name={ "GPSSpeed / GPSSpeedRef" }
                    lead={Some("撮影時の移動速度")}
                    mode={TwinMode::WithRef}
                    value_t={ev!(gps_info.location_info.gps_speed_ref, props)}
                    value_u={ev!(gps_info.location_info.gps_speed, props)}
                    input_refs={[input_refs[7].clone(), input_refs[8].clone()]}
                    on_func={gps_speed} />
                <AccordionTwin<NorthRef, F64>
                    name={ "GPSTrack / GPSTrackRef" }
                    lead={Some("撮影時の移動方向")}
                    mode={TwinMode::WithRef}
                    value_t={ev!(gps_info.location_info.gps_track_ref, props)}
                    value_u={ev!(gps_info.location_info.gps_track, props)}
                    input_refs={[input_refs[9].clone(), input_refs[10].clone()]}
                    on_func={gps_track} />
                <AccordionTwin<NorthRef, F64>
                    name={ "GPSImgDirection / GPSImgDirectionRef" }
                    lead={Some("撮影時のカメラの向き")}
                    mode={TwinMode::WithRef}
                    value_t={ev!(gps_info.location_info.gps_img_direction_ref, props)}
                    value_u={ev!(gps_info.location_info.gps_img_direction, props)}
                    input_refs={[input_refs[11].clone(), input_refs[12].clone()]}
                    on_func={gps_img_direction} />
                
                <AccordionGPS
                    name={ "GPSDestLatitude / GPSDestLongitude" }
                    lead={Some("目的地の緯度経度")}
                    lat_ref={ev!(gps_info.location_info.gps_dest_latitude_ref, props)}
                    lat={ev!(gps_info.location_info.gps_dest_latitude, props)}
                    lng_ref={ev!(gps_info.location_info.gps_dest_longitude_ref, props)}
                    lng={ev!(gps_info.location_info.gps_dest_longitude, props)}
                    on_func={gps_dest} />
                <AccordionTwin<NorthRef, F64>
                    name={ "GPSDestBearing / GPSDestBearingRef" }
                    lead={Some("目的地の方角")}
                    mode={TwinMode::WithRef}
                    value_t={ev!(gps_info.location_info.gps_dest_bearing_ref, props)}
                    value_u={ev!(gps_info.location_info.gps_dest_bearing, props)}
                    input_refs={[input_refs[13].clone(), input_refs[14].clone()]}
                    on_func={gps_dest_bearing} />
                <AccordionTwin<GPSDestDistanceRef, F64>
                    name={ "GPSDestDistance / GPSDestDistanceRef" }
                    lead={Some("目的地までの距離")}
                    mode={TwinMode::WithRef}
                    value_t={ev!(gps_info.location_info.gps_dest_distance_ref, props)}
                    value_u={ev!(gps_info.location_info.gps_dest_distance, props)}
                    input_refs={[input_refs[15].clone(), input_refs[16].clone()]}
                    on_func={gps_dest_distance} />

                <Accordion<Vec<u8>>
                    name={ "GPSProcessingMethod" }
                    lead={Some("測地の方法")}
                    input_ref={input_refs[17].clone()}
                    value={ev!(gps_info.location_info.gps_processing_method, props)}
                    on_func={gps_processing_method} />
                <Accordion<Vec<u8>>
                    name={ "GPSAreaInformation" }
                    lead={Some("地域情報テキスト")}
                    input_ref={input_refs[18].clone()}
                    value={ev!(gps_info.location_info.gps_area_information, props)}
                    on_func={gps_area_information} />
                <Accordion<NaiveDateTime>
                    name={ "GPSDateStamp / GPSTimeStamp" }
                    lead={Some("撮影時の日時 (UTC)")}
                    mode={AccordionMode::Time}
                    input_ref={input_refs[19].clone()}
                    value={
                        if let (Some(nd), Some(nt)) = (
                            props.exif.as_ref().unwrap().gps_info.location_info.gps_date_stamp,
                            props.exif.as_ref().unwrap().gps_info.location_info.gps_time_stamp
                        ) {
                            Some(nd.and_time(nt))
                        } else {
                            None
                        }
                    }
                    on_func={date_time} />
                <Accordion<GPSDifferential>
                    name={ "GPSDifferential" }
                    lead={Some("差分GPS補正の有無")}
                    mode={AccordionMode::Dropdown}
                    input_ref={input_refs[20].clone()}
                    value={ev!(gps_info.location_info.gps_differential, props)}
                    on_func={gps_differential} />
                <Accordion<F64>
                    name={ "GPSHPositioningError" }
                    lead={Some("水平方向の誤差推定")}
                    input_ref={input_refs[21].clone()}
                    value={ev!(gps_info.location_info.gps_h_positioning_error, props)}
                    on_func={gps_h_positioning_error} />
                <Accordion<[u8; 4]>
                    name={ "GPSVersionID" }
                    lead={Some("GPSのバージョン: 長さ 4")}
                    input_ref={input_refs[22].clone()}
                    value={ev!(gps_info.location_info.gps_version_id, props)}
                    on_func={gps_version_id} />
                <Accordion<u32>
                    name={ "GPSInfo" }
                    lead={Some("Exifへのオフセットポインタ")}
                    input_ref={input_refs[23].clone()}
                    value={ev!(gps_info.location_info.gps_info, props)}
                    on_func={gps_info} />
            </div>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct AccordionGPSProps {
    pub name: &'static str,
    #[prop_or_default]
    pub lead: Option<&'static str>,
    pub lat_ref: Option<bool>,
    pub lat: Option<DMS>,
    pub lng_ref: Option<bool>,
    pub lng: Option<DMS>,
    pub on_func: Callback<Mode, Callback<(f64, f64)>>
}

#[function_component(AccordionGPS)]
pub fn accordion_gps(props: &AccordionGPSProps) -> Html {
    let id_safe = props.name.replace(" ", "-").replace("/", "-");
    let (is_open, lat, lng) = match (&props.lat_ref, &props.lat, &props.lng_ref, &props.lng) {
        (Some(lat_ref), Some(lat), Some(lng_ref), Some(lng)) => (
            true, 
            if *lat_ref { lat.to_f64() } else { -lat.to_f64() },
            if *lng_ref { lng.to_f64() } else { -lng.to_f64() },
        ),
        _ => (false, 0.0, 0.0)
    };
    let red_latlng = use_state(|| (lat, lng));

    let btn_classes = classes!(
        "accordion-button",
        if !is_open { "collapsed" } else { "" },
    );

    let collapse_classes = if is_open {
        "accordion-collapse collapse show"
    } else {
        "accordion-collapse collapse"
    };

    let on_func = props.on_func.clone();
    let on_update_base = on_func.emit(Mode::Update);
    let on_delete_base = on_func.emit(Mode::Delete);
    let on_create_base = on_func.emit(Mode::Create);

    let on_update = {
        let on_update_base = on_update_base.clone();
        let red_latlng = red_latlng.clone();
        Callback::from(move |_: MouseEvent| {
            let (lat, lng) = *red_latlng;
            on_update_base.emit((lat, lng));
        })
    };

    let on_delete = {
        let on_delete_base = on_delete_base.clone();
        Callback::from(move |_: MouseEvent| {
            on_delete_base.emit((0.0, 0.0));
        })
    };

    let on_create = {
        let on_create_base = on_create_base.clone();
        Callback::from(move |_: MouseEvent| {
            on_create_base.emit((0.0, 0.0));
        })
    };

    let on_marker_move = {
        let red_latlng = red_latlng.clone();
        Callback::from(move |(lat, lng): (f64, f64)| {
            red_latlng.set((lat, lng));
        })
    };

    html! {
        <div class="accordion-item">
            <h2 class="accordion-header" id={format!("heading-{}", id_safe.clone())}>
                <button class={btn_classes} type="button"
                    data-bs-toggle="collapse"
                    data-bs-target={format!("#{}", id_safe)}
                    aria-expanded={ is_open.to_string() }
                    aria-controls={id_safe.clone()}>
                    <div class="d-flex flex-column text-start w-100">
                        <span>{ props.name }</span>
                        {
                            if let Some(lead) = &props.lead {
                                html! {
                                    <small class="text-muted">{ lead }</small>
                                }
                            } else {
                                html! {}
                            }
                        }
                    </div>
                </button>
            </h2>
            <div id={id_safe.clone()}
                class={collapse_classes}
                aria-labelledby={format!("heading-{}", id_safe.clone())}>
                <div class="accordion-body">
                { if is_open {
                    html! {
                        <>
                        <p>{ format!("緯度：{:.6}", lat) }</p>
                        <p>{ format!("経度：{:.6}", lng) }</p>
                        <div class="mb-3">
                        <MapComponent {lat} {lng} {on_marker_move} />
                        </div>
                        <div class="d-flex justify-content-end gap-2 mb-3">
                            <button type="button" class="btn btn-primary" onclick={on_update.clone()}>{ "更新" }</button>
                            <button type="button" class="btn btn-danger" onclick={on_delete.clone()}>{ "削除" }</button>
                        </div>
                        </>
                    }
                } else {
                    html! {
                        <div class="mb-3">
                            <button type="button" class="btn btn-primary" style="width: 100%;" onclick={on_create.clone()}>{ "追加" }</button>
                        </div>
                    }
                } }
                </div>
            </div>
        </div>
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TwinMode {
    WithRef,
}

#[derive(Properties, PartialEq)]
pub struct AccordionTwinProps<T: AllList + ShowValue, U: AllList + ShowValue> {
    pub name: &'static str,
    #[prop_or_default]
    pub lead: Option<&'static str>,
    pub mode: TwinMode,
    pub value_t: Option<T>,
    pub value_u: Option<U>,
    pub input_refs: [NodeRef; 2],
    pub on_func: Callback<Mode, Callback<MouseEvent>>
}

#[function_component(AccordionTwin)]
pub fn accordion_twin<T: AllList + ShowValue + 'static, U: AllList + ShowValue + 'static>(props: &AccordionTwinProps<T, U>) -> Html {
    let id_safe = props.name.replace(" ", "-").replace("/", "-");
    let is_open = match (&props.value_t, &props.value_u) {
        (Some(_), Some(_)) => true,
        _ => false
    };
    
    let btn_classes = classes!(
        "accordion-button",
        if !is_open { "collapsed" } else { "" },
    );

    let collapse_classes = if is_open {
        "accordion-collapse collapse show"
    } else {
        "accordion-collapse collapse"
    };

    let on_func = props.on_func.clone();
    let on_update = on_func.emit(Mode::Update);
    let on_delete = on_func.emit(Mode::Delete);
    let on_create = on_func.emit(Mode::Create);

    html! {
        <div class="accordion-item">
            <h2 class="accordion-header" id={format!("heading-{}", id_safe.clone())}>
                <button class={btn_classes} type="button"
                    data-bs-toggle="collapse"
                    data-bs-target={format!("#{}", id_safe)}
                    aria-expanded={ is_open.to_string() }
                    aria-controls={id_safe.clone()}>
                    <div class="d-flex flex-column text-start w-100">
                        <span>{ props.name }</span>
                        {
                            if let Some(lead) = &props.lead {
                                html! {
                                    <small class="text-muted">{ lead }</small>
                                }
                            } else {
                                html! {}
                            }
                        }
                    </div>
                </button>
            </h2>
            <div id={id_safe.clone()}
                class={collapse_classes}
                aria-labelledby={format!("heading-{}", id_safe.clone())}>
                <div class="accordion-body">
                { if is_open {
                    html! {
                        <>
                        {
                            match props.mode {
                                TwinMode::WithRef => html! {
                                    <WithRef<T, U>
                                        value_t={props.value_t.clone().unwrap()}
                                        value_u={props.value_u.clone().unwrap()}
                                        input_refs={[props.input_refs[0].clone(), props.input_refs[1].clone()]} />
                                },
                            }
                        }
                        <div class="d-flex justify-content-end gap-2 mb-3">
                            <button type="button" class="btn btn-primary" onclick={on_update.clone()}>{ "更新" }</button>
                            <button type="button" class="btn btn-danger" onclick={on_delete.clone()}>{ "削除" }</button>
                        </div>
                        </>
                    }
                } else {
                    html! {
                        <div class="mb-3">
                            <button type="button" class="btn btn-primary" style="width: 100%;" onclick={on_create.clone()}>{ "追加" }</button>
                        </div>
                    }
                } }
                </div>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct TwinProps<T: AllList + ShowValue, U: AllList + ShowValue> {
    value_t: T,
    value_u: U,
    input_refs: [NodeRef; 2],
}

#[function_component(WithRef)]
pub fn with_ref<T: AllList + ShowValue, U: AllList + ShowValue>(props: &TwinProps<T, U>) -> Html {
    html! {
        <>
        <div class="mb-3">
        <p>{ format!("現在：{}", props.value_u.show_value()) }</p>
        <p>{ format!("({})", props.value_t.show_value()) }</p>
        </div>
        <div class="mb-3">
        { "編集：" }
        <select ref={props.input_refs[0].clone()}>
        {
            for props.value_t.all().iter().map(|(i, en)| {
                if en == &props.value_t {
                    html! {
                        <option value={i.to_string()} selected=true>
                            { en.show_value() }
                        </option>
                    }
                } else {
                    html! {
                        <option value={i.to_string()}>
                            { en.show_value() }
                        </option>
                    }
                }
            })
        }
        </select>
        </div>
        <div class="mb-3">
        <input 
            type="text" 
            ref={props.input_refs[1].clone()}
            class="form-control" 
            value={props.value_u.show_value()} />
        </div>
        </>
    }
}
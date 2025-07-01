use chrono::{Local, NaiveDateTime, Timelike};
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

use little_exif::exif_tag::ExifTag;
use little_exif::rational::{iR64, uR64};

use crate::components::utils::ShowValue;
use crate::{
    ev, ev_time, on_string, on_int, 
    on_f64, on_enum, on_enum_u8, on_vec, 
    on_time, on_offset, on_ascii,
};
use crate::exif::exif_capture::{
    TimeOffset,
    ExposureProgram, ExposureMode, MeteringMode, LightSource, 
    Flash, SensitivityType, SensingMethod, FileSource, SceneType,
    CFA, CFAPattern, FocalPlaneResolutionUnit, WhiteBalance, SceneCaptureType,
    GainControl, Contrast, Saturation, Sharpness, CustomRendered,
    ComponentsConfiguration, CompositeImage
};
use crate::exif::rational::{approx_frac, ExifRational};
use crate::exif::utils::{AllList, F64};

use super::accordion::{
    Mode, AccordionMode, Accordion
};
use super::tabs::TabItem;
use super::utils::InfoProps;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tabs {
    TimeInfo,
    OpticInfo,
    ExposureSettings,
    SensitivityInfo,
    EncodingMetadata,
    IdentifierInfo,
    CompositeMetadata
}

#[function_component(ExifCaptureInfo)]
pub fn exif_capture_info(props: &InfoProps) -> Html {
    let selected_tab = use_state(|| Tabs::TimeInfo);
    let input_refs = [
        use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), 
        use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), 
        use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), 
        use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), 
        use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), 
        use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), 
        use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), 
    ];

    on_time!(original, DateTimeOriginal, SubSecTimeOriginal, input_refs[0], props);
    on_offset!(offset_time_original, OffsetTimeOriginal, input_refs[1], props);
    on_time!(create, CreateDate, SubSecTimeDigitized, input_refs[2], props);
    on_offset!(offset_time_digitized, OffsetTimeDigitized, input_refs[3], props);
    on_time!(modify, ModifyDate, SubSecTime, input_refs[4], props);
    on_offset!(offset_time, OffsetTime, input_refs[5], props);

    on_string!(lens_make, LensMake, input_refs[0], props);
    on_string!(lens_model, LensModel, input_refs[1], props);
    on_string!(lens_serial_number, LensSerialNumber, input_refs[2], props);
    on_f64!(uR64, max_aperture_value, MaxApertureValue, input_refs[3], props);

    on_enum!(ExposureProgram, exposure_program, ExposureProgram, input_refs[0], props);
    on_enum!(ExposureMode, exposure_mode, ExposureMode, input_refs[1], props);
    on_f64!(uR64, exposure_time, ExposureTime, input_refs[2], props);
    on_f64!(iR64, shutter_speed_value, ShutterSpeedValue, input_refs[3], props);
    on_f64!(uR64, f_number, FNumber, input_refs[4], props);
    on_f64!(uR64, aperture_value, ApertureValue, input_refs[5], props);
    on_f64!(iR64, exposure_compensation, ExposureCompensation, input_refs[6], props);
    on_f64!(iR64, brightness_value, BrightnessValue, input_refs[7], props);
    on_enum!(MeteringMode, metering_mode, MeteringMode, input_refs[8], props);
    on_enum!(LightSource, light_source, LightSource, input_refs[9], props);
    on_enum!(Flash, flash, Flash, input_refs[10], props);
    on_f64!(uR64, focal_length, FocalLength, input_refs[11], props);
    on_vec!(u16, subject_area, SubjectArea, input_refs[12], props);
    on_vec!(u16, subject_location, SubjectLocation, input_refs[13], props);

    on_enum!(SensitivityType, sensitivity_type, SensitivityType, input_refs[0], props);
    on_vec!(u16, iso, ISO, input_refs[1], props);
    on_int!(u32, iso_speed, ISOSpeed, input_refs[2], props);
    on_int!(u32, standard_output_sensitivity, StandardOutputSensitivity, input_refs[3], props);
    on_int!(u32, recommended_exposure_index, RecommendedExposureIndex, input_refs[4], props);
    on_f64!(uR64, exposure_index, ExposureIndex, input_refs[5], props);
    on_int!(u32, iso_speed_latitude_yyy, ISOSpeedLatitudeyyy, input_refs[6], props);
    on_int!(u32, iso_speed_latitude_zzz, ISOSpeedLatitudezzz, input_refs[7], props);

    on_ascii!(exif_version, ExifVersion, input_refs[0], props);
    on_ascii!(flashpix_version, FlashpixVersion, input_refs[1], props);
    on_int!(u32, exif_offset, ExifOffset, input_refs[2], props);
    let components_configuration = {
        let input_refs = [
            input_refs[3].clone(), 
            input_refs[4].clone(), 
            input_refs[5].clone(), 
            input_refs[6].clone()
        ];
        let exif = props.exif.clone();
        Callback::from(move |mode: Mode| {
            let input_refs = input_refs.clone();
            let exif = exif.clone();
            Callback::from(move |_: MouseEvent| {
                match mode {
                    Mode::Update => {
                        if let (Some(i0), Some(i1), Some(i2), Some(i3)) = (
                            input_refs[0].cast::<HtmlSelectElement>(),
                            input_refs[1].cast::<HtmlSelectElement>(),
                            input_refs[2].cast::<HtmlSelectElement>(),
                            input_refs[3].cast::<HtmlSelectElement>(),
                        ) {
                            let (Ok(v0), Ok(v1), Ok(v2), Ok(v3)) = (
                                i0.value().parse::<String>(),
                                i1.value().parse::<String>(),
                                i2.value().parse::<String>(),
                                i3.value().parse::<String>(),
                            );
                            if let Some(eed) = exif.as_ref() {
                                let mut eed = eed.clone();
                                eed.update_tag(ExifTag::ComponentsConfiguration(vec![
                                    v0.parse::<u8>().unwrap(),
                                    v1.parse::<u8>().unwrap(),
                                    v2.parse::<u8>().unwrap(),
                                    v3.parse::<u8>().unwrap(),
                                ]));
                                exif.set(Some(eed));
                            }
                        }
                    }
                    Mode::Delete => {
                        if let Some(eed) = exif.as_ref() {
                            let mut eed = eed.clone();
                            let tag = ExifTag::ComponentsConfiguration(vec![0]);
                            eed.delete_tag(tag);
                            exif.set(Some(eed));
                        }
                    }
                    Mode::Create => {
                        if let Some(eed) = exif.as_ref() {
                            let mut eed = eed.clone();
                            let tag = ExifTag::ComponentsConfiguration(vec![0, 0, 0, 0]);
                            eed.update_tag(tag);
                            exif.set(Some(eed));
                        }
                    }
                }
            })
        })
    };
    on_f64!(uR64, compressed_bits_per_pixel, CompressedBitsPerPixel, input_refs[7], props);

    on_enum!(SensingMethod, sensing_method, SensingMethod, input_refs[8], props);
    on_enum_u8!(FileSource, file_source, FileSource, input_refs[9], props);
    on_enum_u8!(SceneType, scene_type, SceneType, input_refs[10], props);
    let cfa_pattern = {
        let input_refs = [
            input_refs[11].clone(),
            input_refs[12].clone(),
            input_refs[13].clone(),
        ];
        let exif = props.exif.clone();
        Callback::from(move |mode: Mode| {
            let input_refs = input_refs.clone();
            let exif = exif.clone();
            Callback::from(move |_: MouseEvent| {
                match mode {
                    Mode::Update => {
                        if let (Some(i0), Some(i1), Some(i2)) = (
                            input_refs[0].cast::<HtmlSelectElement>(),
                            input_refs[1].cast::<HtmlSelectElement>(),
                            input_refs[2].cast::<HtmlSelectElement>(),
                        ) {
                            if let Some(eed) = exif.as_ref() {
                                let c = eed.clone().exif_capture_info.encoding_metadata.cfa_pattern.unwrap();
                                let mut eed = eed.clone();
                                
                                let mut reset_flg = false;
                                if let (Ok(row), Ok(column), Ok(cfa)) = (
                                    i0.value().parse::<u16>(),
                                    i1.value().parse::<u16>(),
                                    i2.value().parse::<String>(),
                                ) {
                                    let n_cfa = row as usize * column as usize;
                                    let cfa = cfa.split(",").filter_map(|piece| {
                                        let t = piece.trim();
                                        match t {
                                            "" => None,
                                            "Red" => Some(0),
                                            "Green" => Some(1),
                                            "Blue" => Some(2),
                                            "Cyan" => Some(3),
                                            "Magenta" => Some(4),
                                            "Yellow" => Some(5),
                                            "White" => Some(6),
                                            t => t.parse::<u8>().ok(),
                                        }
                                    }).collect::<Vec<u8>>();
                                    if cfa.len() == n_cfa {
                                        let mut v = vec![0; 4 + n_cfa];
                                        let row_bytes: [u8; 2] = row.to_be_bytes();
                                        v[0] = row_bytes[0];
                                        v[1] = row_bytes[1];
                                        let column_bytes: [u8; 2] = column.to_be_bytes();
                                        v[2] = column_bytes[0];
                                        v[3] = column_bytes[1];
                                        for i in 0..n_cfa { v[4 + i] = cfa[i]; }
                                        eed.update_tag(ExifTag::CFAPattern(v));
                                        exif.set(Some(eed));
                                    } else { reset_flg = true; }
                                } else { reset_flg = true; }
                                if reset_flg {
                                    let (row, column, cfa) = cfa_str(&c);
                                    i0.set_value(&row);
                                    i1.set_value(&column);
                                    i2.set_value(&cfa);
                                }
                            }
                        }
                    }
                    Mode::Delete => {
                        if let Some(eed) = exif.as_ref() {
                            let mut eed = eed.clone();
                            let tag = ExifTag::CFAPattern(vec![0]);
                            eed.delete_tag(tag);
                            exif.set(Some(eed));
                        }
                    }
                    Mode::Create => {
                        if let Some(eed) = exif.as_ref() {
                            let mut eed = eed.clone();
                            let tag = ExifTag::CFAPattern(vec![0, 1, 0, 1, 0]);
                            eed.update_tag(tag);
                            exif.set(Some(eed));
                        }
                    }
                }
            })
        })
    };

    on_f64!(iR64, ambient_temperature, AmbientTemperature, input_refs[14], props);
    on_f64!(uR64, humidity, Humidity, input_refs[15], props);
    on_f64!(uR64, pressure, Pressure, input_refs[16], props);
    on_f64!(iR64, water_depth, WaterDepth, input_refs[17], props);
    on_f64!(uR64, acceleration, Acceleration, input_refs[18], props);
    on_f64!(iR64, camera_elevation_angle, CameraElevationAngle, input_refs[19], props);

    on_string!(spectral_sensitivity, SpectralSensitivity, input_refs[20], props);
    on_vec!(u8, oecf, OECF, input_refs[21], props);
    on_f64!(uR64, subject_distance, SubjectDistance, input_refs[22], props);
    on_int!(u16, subject_distance_range, SubjectDistanceRange, input_refs[23], props);
    on_f64!(uR64, flash_energy, FlashEnergy, input_refs[24], props);
    on_vec!(u16, spatial_frequency_response, SpatialFrequencyResponse, input_refs[25], props);
    on_f64!(uR64, focal_plane_x_resolution, FocalPlaneXResolution, input_refs[26], props);
    on_f64!(uR64, focal_plane_y_resolution, FocalPlaneYResolution, input_refs[27], props);
    on_enum!(FocalPlaneResolutionUnit, focal_plane_resolution_unit, FocalPlaneResolutionUnit, input_refs[28], props);
    
    on_enum!(WhiteBalance, white_balance, WhiteBalance, input_refs[29], props);
    on_f64!(uR64, digital_zoom_ratio, DigitalZoomRatio, input_refs[30], props);
    on_int!(u16, focal_length_in_35mm_format, FocalLengthIn35mmFormat, input_refs[31], props);
    on_enum!(SceneCaptureType, scene_capture_type, SceneCaptureType, input_refs[32], props);
    on_enum!(GainControl, gain_control, GainControl, input_refs[33], props);
    on_enum!(Contrast, contrast, Contrast, input_refs[34], props);
    on_enum!(Saturation, saturation, Saturation, input_refs[35], props);
    on_enum!(Sharpness, sharpness, Sharpness, input_refs[36], props);
    on_enum!(CustomRendered, custom_rendered, CustomRendered, input_refs[37], props);
    on_vec!(u8, device_setting_description, DeviceSettingDescription, input_refs[38], props);
    on_f64!(uR64, gamma, Gamma, input_refs[39], props);

    on_string!(related_sound_file, RelatedSoundFile, input_refs[40], props);

    on_string!(image_unique_id, ImageUniqueID, input_refs[0], props);

    on_enum!(CompositeImage, composite_image, CompositeImage, input_refs[0], props);
    on_vec!(u16, composite_image_count, CompositeImageCount, input_refs[1], props);
    on_vec!(u8, composite_image_exposure_times, CompositeImageExposureTimes, input_refs[2], props);

    html! {
        <div>
            <ul class="nav nav-tabs flex-nowrap mb-3">
                <TabItem<Tabs> tab={Tabs::TimeInfo} selected_tab={selected_tab.clone()} message={"時間情報"} icon={"clock"} />
                <TabItem<Tabs> tab={Tabs::OpticInfo} selected_tab={selected_tab.clone()} message={"レンズ情報"} icon={"camera2"} />
                <TabItem<Tabs> tab={Tabs::ExposureSettings} selected_tab={selected_tab.clone()} message={"露光情報"} icon={"lightning"} />
                <TabItem<Tabs> tab={Tabs::SensitivityInfo} selected_tab={selected_tab.clone()} message={"感度情報"} icon={"lightbulb"} />
                <TabItem<Tabs> tab={Tabs::EncodingMetadata} selected_tab={selected_tab.clone()} message={"撮影時情報"} icon={"camera-reels"} />
                <TabItem<Tabs> tab={Tabs::IdentifierInfo} selected_tab={selected_tab.clone()} message={"特定用ID"} icon={"check-circle"} />
                <TabItem<Tabs> tab={Tabs::CompositeMetadata} selected_tab={selected_tab.clone()} message={"合成情報"} icon={"images"} />
            </ul>

            <div class="tab-content border border-top-0 p-3">
            <div class="accordion">
            {
                match *selected_tab {
                    Tabs::TimeInfo => html! {
                        <>
                        <Accordion<NaiveDateTime>
                            name={ "DateTimeOriginal / SubSecTimeOriginal" }
                            lead={Some("画像の撮影日時")}
                            mode={AccordionMode::Time}
                            input_ref={input_refs[0].clone()}
                            value={ev_time!(
                                exif_capture_info.time_info.date_time_original, 
                                exif_capture_info.time_info.sub_sec_time_original,
                                props
                            )}
                            on_func={original} />
                        <Accordion<TimeOffset>
                            name={ "OffsetTimeOriginal" }
                            lead={Some("撮影日時のタイムゾーン")}
                            mode={AccordionMode::OffsetTime}
                            input_ref={input_refs[1].clone()}
                            value={ev!(exif_capture_info.time_info.offset_time_original, props)}
                            on_func={offset_time_original} />
                        <Accordion<NaiveDateTime>
                            name={ "CreateDate / SubSecTimeDigitized" }
                            lead={Some("画像がデジタル化 (ファイルに保存) された日時")}
                            mode={AccordionMode::Time}
                            input_ref={input_refs[2].clone()}
                            value={ev_time!(
                                exif_capture_info.time_info.create_date, 
                                exif_capture_info.time_info.sub_sec_time_digitized,
                                props
                            )}
                            on_func={create} />
                        <Accordion<TimeOffset>
                            name={ "OffsetTimeDigitized" }
                            lead={Some("デジタル化日時のタイムゾーン")}
                            mode={AccordionMode::OffsetTime}
                            input_ref={input_refs[3].clone()}
                            value={ev!(exif_capture_info.time_info.offset_time_digitized, props)}
                            on_func={offset_time_digitized} />
                        <Accordion<NaiveDateTime>
                            name={ "ModifyDate / SubSecTime" }
                            lead={Some("画像ファイルの最終修正日時")}
                            mode={AccordionMode::Time}
                            input_ref={input_refs[4].clone()}
                            value={ev_time!(
                                exif_capture_info.time_info.modify_date, 
                                exif_capture_info.time_info.sub_sec_time,
                                props
                            )}
                            on_func={modify} />
                        <Accordion<TimeOffset>
                            name={ "OffsetTime" }
                            lead={Some("最終日時のタイムゾーン")}
                            mode={AccordionMode::OffsetTime}
                            input_ref={input_refs[5].clone()}
                            value={ev!(exif_capture_info.time_info.offset_time, props)}
                            on_func={offset_time} />
                        </>
                    },
                    Tabs::OpticInfo => html! {
                        <>
                        <Accordion<String>
                            name={ "LensMake" }
                            lead={Some("レンズ製造元")}
                            input_ref={input_refs[0].clone()} 
                            value={ev!(exif_capture_info.optic_info.lens_make, props)} 
                            on_func={lens_make} />
                        <Accordion<String>
                            name={ "LensModel" }
                            lead={Some("レンズ製品名 (型番)")}
                            input_ref={input_refs[1].clone()}
                            value={ev!(exif_capture_info.optic_info.lens_model, props)}
                            on_func={lens_model} />
                        <Accordion<String>
                            name={ "LensSerialNumber" }
                            lead={Some("レンズの個体識別番号")}
                            input_ref={input_refs[2].clone()}
                            value={ev!(exif_capture_info.optic_info.lens_serial_number, props)}
                            on_func={lens_serial_number} />
                        <Accordion<F64>
                            name={ "MaxApertureValue" }
                            lead={Some("レンズの光学仕様 (レンズの最大開放F値 = 最小のFNumber)")}
                            input_ref={input_refs[3].clone()}
                            value={ev!(exif_capture_info.optic_info.max_aperture_value, props)}
                            on_func={max_aperture_value} />
                        </>
                    },
                    Tabs::ExposureSettings => html! {
                        <>
                        <Accordion<ExposureProgram>
                            name={ "ExposureProgram" }
                            lead={Some("露出プログラムの種類")}
                            mode={AccordionMode::Dropdown}
                            input_ref={input_refs[0].clone()}
                            value={ev!(exif_capture_info.exposure_settings.exposure_program, props)}
                            on_func={exposure_program}
                            />
                        <Accordion<ExposureMode>
                            name={ "ExposureMode" }
                            lead={Some("実際の撮影者の操作方法")}
                            mode={AccordionMode::Dropdown}
                            input_ref={input_refs[1].clone()}
                            value={ev!(exif_capture_info.exposure_settings.exposure_mode, props)}
                            on_func={exposure_mode} />
                        <Accordion<F64>
                            name={ "ExposureTime" }
                            lead={Some("シャッター開放時間 (秒)")}
                            input_ref={input_refs[2].clone()}
                            value={ev!(exif_capture_info.exposure_settings.exposure_time, props)}
                            on_func={exposure_time} />
                        <Accordion<F64>
                            name={ "ShutterSpeedValue" }
                            lead={Some("シャッター速度のLog2表現 (Apex値) = -log2(ExposureTime)")}
                            input_ref={input_refs[3].clone()}
                            value={ev!(exif_capture_info.exposure_settings.shutter_speed_value, props)}
                            on_func={shutter_speed_value} />
                        <Accordion<F64>
                            name={ "FNumber" }
                            lead={Some("絞り値")}
                            input_ref={input_refs[4].clone()}
                            value={ev!(exif_capture_info.exposure_settings.f_number, props)}
                            on_func={f_number} />
                        <Accordion<F64>
                            name={ "ApertureValue" }
                            lead={Some("絞り値のLog2表現 (Apex値) = 2 × log2(FNumber)")}
                            input_ref={input_refs[5].clone()}
                            value={ev!(exif_capture_info.exposure_settings.aperture_value, props)}
                            on_func={aperture_value} />
                        <Accordion<F64>
                            name={ "ExposureCompensation" }
                            lead={Some("カメラが意図的に露出を+/-補正した量 (Apex値)")}
                            input_ref={input_refs[6].clone()}
                            value={ev!(exif_capture_info.exposure_settings.exposure_compensation, props)}
                            on_func={exposure_compensation} />
                        <Accordion<F64>
                            name={ "brightness_value" }
                            lead={Some("被写体の平均輝度 (Apex値, 推定値)")}
                            input_ref={input_refs[7].clone()}
                            value={ev!(exif_capture_info.exposure_settings.brightness_value, props)}
                            on_func={brightness_value} />
                        <Accordion<MeteringMode>
                            name={ "MeteringMode" }
                            lead={Some("露出計測の方式")}
                            mode={AccordionMode::Dropdown}
                            input_ref={input_refs[8].clone()}
                            value={ev!(exif_capture_info.exposure_settings.metering_mode, props)}
                            on_func={metering_mode} />
                        <Accordion<LightSource>
                            name={ "LightSource" }
                            lead={Some("撮影時の光源タイプ")}
                            mode={AccordionMode::Dropdown}
                            input_ref={input_refs[9].clone()}
                            value={ev!(exif_capture_info.exposure_settings.light_source, props)}
                            on_func={light_source} />
                        <Accordion<Flash>
                            name={ "Flash" }
                            lead={Some("フラッシュの発光状況")}
                            mode={AccordionMode::Dropdown}
                            input_ref={input_refs[10].clone()}
                            value={ev!(exif_capture_info.exposure_settings.flash, props)}
                            on_func={flash} />
                        <Accordion<F64>
                            name={ "FocalLength" }
                            lead={Some("レンズの焦点距離 (mm)")}
                            input_ref={input_refs[11].clone()}
                            value={ev!(exif_capture_info.exposure_settings.focal_length, props)}
                            on_func={focal_length} />
                        <Accordion<Vec<u16>>
                            name={ "SubjectArea" }
                            lead={Some("フォーカスされた領域の位置とサイズ: 長さ 2 or 3 or 4")}
                            input_ref={input_refs[12].clone()}
                            value={ev!(exif_capture_info.exposure_settings.subject_area, props)}
                            on_func={subject_area}
                            caution=true />
                        <Accordion<[u16; 2]>
                            name={ "SubjectLocation" }
                            lead={Some("ピントが合った被写体の中心座標 (2D): 長さ 2")}
                            input_ref={input_refs[13].clone()}
                            value={ev!(exif_capture_info.exposure_settings.subject_location, props)}
                            on_func={subject_location} />
                        </>
                    },
                    Tabs::SensitivityInfo => html! {
                        <>
                        <Accordion<SensitivityType>
                            name={ "SensitivityType" }
                            lead={Some("ISO系タグの選択ルール")}
                            mode={AccordionMode::Dropdown}
                            input_ref={input_refs[0].clone()}
                            value={ev!(exif_capture_info.sensitivity_info.sensitivity_type, props)}
                            on_func={sensitivity_type} />
                        <Accordion<Vec<u16>>
                            name={ "ISO" }
                            lead={Some("カメラが設定したISO感度 (Exif 2.2以前で主流)")}
                            input_ref={input_refs[1].clone()}
                            value={ev!(exif_capture_info.sensitivity_info.iso, props)}
                            on_func={iso}
                            caution=true />
                        <Accordion<u32>
                            name={ "ISOSpeed" }
                            lead={Some("カメラが設定したISO感度 (Exif 2.3以降で主流)")}
                            input_ref={input_refs[2].clone()}
                            value={ev!(exif_capture_info.sensitivity_info.iso_speed, props)}
                            on_func={iso_speed} />
                        <Accordion<u32>
                            name={ "StandardOutputSensitivity" }
                            lead={Some("標準出力感度")}
                            input_ref={input_refs[3].clone()}
                            value={ev!(exif_capture_info.sensitivity_info.standard_output_sensitivity, props)}
                            on_func={standard_output_sensitivity} />
                        <Accordion<u32>
                            name={ "RecommendedExposureIndex" }
                            lead={Some("推奨露出指数 (REI)")}
                            input_ref={input_refs[4].clone()}
                            value={ev!(exif_capture_info.sensitivity_info.recommended_exposure_index, props)}
                            on_func={recommended_exposure_index} />
                        <Accordion<F64>
                            name={ "ExposureIndex" }
                            lead={Some("実際に使用された感度指数")}
                            input_ref={input_refs[5].clone()}
                            value={ev!(exif_capture_info.sensitivity_info.exposure_index, props)}
                            on_func={exposure_index} />
                        <Accordion<u32>
                            name={ "ISOSpeedLatitudeyyy" }
                            lead={Some("フィルムにおける露光許容範囲の「下限」感度")}
                            input_ref={input_refs[6].clone()}
                            value={ev!(exif_capture_info.sensitivity_info.iso_speed_latitude_yyy, props)}
                            on_func={iso_speed_latitude_yyy} />
                        <Accordion<u32>
                            name={ "ISOSpeedLatitudezzz" }
                            lead={Some("フィルムにおける露光許容範囲の「上限」感度")}
                            input_ref={input_refs[7].clone()}
                            value={ev!(exif_capture_info.sensitivity_info.iso_speed_latitude_zzz, props)}
                            on_func={iso_speed_latitude_zzz} />
                        </>
                    },
                    Tabs::EncodingMetadata => html! {
                        <>
                        <Accordion<String>
                            name={ "ExifVersion" }
                            lead={Some("Exif仕様のバージョン: 長さ 4")}
                            input_ref={input_refs[0].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.exif_version, props)}
                            on_func={exif_version} />
                        <Accordion<String>
                            name={ "FlashpixVersion" }
                            lead={Some("Flashpix規格バージョン: 長さ 4")}
                            input_ref={input_refs[1].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.flashpix_version, props)}
                            on_func={flashpix_version} />
                        <Accordion<u32>
                            name={ "ExifOffset" }
                            lead={Some("Exif IFD (画像情報) へのポインタ")}
                            input_ref={input_refs[2].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.exif_offset, props)}
                            on_func={exif_offset}
                            caution=true />
                        <AccordionComponentsConfiguration
                            input_refs={[
                                input_refs[3].clone(),
                                input_refs[4].clone(),
                                input_refs[5].clone(),
                                input_refs[6].clone(),
                            ]}
                            value={ev!(exif_capture_info.encoding_metadata.components_configuration, props)}
                            on_func={components_configuration} />
                        <Accordion<F64>
                            name={ "CompressedBitsPerPixel" }
                            lead={Some("圧縮された1ピクセルあたりの平均ビット数")}
                            input_ref={input_refs[7].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.compressed_bits_per_pixel, props)}
                            on_func={compressed_bits_per_pixel} />

                        <Accordion<SensingMethod>
                            name={ "SensingMethod" }
                            lead={Some("撮像方式")}
                            mode={AccordionMode::Dropdown}
                            input_ref={input_refs[8].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.sensing_method, props)}
                            on_func={sensing_method} />
                        <Accordion<FileSource>
                            name={ "FileSource" }
                            lead={Some("ファイルの生成元")}
                            mode={AccordionMode::Dropdown}
                            input_ref={input_refs[9].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.file_source, props)}
                            on_func={file_source} />
                        <Accordion<SceneType>
                            name={ "SceneType" }
                            lead={Some("どのような方法で画像が生成されたか")}
                            mode={AccordionMode::Dropdown}
                            input_ref={input_refs[10].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.scene_type, props)}
                            on_func={scene_type} />
                        <AccordionCFAPattern
                            input_refs={[
                                input_refs[11].clone(),
                                input_refs[12].clone(),
                                input_refs[13].clone(),
                            ]}
                            value={ev!(exif_capture_info.encoding_metadata.cfa_pattern, props)}
                            on_func={cfa_pattern} />

                        <Accordion<F64>
                            name={ "AmbientTemperature" }
                            lead={Some("撮影時の気温 (℃)")}
                            input_ref={input_refs[14].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.ambient_temperature, props)}
                            on_func={ambient_temperature} />
                        <Accordion<F64>
                            name={ "Humidity" }
                            lead={Some("撮影時の湿度 (%)")}
                            input_ref={input_refs[15].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.humidity, props)}
                            on_func={humidity} />
                        <Accordion<F64>
                            name={ "Pressure" }
                            lead={Some("撮影時の気圧 (hPa)")}
                            input_ref={input_refs[16].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.pressure, props)}
                            on_func={pressure} />
                        <Accordion<F64>
                            name={ "WaterDepth" }
                            lead={Some("撮影時の水深 (メートル: 水中撮影など)")}
                            input_ref={input_refs[17].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.water_depth, props)}
                            on_func={water_depth} />
                        <Accordion<F64>
                            name={ "Acceleration" }
                            lead={Some("撮影時の加速度 (車載カメラなど)")}
                            input_ref={input_refs[18].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.acceleration, props)}
                            on_func={acceleration} />
                        <Accordion<F64>
                            name={ "CameraElevationAngle" }
                            lead={Some("カメラの仰角 (水平基準の角度)")}
                            input_ref={input_refs[19].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.camera_elevation_angle, props)}
                            on_func={camera_elevation_angle} />

                        <Accordion<String>
                            name={ "SpectralSensitivity" }
                            lead={Some("撮影素子の分光感度特性")}
                            input_ref={input_refs[20].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.spectral_sensitivity, props)}
                            on_func={spectral_sensitivity} />
                        <Accordion<Vec<u8>>
                            name={ "OECF" }
                            lead={Some("入力→出力の変換特性 (センサの直線性)")}
                            input_ref={input_refs[21].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.oecf, props)}
                            on_func={oecf} />
                        <Accordion<F64>
                            name={ "SubjectDistance" }
                            lead={Some("被写体までの距離 (メートル)")}
                            input_ref={input_refs[22].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.subject_distance, props)}
                            on_func={subject_distance} />
                        <Accordion<u16>
                            name={ "SubjectDistanceRange" }
                            lead={Some("被写体の距離カテゴリ")}
                            input_ref={input_refs[23].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.subject_distance_range, props)}
                            on_func={subject_distance_range} />
                        <Accordion<F64>
                            name={ "FlashEnergy" }
                            lead={Some("フラッシュの発光エネルギー")}
                            input_ref={input_refs[24].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.flash_energy, props)}
                            on_func={flash_energy} />
                        <Accordion<Vec<u16>>
                            name={ "SpatialFrequencyResponse" }
                            lead={Some("シャープネス指標")}
                            input_ref={input_refs[25].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.spatial_frequency_response, props)}
                            on_func={spatial_frequency_response} />

                        <Accordion<F64>
                            name={ "FocalPlaneXResolution" }
                            lead={Some("撮像素子上の水平方向の解像度")}
                            input_ref={input_refs[26].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.focal_plane_x_resolution, props)}
                            on_func={focal_plane_x_resolution} />
                        <Accordion<F64>
                            name={ "FocalPlaneYResolution" }
                            lead={Some("撮像素子上の垂直方向の解像度")}
                            input_ref={input_refs[27].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.focal_plane_y_resolution, props)}
                            on_func={focal_plane_y_resolution} />
                        <Accordion<FocalPlaneResolutionUnit>
                            name={ "FocalPlaneResolutionUnit" }
                            lead={Some("撮像素子上の解像度の単位")}
                            mode={AccordionMode::Dropdown}
                            input_ref={input_refs[28].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.focal_plane_resolution_unit, props)}
                            on_func={focal_plane_resolution_unit} />

                        <Accordion<WhiteBalance>
                            name={ "WhiteBalance" }
                            lead={Some("ホワイトバランス")}
                            mode={AccordionMode::Dropdown}
                            input_ref={input_refs[29].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.white_balance, props)}
                            on_func={white_balance} />
                        <Accordion<F64>
                            name={ "DigitalZoomRatio" }
                            lead={Some("デジタルズーム倍率")}
                            input_ref={input_refs[30].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.digital_zoom_ratio, props)}
                            on_func={digital_zoom_ratio} />
                        <Accordion<u16>
                            name={ "FocalLengthIn35mmFormat" }
                            lead={Some("35mm換算焦点距離 (mm)")}
                            input_ref={input_refs[31].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.focal_length_in_35mm_format, props)}
                            on_func={focal_length_in_35mm_format} />
                        <Accordion<SceneCaptureType>
                            name={ "SceneCaptureType" }
                            lead={Some("撮影シーン")}
                            mode={AccordionMode::Dropdown}
                            input_ref={input_refs[32].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.scene_capture_type, props)}
                            on_func={scene_capture_type} />
                        <Accordion<GainControl>
                            name={ "GainControl" }
                            lead={Some("ゲイン調整")}
                            mode={AccordionMode::Dropdown}
                            input_ref={input_refs[33].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.gain_control, props)}
                            on_func={gain_control} />
                        <Accordion<Contrast>
                            name={ "Contrast" }
                            lead={Some("画像のコントラスト設定")}
                            mode={AccordionMode::Dropdown}
                            input_ref={input_refs[34].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.contrast, props)}
                            on_func={contrast} />
                        <Accordion<Saturation>
                            name={ "Saturation" }
                            lead={Some("彩度設定")}
                            mode={AccordionMode::Dropdown}
                            input_ref={input_refs[35].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.saturation, props)}
                            on_func={saturation} />
                        <Accordion<Sharpness>
                            name={ "Sharpness" }
                            lead={Some("シャープネス設定")}
                            mode={AccordionMode::Dropdown}
                            input_ref={input_refs[36].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.sharpness, props)}
                            on_func={sharpness} />
                        <Accordion<CustomRendered>
                            name={ "CustomRendered" }
                            lead={Some("カスタム画像処理の有無 (ソフト補正など)")}
                            mode={AccordionMode::Dropdown}
                            input_ref={input_refs[37].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.custom_rendered, props)}
                            on_func={custom_rendered} />
                        <Accordion<Vec<u8>>
                            name={ "DeviceSettingDescription" }
                            lead={Some("構造化されたカメラ設定")}
                            input_ref={input_refs[38].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.device_setting_description, props)}
                            on_func={device_setting_description} />
                        <Accordion<F64>
                            name={ "Gamma" }
                            lead={Some("ガンマ補正値")}
                            input_ref={input_refs[39].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.gamma, props)}
                            on_func={gamma} />

                        <Accordion<String>
                            name={ "RelatedSoundFile" }
                            lead={Some("関連する音声ファイル名 (撮影時の音声メモなど)")}
                            input_ref={input_refs[40].clone()}
                            value={ev!(exif_capture_info.encoding_metadata.related_sound_file, props)}
                            on_func={related_sound_file} />
                        </>
                    },
                    Tabs::IdentifierInfo => html! {
                        <>
                        <Accordion<String>
                            name={ "ImageUniqueID" }
                            lead={Some("画像ファイルの識別子 (ID)")}
                            input_ref={input_refs[0].clone()}
                            value={ev!(exif_capture_info.identifier_info.image_unique_id, props)}
                            on_func={image_unique_id} />
                        </>
                    },
                    Tabs::CompositeMetadata => html! {
                        <>
                        <Accordion<CompositeImage>
                            name={ "CompositeImage" }
                            lead={Some("この画像が複数画像の合成 (合成写真) であるかどうか")}
                            mode={AccordionMode::Dropdown}
                            input_ref={input_refs[0].clone()}
                            value={ev!(exif_capture_info.composite_metadata.composite_image, props)}
                            on_func={composite_image} />
                        <Accordion<[u16; 2]>
                            name={ "CompositeImageCount" }
                            lead={Some("何枚の画像から合成されたか")}
                            input_ref={input_refs[1].clone()}
                            value={ev!(exif_capture_info.composite_metadata.composite_image_count, props)}
                            on_func={composite_image_count} />
                        <Accordion<Vec<u8>>
                            name={ "CompositeImageExposureTimes" }
                            lead={Some("合成元となった各画像の露出時間一覧: 長さ CompositeImageCount")}
                            input_ref={input_refs[2].clone()}
                            value={ev!(exif_capture_info.composite_metadata.composite_image_exposure_times, props)}
                            on_func={composite_image_exposure_times} />
                        </>
                    },
                }
            }
            </div>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct AccordionComponentsConfigurationProps {
    pub value: Option<[ComponentsConfiguration; 4]>,
    pub input_refs: [NodeRef; 4],
    pub on_func: Callback<Mode, Callback<MouseEvent>>
}

#[function_component(AccordionComponentsConfiguration)]
pub fn accordion_components_configuration(props: &AccordionComponentsConfigurationProps) -> Html {
    let (is_open, value) = match &props.value {
        Some(v) => (true, v.clone()),
        None => (false, [
            ComponentsConfiguration::Unused,
            ComponentsConfiguration::Unused,
            ComponentsConfiguration::Unused,
            ComponentsConfiguration::Unused,
        ])
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
            <h2 class="accordion-header" id={"heading-ComponentsConfiguration"}>
                <button class={btn_classes} type="button"
                    data-bs-toggle="collapse"
                    data-bs-target={"#ComponentsConfiguration"}
                    aria-expanded={ is_open.to_string() }
                    aria-controls={"ComponentsConfiguration"}>
                    <div class="d-flex flex-column text-start w-100">
                        <span>{ "ComponentsConfiguration" }</span>
                        <small class="text-muted">{ "RGB/BGRなどのカラーチャネルの順序: 長さ 4" }</small>
                    </div>
                    { "ComponentsConfiguration" }
                </button>
            </h2>
            <div id={"ComponentsConfiguration"}
                class={collapse_classes}
                aria-labelledby={"heading-ComponentsConfiguration"}>
                <div class="accordion-body">
                { if is_open {
                    html! {
                        <>
                        <div class="mb-3">
                        { format!(
                            "現在：{}, {}, {}, {}",
                            value[0].to_string(),
                            value[1].to_string(),
                            value[2].to_string(),
                            value[3].to_string(),
                        ) }
                        </div>
                        <div class="mb-3">
                        <p>{ "選択：" }</p>
                        {
                            for value.iter().enumerate().map(|(i, vi)| {
                                html! {
                                    <p><select ref={props.input_refs[i].clone()}>
                                    {
                                        for vi.all().iter().map(|(j, en)| {
                                            if en == vi {
                                                html! {
                                                    <option value={j.to_string()} selected=true>
                                                        { en.show_value() }
                                                    </option>
                                                }
                                            } else {
                                                html! {
                                                    <option value={j.to_string()}>
                                                        { en.show_value() }
                                                    </option>
                                                }
                                            }
                                        })
                                    }
                                    </select></p>
                                }
                            })
                        }
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

fn cfa_str(c: &CFAPattern) -> (String, String, String) {
    (
        c.row.to_string(), 
        c.column.to_string(),
        c.cfa.iter().map(|ci| match ci {
            CFA::Red => "Red".to_string(),
            CFA::Green => "Green".to_string(),
            CFA::Blue => "Blue".to_string(),
            CFA::Cyan => "Cyan".to_string(),
            CFA::Magenta => "Magenta".to_string(),
            CFA::Yellow => "Yellow".to_string(),
            CFA::White => "White".to_string(),
            CFA::UnknownValue(v) => v[0].to_string()
        }).collect::<Vec<String>>().join(", ")
    )
}

#[derive(Properties, PartialEq)]
pub struct AccordionCFAPatternProps {
    pub value: Option<CFAPattern>,
    pub input_refs: [NodeRef; 3],
    pub on_func: Callback<Mode, Callback<MouseEvent>>
}

#[function_component(AccordionCFAPattern)]
pub fn accordion_cfa_pattern(props: &AccordionCFAPatternProps) -> Html {
    let (is_open, value) = match &props.value {
        Some(c) => (true, cfa_str(c)),
        None => (false, ("".to_string(), "".to_string(), "".to_string()))
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
            <h2 class="accordion-header" id={"heading-CFAPattern"}>
                <button class={btn_classes} type="button"
                    data-bs-toggle="collapse"
                    data-bs-target={"#CFAPattern"}
                    aria-expanded={ is_open.to_string() }
                    aria-controls={"CFAPattern"}>
                    <div class="d-flex flex-column text-start w-100">
                        <span>{ "CFAPattern" }</span>
                        <small class="text-muted">{ "ベイヤー配列などのカラー配列パターン" }</small>
                    </div>
                </button>
            </h2>
            <div id={"CFAPattern"}
                class={collapse_classes}
                aria-labelledby={"heading-CFAPattern"}>
                <div class="accordion-body">
                { if is_open {
                    html! {
                        <>
                        <div class="mb-3">
                            <p>{format!("{} {} {}", value.0.clone(), value.1.clone(), value.2.clone())}</p>
                            <p>{ "行数" }</p>
                            <input 
                                type="text" 
                                ref={props.input_refs[0].clone()}
                                class="form-control" 
                                value={value.0} />
                            <p>{ "列数" }</p>
                            <input 
                                type="text" 
                                ref={props.input_refs[1].clone()}
                                class="form-control" 
                                value={value.1} />
                            <p>{ "CFAパターン" }</p>
                            <input 
                                type="text" 
                                ref={props.input_refs[2].clone()}
                                class="form-control" 
                                value={value.2} />
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
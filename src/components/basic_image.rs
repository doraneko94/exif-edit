use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

use little_exif::exif_tag::ExifTag;
use little_exif::rational::uR64;

use crate::{
    ev, on_string, on_int, on_int_ref,
    on_f64, on_enum, on_vec, on_f64_vec
};
use crate::exif::rational::{approx_frac, ExifRational};
use crate::exif::utils::{AllList, F64};
use crate::exif::basic_image::{
    ResolutionUnit, Compression, PhotometricInterpretation, ColorSpace,
    PlanarConfiguration, YCbCrSubSampling, YCbCrPositioning, Orientation
};

use super::accordion::{
    Mode, AccordionMode, Accordion
};
use super::tabs::TabItem;
use super::utils::{InfoProps, ShowValue};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tabs {
    DeviceModel,
    ImageFormat,
}

#[function_component(BasicImageInfo)]
pub fn basic_image_info(props: &InfoProps) -> Html {
    let selected_tab = use_state(|| Tabs::DeviceModel);
    let input_refs = [
        use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(),
        use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(),
        use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(),
        use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(),
        use_node_ref(), use_node_ref(),
    ];

    on_string!(make, Make, input_refs[0], props);
    on_string!(model, Model, input_refs[1], props);
    on_string!(software, Software, input_refs[2], props);

    on_string!(serial_number, SerialNumber, input_refs[3], props);
    on_string!(owner_name, OwnerName, input_refs[4], props);
    on_f64_vec!(uR64, lens_info, LensInfo, input_refs[5], props);

    on_int!(u32, image_width, ImageWidth, input_refs[0], props);
    on_int!(u32, image_height, ImageHeight, input_refs[1], props);
    on_int_ref!(u32, u16, exif_image_width, ExifImageWidth, input_refs[2], props);
    on_int_ref!(u32, u16, exif_image_height, ExifImageHeight, input_refs[3], props);

    on_f64!(uR64, x_resolution, XResolution, input_refs[4], props);
    on_f64!(uR64, y_resolution, YResolution, input_refs[5], props);
    on_enum!(ResolutionUnit, resolution_unit, ResolutionUnit, input_refs[6], props);

    on_enum!(Compression, compression, Compression, input_refs[7], props);
    on_enum!(PhotometricInterpretation, photometric_interpretation, PhotometricInterpretation, input_refs[8], props);
    on_enum!(ColorSpace, color_space, ColorSpace, input_refs[9], props);

    on_vec!(u16, bits_per_sample, BitsPerSample, input_refs[10], props);
    on_int!(u16, samples_per_pixel, SamplesPerPixel, input_refs[11], props);
    on_enum!(PlanarConfiguration, planar_configuration, PlanarConfiguration, input_refs[12], props);
    let ycbcr_sub_sampling = {
        let input_ref = input_refs[13].clone();
        let exif = props.exif.clone();
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
                                eed.update_tag(ExifTag::YCbCrSubSampling(
                                    YCbCrSubSampling::from_u16(value.parse::<u16>().unwrap()).to_vec()
                                ));
                                exif.set(Some(eed));
                            }
                        }
                    }
                    Mode::Delete => {
                        if let Some(eed) = exif.as_ref() {
                            let mut eed = eed.clone();
                            let tag = ExifTag::YCbCrSubSampling(Vec::new());
                            eed.delete_tag(tag);
                            exif.set(Some(eed));
                        }
                    }
                    Mode::Create => {
                        if let Some(eed) = exif.as_ref() {
                            let mut eed = eed.clone();
                            let tag = ExifTag::YCbCrSubSampling(vec![0, 0]);
                            eed.update_tag(tag);
                            exif.set(Some(eed));
                        }
                    }
                }
            })
        })
    };
    on_enum!(YCbCrPositioning, ycbcr_positioning, YCbCrPositioning, input_refs[14], props);
    on_f64_vec!(uR64, ycbcr_coefficients, YCbCrCoefficients, input_refs[15], props);

    on_vec!(u16, transfer_function, TransferFunction, input_refs[16], props);
    on_f64_vec!(uR64, white_point, WhitePoint, input_refs[17], props);
    on_f64_vec!(uR64, primary_chromaticities, PrimaryChromaticities, input_refs[18], props);
    on_f64_vec!(uR64, reference_black_white, ReferenceBlackWhite, input_refs[19], props);
    on_vec!(u16, color_map, ColorMap, input_refs[20], props);

    //strip_offsets
    on_vec!(u32, strip_byte_counts, StripByteCounts, input_refs[21], props);
    on_int!(u32, rows_per_strip, RowsPerStrip, input_refs[22], props);

    on_enum!(Orientation, orientation, Orientation, input_refs[23], props);
    on_int!(u16, cell_width, CellWidth, input_refs[24], props);
    on_int!(u16, cell_height, CellHeight, input_refs[25], props);

    html! {
        <div>
            <ul class="nav nav-tabs flex-nowrap mb-3">
                <TabItem<Tabs> tab={Tabs::DeviceModel} selected_tab={selected_tab.clone()} message={"デバイス情報"} icon={"camera"} />
                <TabItem<Tabs> tab={Tabs::ImageFormat} selected_tab={selected_tab.clone()} message={"フォーマット情報"} icon={"file-earmark-image"} />
            </ul>

            <div class="tab-content border border-top-0 p-3">
            <div class="accordion">
            {
                match *selected_tab {
                    Tabs::DeviceModel => html! {
                        <>
                        <Accordion<String> 
                            name={ "Make" } 
                            lead={Some("カメラ本体の製造元")}
                            input_ref={input_refs[0].clone()} 
                            value={ev!(basic_image_info.device_model.make, props)} 
                            on_func={make} />
                        <Accordion<String> 
                            name={ "Model" } 
                            lead={Some("カメラ本体のモデル名")}
                            input_ref={input_refs[1].clone()} 
                            value={ev!(basic_image_info.device_model.model, props)} 
                            on_func={model} />
                        <Accordion<String> 
                            name={ "Software" } 
                            lead={Some("撮影画像に使用されたソフトウェア (ファームウェアや編集ソフト)")}
                            input_ref={input_refs[2].clone()} 
                            value={ev!(basic_image_info.device_model.software, props)} 
                            on_func={software} />
                        <Accordion<String>
                            name={ "SerialNumber" }
                            lead={Some("カメラ本体の固有ID")} 
                            input_ref={input_refs[3].clone()}
                            value={ev!(basic_image_info.device_info.serial_number, props)}
                            on_func={serial_number} />
                        <Accordion<String>
                            name={ "OwnerName" }
                            lead={Some("カメラ本体の所有者")} 
                            input_ref={input_refs[4].clone()}
                            value={ev!(basic_image_info.device_info.owner_name, props)}
                            on_func={owner_name} />
                        <Accordion<[F64; 4]>
                            name={ "LensInfo" }
                            lead={Some("装着レンズの仕様: 長さ 4")} 
                            input_ref={input_refs[5].clone()}
                            value={ev!(basic_image_info.device_info.lens_info, props)}
                            on_func={lens_info} />
                        </>
                    },
                    Tabs::ImageFormat => html! {
                        <>
                        <Accordion<u32> 
                            name={ "ImageWidth" } 
                            lead={Some("現在の画像の見た目の幅")}
                            input_ref={input_refs[0].clone()} 
                            value={ev!(basic_image_info.image_format.image_width, props)} 
                            on_func={image_width} 
                            caution=true />
                        <Accordion<u32> 
                            name={ "ImageHeight" } 
                            lead={Some("現在の画像の見た目の高さ")} 
                            input_ref={input_refs[1].clone()} 
                            value={ev!(basic_image_info.image_format.image_height, props)} 
                            on_func={image_height} 
                            caution=true />
                        <Accordion<u16> 
                            name={ "ExifImageWidth" } 
                            lead={Some("撮影時のオリジナルの画像の幅")} 
                            input_ref={input_refs[2].clone()} 
                            value={ev!(basic_image_info.image_format.exif_image_width, props)} 
                            on_func={exif_image_width} 
                            caution=true />
                        <Accordion<u16> 
                            name={ "ExifImageHeight" } 
                            lead={Some("撮影時のオリジナルの画像の高さ")} 
                            input_ref={input_refs[3].clone()} 
                            value={ev!(basic_image_info.image_format.exif_image_height, props)} 
                            on_func={exif_image_height} 
                            caution=true />

                        <Accordion<F64>
                            name={ "XResolution" }
                            lead={Some("X軸解像度")} 
                            input_ref={input_refs[4].clone()}
                            value={ev!(basic_image_info.image_format.x_resolution, props)}
                            on_func={x_resolution}
                            caution=true />
                        <Accordion<F64>
                            name={ "YResolution" }
                            lead={Some("Y軸解像度")} 
                            input_ref={input_refs[5].clone()}
                            value={ev!(basic_image_info.image_format.y_resolution, props)}
                            on_func={y_resolution}
                            caution=true />
                        <Accordion<ResolutionUnit>
                            name={ "ResolutionUnit" }
                            lead={Some("解像度の単位")} 
                            mode={AccordionMode::Dropdown}
                            input_ref={input_refs[6].clone()}
                            value={ev!(basic_image_info.image_format.resolution_unit, props)}
                            on_func={resolution_unit}
                            caution=true />

                        <Accordion<Compression>
                            name={ "Compression" }
                            lead={Some("データ圧縮形式")} 
                            mode={AccordionMode::Dropdown}
                            input_ref={input_refs[7].clone()}
                            value={ev!(basic_image_info.image_format.compression, props)}
                            on_func={compression}
                            caution=true />
                        <Accordion<PhotometricInterpretation>
                            name={ "PhotometricInterpretation" }
                            lead={Some("カラーモデル")} 
                            mode={AccordionMode::Dropdown}
                            input_ref={input_refs[8].clone()}
                            value={ev!(basic_image_info.image_format.photometric_interpretation, props)}
                            on_func={photometric_interpretation}
                            caution=true />
                        <Accordion<ColorSpace>
                            name={ "ColorSpace" }
                            lead={Some("色空間")} 
                            mode={AccordionMode::Dropdown}
                            input_ref={input_refs[9].clone()}
                            value={ev!(basic_image_info.image_format.color_space, props)}
                            on_func={color_space}
                            caution=true />

                        <Accordion<Vec<u16>>
                            name={ "BitsPerSample" }
                            lead={Some("各色チャネルあたりのビット深度 (SamplesPerPixelの長さに相当)")} 
                            input_ref={input_refs[10].clone()}
                            value={ev!(basic_image_info.image_format.bits_per_sample, props)}
                            on_func={bits_per_sample}
                            caution=true />
                        <Accordion<u16>
                            name={ "SamplesPerPixel" }
                            lead={Some("ピクセル当たりのチャネル数")} 
                            input_ref={input_refs[11].clone()}
                            value={ev!(basic_image_info.image_format.samples_per_pixel, props)}
                            on_func={samples_per_pixel}
                            caution=true />
                        <Accordion<PlanarConfiguration>
                            name={ "PlanarConfiguration" }
                            lead={Some("チャネルの配置形式")} 
                            mode={AccordionMode::Dropdown}
                            input_ref={input_refs[12].clone()}
                            value={ev!(basic_image_info.image_format.planar_configuration, props)}
                            on_func={planar_configuration}
                            caution=true />
                        <Accordion<YCbCrSubSampling>
                            name={ "YCbCrSubSampling" }
                            lead={Some("サブサンプリング方式")} 
                            mode={AccordionMode::Dropdown}
                            input_ref={input_refs[13].clone()}
                            value={ev!(basic_image_info.image_format.ycbcr_sub_sampling, props)}
                            on_func={ycbcr_sub_sampling}
                            caution=true />
                        <Accordion<YCbCrPositioning>
                            name={ "YCbCrPositioning" }
                            lead={Some("サブサンプリングされた成分の位置づけ")} 
                            mode={AccordionMode::Dropdown}
                            input_ref={input_refs[14].clone()}
                            value={ev!(basic_image_info.image_format.ycbcr_positioning, props)}
                            on_func={ycbcr_positioning}
                            caution=true />
                        <Accordion<[F64; 3]>
                            name={ "YCbCrCoefficients" }
                            lead={Some("RGB→YCbCr変換の係数: 3次元ベクトル")} 
                            input_ref={input_refs[15].clone()}
                            value={ev!(basic_image_info.image_format.ycbcr_coefficients, props)}
                            on_func={ycbcr_coefficients}
                            caution=true />

                        <Accordion<[u16; 768]>
                            name={ "TransferFunction" }
                            lead={Some("トーン再現カーブ (LUT): 長さ 768")} 
                            input_ref={input_refs[16].clone()}
                            value={ev!(basic_image_info.image_format.transfer_function, props)}
                            on_func={transfer_function}
                            caution=true />
                        <Accordion<[F64; 2]>
                            name={ "WhitePoint" }
                            lead={Some("色の基準点 (白のCIE座標): 長さ 2")} 
                            input_ref={input_refs[17].clone()}
                            value={ev!(basic_image_info.image_format.white_point, props)}
                            on_func={white_point}
                            caution=true />
                        <Accordion<[F64; 6]>
                            name={ "PrimaryChromaticities" }
                            lead={Some("RGBそれぞれの原色の色度点: 長さ 6")} 
                            input_ref={input_refs[18].clone()}
                            value={ev!(basic_image_info.image_format.primary_chromaticities, props)}
                            on_func={primary_chromaticities}
                            caution=true />
                        <Accordion<[F64; 6]>
                            name={ "ReferenceBlackWhite" }
                            lead={Some("各チャネルの黒・白の基準値: 長さ 6")} 
                            input_ref={input_refs[19].clone()}
                            value={ev!(basic_image_info.image_format.reference_black_white, props)}
                            on_func={reference_black_white}
                            caution=true />
                        <Accordion<Vec<u16>>
                            name={ "ColorMap" }
                            lead={Some("インデックスカラーモード用のカラー定義: 長さ 3×2^{BitsPerSample[0]}")} 
                            input_ref={input_refs[20].clone()}
                            value={ev!(basic_image_info.image_format.color_map, props)}
                            on_func={color_map}
                            caution=true />
                        
                        <AccordionStripOffsets 
                            lead={Some("画像データの格納開始位置: 長さ ceil(ImageHeight / RowsPerStrip)")} 
                            value={ev!(basic_image_info.image_format.strip_offsets, props)} />
                        <Accordion<Vec<u32>>
                            name={ "StripByteCounts" }
                            lead={Some("各ストリップのデータ量: 長さ ceil(ImageHeight / RowsPerStrip)")} 
                            input_ref={input_refs[21].clone()}
                            value={ev!(basic_image_info.image_format.strip_byte_counts, props)}
                            on_func={strip_byte_counts}
                            caution=true />
                        <Accordion<u32>
                            name={ "RowsPerStrip" }
                            lead={Some("ストリップ単位あたりの行数 (画像の分割単位)")} 
                            input_ref={input_refs[22].clone()}
                            value={ev!(basic_image_info.image_format.rows_per_strip, props)}
                            on_func={rows_per_strip}
                            caution=true />

                        <Accordion<Orientation>
                            name={ "Orientation" }
                            lead={Some("回転・反転情報")} 
                            mode={AccordionMode::Dropdown}
                            input_ref={input_refs[23].clone()}
                            value={ev!(basic_image_info.image_format.orientation, props)}
                            on_func={orientation}
                            caution=true />
                        <Accordion<u16>
                            name={ "CellWidth" }
                            lead={Some("古いTIFF形式でのセルの幅 (非推奨)")} 
                            input_ref={input_refs[24].clone()}
                            value={ev!(basic_image_info.image_format.cell_width, props)}
                            on_func={cell_width}
                            caution=true />
                        <Accordion<u16>
                            name={ "CellHeight" }
                            lead={Some("古いTIFF形式でのセルの高さ (非推奨)")} 
                            input_ref={input_refs[25].clone()}
                            value={ev!(basic_image_info.image_format.cell_height, props)}
                            on_func={cell_height}
                            caution=true />
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
pub struct AccordionStripOffsetsProps {
    #[prop_or_default]
    lead: Option<String>,
    value: Option<(Vec<u32>, Vec<Vec<u8>>)>,
}

#[function_component(AccordionStripOffsets)]
pub fn accordion_strip_offsets(props: &AccordionStripOffsetsProps) -> Html {
    let (is_open, value0, value1) = match &props.value {
        Some((v0, v1)) => {
            let value0 = v0.show_value();
            let value1 = v1.iter().map(|vi| format!("{:?}", vi)).collect::<Vec<String>>().join(", ");
            (true, value0, value1)
        }
        None => (false, "".to_string(), "".to_string())
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

    html! {
        <div class="accordion-item">
            <h2 class="accordion-header" id={"heading-StripOffsets"}>
                <button class={btn_classes} type="button"
                    data-bs-toggle="collapse"
                    data-bs-target={"#StripOffsets"}
                    aria-expanded={ is_open.to_string() }
                    aria-controls={"StripOffsets"}>
                    <div class="d-flex flex-column text-start w-100">
                        <span>{ "StripOffsets" }</span>
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
                    <i class="bi bi-slash-circle-fill text-danger ms-2" aria-hidden="true"></i>
                </button>
            </h2>
            <div id={"StripOffsets"}
                class={collapse_classes}
                aria-labelledby={"heading-StripOffsets"}>
                <div class="accordion-body">
                { if is_open { html! {
                    <div class="mb-3">
                        <input type="text" class="form-control" readonly=true value={value0.clone()} />
                        <input type="text" class="form-control" readonly=true value={value1.clone()} />
                    </div>
                } } else {
                    html! {}
                } }
                </div>
            </div>
        </div>
    }
}




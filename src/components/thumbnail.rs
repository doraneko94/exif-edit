use web_sys::HtmlInputElement;
use yew::prelude::*;

use little_exif::exif_tag::ExifTag;

use crate::components::utils::ShowValue;
use crate::{ev, on_int};

use super::accordion::{Accordion, Mode};
use super::utils::InfoProps;

#[function_component(ThumbnailInfo)]
pub fn interop_info(props: &InfoProps) -> Html {
    let input_refs = [use_node_ref()];

    on_int!(u32, thumbnail_length, ThumbnailLength, input_refs[0], props);

    html! {
        <div class="tab-content border border-top-0 p-3">
        <div class="accordion">
            <AccordionThumbnailOffsets
                value={ev!(thumbnail_info.thumbnail_offset, props)} />
            <Accordion<u32>
                name={ "ThumbnailLength" }
                lead={Some("サムネイル画像のサイズ（バイト数）")}
                input_ref={input_refs[0].clone()}
                value={ev!(thumbnail_info.thumbnail_length, props)}
                on_func={thumbnail_length}
                caution=true />
        </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct AccordionThumbnailOffsetsProps {
    value: Option<(Vec<u32>, Vec<u8>)>,
}

#[function_component(AccordionThumbnailOffsets)]
pub fn accordion_thumbnail_offsets(props: &AccordionThumbnailOffsetsProps) -> Html {
    let (is_open, value0, value1) = match &props.value {
        Some((v0, v1)) => {
            let value0 = v0.show_value();
            let value1 = v1.show_value();
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
            <h2 class="accordion-header" id={"heading-ThumbnailOffsets"}>
                <button class={btn_classes} type="button"
                    data-bs-toggle="collapse"
                    data-bs-target={"#ThumbnailOffsets"}
                    aria-expanded={ is_open.to_string() }
                    aria-controls={"ThumbnailOffsets"}>
                    <div class="d-flex flex-column text-start w-100">
                        <span>{ "ThumbnailOffsets" }</span>
                        <small class="text-muted">{ "サムネイル画像の先頭位置（バイトオフセット）" }</small>
                    </div>
                    <i class="bi bi-slash-circle-fill text-danger ms-2" aria-hidden="true"></i>
                </button>
            </h2>
            <div id={"ThumbnailOffsets"}
                class={collapse_classes}
                aria-labelledby={"heading-ThumbnailOffsets"}>
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
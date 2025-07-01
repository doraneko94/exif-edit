use web_sys::HtmlInputElement;
use yew::prelude::*;

use little_exif::exif_tag::ExifTag;

use crate::{ev, on_string, on_int, on_vec};

use super::accordion::{Accordion, AccordionMode, Mode};
use super::utils::InfoProps;

use crate::exif::interop::InteroperabilityIndex;

#[function_component(InteropInfo)]
pub fn interop_info(props: &InfoProps) -> Html {
    let input_refs = [
        use_node_ref(), use_node_ref(), use_node_ref()
    ];

    on_int!(u32, interop_offset, InteropOffset, input_refs[0], props);
    on_string!(interoperability_index, InteroperabilityIndex, input_refs[1], props);
    on_vec!(u8, interoperability_version, InteroperabilityVersion, input_refs[2], props);

    html! {
        <div class="tab-content border border-top-0 p-3">
        <div class="accordion">
            <Accordion<u32>
                name={ "InteropOffset" }
                lead={Some("InteropIFDへのポインタ")}
                input_ref={input_refs[0].clone()}
                value={ev!(interop_info.interop_offset, props)}
                on_func={interop_offset}
                caution=true />
            <Accordion<InteroperabilityIndex>
                name={ "InteroperabilityIndex" }
                lead={Some("InteropIFDの識別子")}
                mode={AccordionMode::Dropdown}
                input_ref={input_refs[1].clone()}
                value={ev!(interop_info.interoperability_index, props)}
                on_func={interoperability_index} />
            <Accordion<[u8; 4]>
                name={ "InteroperabilityVersion" }
                lead={Some("InteropIFDのバージョン番号")}
                input_ref={input_refs[2].clone()}
                value={ev!(interop_info.interoperability_version, props)}
                on_func={interoperability_version} />
        </div>
        </div>
    }
}
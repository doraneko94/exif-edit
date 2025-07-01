use yew::prelude::*;

use crate::ev;

use super::utils::InfoProps;

#[function_component(Uneditable)]
pub fn uneditable(props: &InfoProps) -> Html {
    html! {
        <div class="tab-content border border-top-0 p-3">
        <div class="accordion">
            {
                if let Some(value) = ev!(uneditable.maker_note, props) {
                    html! {
                        <AccordionUneditable 
                            name="MakerNote" 
                            lead={Some("カメラメーカーが独自に記述した内容")}
                            value={value.clone()} 
                            is_open=true /> }
                } else {
                    html! {
                        <AccordionUneditable 
                            name="MakerNote" 
                            lead={Some("カメラメーカーが独自に記述した内容")}
                            value={"".to_string()} 
                            is_open=false /> }
                }
            }
            {
                for props.exif.as_ref().unwrap().uneditable.unknown_all().iter().map(|(name, value)| html! {
                    <AccordionUneditable name={name.clone()} value={value.clone()} is_open=true />
                })
            }
        </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct  AccordionUneditableProps {
    name: String,
    #[prop_or_default]
    lead: Option<&'static str>,
    value: String,
    pub is_open: bool,
}

#[function_component(AccordionUneditable)]
pub fn accordion_uneditable(props: &AccordionUneditableProps) -> Html {
    let is_open = props.is_open;
    let id_safe = props.name.replace(" ", "-").replace("/", "-").replace("(", "").replace(")", "");
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
            <h2 class="accordion-header" id={format!("heading-{}", id_safe.clone())}>
                <button class={btn_classes} type="button"
                    data-bs-toggle="collapse"
                    data-bs-target={format!("#{}", id_safe.clone())}
                    aria-expanded={ is_open.to_string() }
                    aria-controls={id_safe.clone()}>
                    <div class="d-flex flex-column text-start w-100">
                        <span>{ props.name.clone() }</span>
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
            <div id={id_safe.clone()}
                class={collapse_classes}
                aria-labelledby={format!("heading-{}", id_safe.clone())}>
                <div class="accordion-body">
                { if is_open { html! {
                    <div class="mb-3">
                        <textarea
                            id="multiline"
                            class="form-control"
                            rows=3
                            value={props.value.clone()}
                            readonly=true />
                    </div>
                } } else {
                    html! {
                        <p>{ format!("{}は存在しません。", props.name.clone()) }</p>
                    }
                } }
                </div>
            </div>
        </div>
    }
}
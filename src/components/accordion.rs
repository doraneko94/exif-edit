use yew::prelude::*;

use super::utils::ShowValue;
use crate::exif::utils::AllList;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Update,
    Delete,
    Create,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AccordionMode {
    Input,
    Dropdown,
    Time,
    OffsetTime,
}

impl Default for AccordionMode {
    fn default() -> Self {
        Self::Input
    }
}

#[derive(Properties, PartialEq)]
pub struct AccordionProps<T: ShowValue + AllList> {
    pub name: &'static str,
    #[prop_or_default]
    pub mode: AccordionMode,
    pub value: Option<T>,
    #[prop_or_default]
    pub lead: Option<&'static str>,
    pub input_ref: NodeRef,
    pub on_func: Callback<Mode, Callback<MouseEvent>>,
    #[prop_or_default]
    pub caution: bool,
}

#[function_component(Accordion)]
pub fn accordion<T: ShowValue + AllList + 'static>(props: &AccordionProps<T>) -> Html {
    let id_safe = props.name.replace(" ", "-").replace("/", "-");
    let (is_open, value) = match &props.value {
        Some(value) => (true, value.show_value()),
        None => (false, "".to_string())
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
                    data-bs-target={format!("#{}", id_safe.clone())}
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
                    { if props.caution { html! {
                        <i
                            class="bi bi-exclamation-triangle-fill text-warning ms-2"
                            aria-hidden="true">
                        </i>
                    } } else { html! {} } }
                </button>
            </h2>
            <div id={id_safe.clone()}
                class={collapse_classes}
                aria-labelledby={format!("heading-{}", id_safe.clone())}>
                <div class="accordion-body">
                { if is_open {
                    
                    match props.mode {
                        AccordionMode::Input => html! {
                            <Input
                                value={value.clone()}
                                input_ref={props.input_ref.clone()}
                                {on_update} {on_delete}
                            />
                        },
                        AccordionMode::Dropdown => {
                            let value = props.value.clone().unwrap();
                            html! {
                            <Dropdown<T>
                                value={value}
                                input_ref={props.input_ref.clone()}
                                {on_update} {on_delete}
                            />
                        }}
                        AccordionMode::Time => html! {
                            <Time
                                {value}
                                input_ref={props.input_ref.clone()}
                                {on_update} {on_delete} />
                        },
                        AccordionMode::OffsetTime => html! {
                            <OffsetTime 
                                {value}
                                input_ref={props.input_ref.clone()}
                                {on_update} {on_delete} />
                        }
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
pub struct InputProps {
    value: String,
    input_ref: NodeRef,
    on_update: Callback<MouseEvent>,
    on_delete: Callback<MouseEvent>
}

#[function_component(Input)]
pub fn input(props: &InputProps) -> Html {
    html! {
        <>
        <div class="mb-3">
            <input 
                type="text" 
                ref={props.input_ref.clone()}
                class="form-control" 
                value={props.value.clone()} />
        </div>
        <div class="d-flex justify-content-end gap-2 mb-3">
            <button type="button" class="btn btn-primary" onclick={props.on_update.clone()}>{ "更新" }</button>
            <button type="button" class="btn btn-danger" onclick={props.on_delete.clone()}>{ "削除" }</button>
        </div>
        </>
    }
}

#[derive(Properties, PartialEq)]
pub struct DropdownProps<T: AllList + ShowValue> {
    value: T,
    input_ref: NodeRef,
    on_update: Callback<MouseEvent>,
    on_delete: Callback<MouseEvent>
}

#[function_component(Dropdown)]
pub fn dropdown<T: AllList + ShowValue>(props: &DropdownProps<T>) -> Html {
    html! {
        <>
        <div class="mb-3">
        { format!("現在：{}", props.value.show_value()) }
        </div>
        <div class="mb-3">
        { "選択：" }
        <select ref={props.input_ref.clone()}>
        {
            for props.value.all().iter().map(|(i, en)| {
                if en == &props.value {
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
        <div class="d-flex justify-content-end gap-2 mb-3">
            <button type="button" class="btn btn-primary" onclick={props.on_update.clone()}>{ "更新" }</button>
            <button type="button" class="btn btn-danger" onclick={props.on_delete.clone()}>{ "削除" }</button>
        </div>
        </>
    }
}

#[function_component(Time)]
pub fn time(props: &InputProps) -> Html {
    html! {
        <>
        <div class="mb-3">
            <input 
                type="datetime-local" 
                step="0.001"
                ref={props.input_ref.clone()}
                value={props.value.clone()} />
        </div>
        <div class="d-flex justify-content-end gap-2 mb-3">
            <button type="button" class="btn btn-primary" onclick={props.on_update.clone()}>{ "更新" }</button>
            <button type="button" class="btn btn-danger" onclick={props.on_delete.clone()}>{ "削除" }</button>
        </div>
        </>
    }
}

#[function_component(OffsetTime)]
pub fn offset_time(props: &InputProps) -> Html {
    let options = (-12..=14)
        .flat_map(|h: i32| {
            [0, 30]
                .into_iter()
                .filter(move |&m| !(h == 14 && m > 0))
                .map(move |m| {
                    let sign = if h < 0 { "-" } else { "+" };
                    let hour = h.abs();
                    format!("{sign}{:02}:{:02}", hour, m)
                })
        })
        .collect::<Vec<_>>();

    html! {
        <>
        <div class="mb-3">
        { format!("現在：{}", props.value.show_value()) }
        </div>
        <div class="mb-3">
        { "選択：" }
        <select ref={props.input_ref.clone()}>
            { for options.iter().map(|tz| html! {
                <option value={tz.clone()} selected={*tz == props.value}>{ tz }</option>
            }) }
        </select>
        </div>
        <div class="d-flex justify-content-end gap-2 mb-3">
            <button type="button" class="btn btn-primary" onclick={props.on_update.clone()}>{ "更新" }</button>
            <button type="button" class="btn btn-danger" onclick={props.on_delete.clone()}>{ "削除" }</button>
        </div>
        </>
    }
}
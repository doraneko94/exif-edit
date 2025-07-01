use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

use little_exif::exif_tag::ExifTag;

use crate::{ev, on_string};

use super::accordion::{Accordion, Mode};
use super::utils::InfoProps;

use crate::exif::user::{UserComment, UserCommentCode};

#[function_component(UserInfo)]
pub fn user_info(props: &InfoProps) -> Html {
    let input_refs = [
        use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref(), use_node_ref()
    ];

    on_string!(image_description, ImageDescription, input_refs[0], props);
    on_string!(artist, Artist, input_refs[1], props);
    on_string!(copyright, Copyright, input_refs[2], props);
    let user_comment = {
        let input_ref_select = input_refs[3].clone();
        let input_ref_input = input_refs[4].clone();
        let exif = props.exif.clone();
        Callback::from(move |mode: Mode| {
            let input_ref_select = input_ref_select.clone();
            let input_ref_input = input_ref_input.clone();
            let exif = exif.clone();
            Callback::from(move |_: MouseEvent| {
                match mode {
                    Mode::Update => {
                        if let (
                            Some(input_select), 
                            Some(input_input),
                            Some(eed),
                        ) = (
                            input_ref_select.cast::<HtmlSelectElement>(),
                            input_ref_input.cast::<HtmlInputElement>(),
                            exif.as_ref(),
                        ) {
                            let mut eed = eed.clone();
                            let value_input = input_input.value();
                            if let Ok(value_select) = input_select.value().parse::<u64>() {
                                let ucc = UserCommentCode::from_u64(value_select);
                                let uc = UserComment::from_str(&value_input, &ucc);
                                eed.update_tag(ExifTag::UserComment(uc.data.clone()));
                                exif.set(Some(eed));
                            }
                        }
                    }
                    Mode::Delete => {
                        if let Some(eed) = exif.as_ref() {
                            let mut eed = eed.clone();
                            eed.delete_tag(ExifTag::UserComment(Vec::new()));
                            exif.set(Some(eed));
                        }
                    }
                    Mode::Create => {
                        if let Some(eed) = exif.as_ref() {
                            let mut eed = eed.clone();
                            let uc = UserComment::from_str("", &UserCommentCode::ASCII);
                            eed.update_tag(ExifTag::UserComment(uc.data.clone()));
                            exif.set(Some(eed));
                        }
                    }
                }
            })
        })
    };

    html! {
        <div class="tab-content border border-top-0 p-3">
        <div class="accordion">
            <Accordion<String>
                name={ "ImageDescription" }
                lead={Some("画像の簡潔な説明")}
                input_ref={input_refs[0].clone()}
                value={ev!(user_info.image_description, props)}
                on_func={image_description} />
            <Accordion<String>
                name={ "Artist" }
                lead={Some("撮影者や著作権者の名前")}
                input_ref={input_refs[1].clone()}
                value={ev!(user_info.artist, props)}
                on_func={artist} />
            <Accordion<String>
                name={ "Copyright" }
                lead={Some("著作権表記")}
                input_ref={input_refs[2].clone()}
                value={ev!(user_info.copyright, props)}
                on_func={copyright} />
            <AccordionUserComment
                input_refs={[input_refs[3].clone(), input_refs[4].clone()]}
                value={ev!(user_info.user_comment, props)}
                on_func={user_comment} />
        </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct AccordionUserCommentProps {
    pub input_refs: [NodeRef; 2],
    pub value: Option<UserComment>,
    pub on_func: Callback<Mode, Callback<MouseEvent>>
}

#[function_component(AccordionUserComment)]
pub fn accordion_user_comment(props: &AccordionUserCommentProps) -> Html {
    let (is_open, code, decoded) = match &props.value {
        Some(uc) => (true, uc.code.clone(), uc.decoded.clone()),
        _ => (false, UserCommentCode::ASCII, "".to_string())
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
            <h2 class="accordion-header" id={"heading-UserComment"}>
                <button class={btn_classes} type="button"
                    data-bs-toggle="collapse"
                    data-bs-target={"#UserComment"}
                    aria-expanded={ is_open.to_string() }
                    aria-controls={"UserComment"}>
                    <div class="d-flex flex-column text-start w-100">
                        <span>{ "UserComment" }</span>
                        <small class="text-muted">{ "ユーザ自由記述欄" }</small>
                    </div>
                    { "UserComment" }
                </button>
            </h2>
            <div id={"UserComment"}
                class={collapse_classes}
                aria-labelledby={"heading-UserComment"}>
                <div class="accordion-body">
                { if is_open {
                    html! {
                        <>
                        <div class="mb-3">
                        <select ref={props.input_refs[0].clone()}>
                        {
                            for code.all().iter().map(|(i, en)| {
                                if en == &code {
                                    html! {
                                        <option value={i.to_string()} selected=true>
                                            { en.to_string() }
                                        </option>
                                    }
                                } else {
                                    html! {
                                        <option value={i.to_string()}>
                                            { en.to_string() }
                                        </option>
                                    }
                                }
                            })
                        }
                        </select>
                        </div>
                        <div class="mb-3">
                        <textarea
                            id="multiline"
                            class="form-control"
                            rows=5
                            value={decoded}
                            ref={props.input_refs[1].clone()}
                        />
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
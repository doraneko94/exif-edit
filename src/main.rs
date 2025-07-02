use base64::Engine;
use chrono::Local;
use little_exif::filetype::FileExtension;
use little_exif::metadata::Metadata;

use serde_wasm_bindgen::from_value;
use wasm_bindgen::prelude::*;
use web_sys::js_sys::Uint8Array;
use web_sys::{js_sys, HtmlInputElement, Url};
use yew::prelude::*;

use exif_edit::components::basic_image::BasicImageInfo;
use exif_edit::components::exif_capture::ExifCaptureInfo;
use exif_edit::components::gps::GPSInfo;
use exif_edit::components::interop::InteropInfo;
use exif_edit::components::thumbnail::ThumbnailInfo;
use exif_edit::components::user::UserInfo;
use exif_edit::components::uneditable::Uneditable;
use exif_edit::components::tabs::TabItem;
use exif_edit::exif::ExifEditData;
use exif_edit::exif_heic::metadata_heic;

const MAX_FILE_SIZE: usize = 1_073_741_824;

#[wasm_bindgen(module = "/js/heic_handler.js")]
extern "C" {
    #[wasm_bindgen(js_name = ensureJpegBytes)]
    fn ensure_jpeg_bytes(file: web_sys::File) -> js_sys::Promise;
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FileType {
    JPEG,
    HEIC,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tabs {
    BasicImageInfo,
    ExifCaptureInfo,
    GPSInfo,
    InteropInfo,
    ThumbnailInfo,
    UserInfo,
    Uneditable,
}

#[function_component(App)]
fn app() -> Html {
    let file_input = use_node_ref();
    let file_name = use_state(|| None);
    let file_size = use_state(|| None);
    let file_bytes = use_state(|| None);
    let img_data_url = use_state(|| None);
    let exif = use_state(|| None);

    let is_converting = use_state(|| false);
    let selected_tab = use_state(|| Tabs::BasicImageInfo);

    let show_error = use_state(|| None);
    let show_toast = use_state(|| None);

    let final_img_url = use_state(|| None);
    let final_img_ndt = use_state(|| None);

    let process_file = {
        let file_name = file_name.clone();
        let file_size = file_size.clone();
        let file_bytes = file_bytes.clone();
        let img_data_url = img_data_url.clone();
        let exif = exif.clone();
        let is_converting = is_converting.clone();
        let show_error = show_error.clone();
        let show_toast = show_toast.clone();
        let final_img_url = final_img_url.clone();
        let final_img_ndt = final_img_ndt.clone();

        Callback::from(move |file: web_sys::File| {
            let file_type = if file.name().to_lowercase().ends_with(".heic") {
                FileType::HEIC
            } else {
                FileType::JPEG
            };

            let file_name_value = file.name();
            let promise = ensure_jpeg_bytes(file);

            file_name.set(None);
            file_size.set(None);
            file_bytes.set(None);
            img_data_url.set(None);
            exif.set(None);
            final_img_url.set(None);
            final_img_ndt.set(None);

            let file_name = file_name.clone();
            let file_size = file_size.clone();
            let file_bytes = file_bytes.clone();
            let img_data_url = img_data_url.clone();
            let exif = exif.clone();
            let is_converting = is_converting.clone();
            let show_error = show_error.clone();
            let show_toast = show_toast.clone();

            wasm_bindgen_futures::spawn_local(async move {
                is_converting.set(true);
                match wasm_bindgen_futures::JsFuture::from(promise).await {
                    Ok(js_val) => {
                        match (
                            js_sys::Reflect::get(&js_val, &"jpeg".into()),
                            js_sys::Reflect::get(&js_val, &"exif".into()),
                        ) {
                            (Ok(v), Ok(ex)) => {
                                let jpeg_u8 = js_sys::Uint8Array::new(&v).to_vec();

                                if jpeg_u8.len() > MAX_FILE_SIZE {
                                    show_error.set(Some("ファイルサイズが大きすぎます (最大1GBまで対応) 。".to_string()));
                                    is_converting.set(false);
                                    return;
                                }
                                if let Ok(metadata) = match file_type {
                                    FileType::JPEG => Metadata::new_from_vec(&jpeg_u8, FileExtension::JPEG),
                                    FileType::HEIC => {
                                        match from_value::<serde_json::Value>(ex) {
                                            Ok(exif_dict) => Ok(metadata_heic(&exif_dict)),
                                            Err(_) => Ok(Metadata::new()),
                                        }
                                    }
                                } {
                                    file_name.set(Some(file_name_value));
                                    file_size.set(Some(jpeg_u8.len()));
                                    exif.set(Some(ExifEditData::new(&metadata)));
                                } else {
                                    file_name.set(Some(file_name_value));
                                    file_size.set(Some(jpeg_u8.len()));
                                    exif.set(Some(ExifEditData::new(&Metadata::new())));
                                }

                                let jpeg_base64 = base64::engine::general_purpose::STANDARD.encode(&jpeg_u8);
                                let data_url = format!("data:image/jpeg;base64,{}", jpeg_base64);
                                img_data_url.set(Some(data_url));
                                file_bytes.set(Some(jpeg_u8));
                                show_toast.set(Some("画像の読み込みと変換に成功しました。".to_string()));
                            }
                            _ => {
                                file_name.set(None);
                                file_size.set(None);
                                exif.set(None);
                            }
                        }
                    }
                    Err(_) => {
                        file_name.set(None);
                        file_size.set(None);
                        exif.set(None);
                        show_error.set(Some("ファイルの読み込みまたは変換に失敗しました。対応形式かどうかをご確認ください。".to_string()));
                    }
                }
                is_converting.set(false);
            });
        })
    };

    let on_file_change = {
        let file_input = file_input.clone();
        let process_file = process_file.clone();
        let show_error = show_error.clone();
        Callback::from(move |_| {
            if let Some(input) = file_input.cast::<HtmlInputElement>() {
                if let Some(files) = input.files() {
                    if let Some(file) = files.get(0) {
                        if file.size() > MAX_FILE_SIZE as f64 {
                            show_error.set(Some("ファイルサイズが大きすぎます (最大1GBまで対応) 。".to_string()));
                            return;
                        }
                        process_file.emit(file);
                    }
                }
            }
        })
    };

    let on_drop = {
        let process_file = process_file.clone();
        Callback::from(move |event: DragEvent| {
            event.prevent_default();
            if let Some(files) = event.data_transfer().and_then(|dt| dt.files()) {
                if let Some(file) = files.get(0) {
                    process_file.emit(file);
                }
            }
        })
    };

    let on_drag_over = Callback::from(|event: DragEvent| {
        event.prevent_default();
    });

    let on_download = {
        let file_name = file_name.clone();
        let file_bytes = file_bytes.clone();
        let exif = exif.clone();
        Callback::from(move |_: MouseEvent| {
            if let (
                Some(name), 
                Some(bytes), 
                Some(eed)
            ) = (file_name.as_ref(), file_bytes.as_ref(), exif.as_ref()) {
                let mut bytes = bytes.clone();
                if let Ok(()) = eed.metadata.write_to_vec(&mut bytes, FileExtension::JPEG) {
                    let u8_array = Uint8Array::new_with_length(bytes.len() as u32);
                    u8_array.copy_from(&bytes);
                    let array = js_sys::Array::new();
                    array.push(&u8_array.buffer());

                    if let Ok(blob) = web_sys::Blob::new_with_u8_array_sequence(&array) {
                        if let Some(window) = web_sys::window() {
                            if let Some(document) = window.document() {
                                if let (Ok(anchor), Ok(url)) = (
                                    document.create_element("a"), 
                                    Url::create_object_url_with_blob(&blob)
                                ) {
                                    let savename = format!("{}_{}.jpg",
                                        name.split(".").collect::<Vec<&str>>()[0],
                                        Local::now().naive_local().format("%Y_%m_%dT%H_%M_%S").to_string()
                                    );
                                    if let (Some(body), Ok(()), Ok(())) = (
                                        document.body(),
                                        anchor.set_attribute("href", &url),
                                        anchor.set_attribute("download", &savename)
                                    ) {
                                        if let (Ok(_), Some(dr)) = (
                                            body.append_child(&anchor),
                                            anchor.dyn_ref::<web_sys::HtmlElement>()
                                        ) {
                                            dr.click();
                                            match Url::revoke_object_url(&url) { Ok(_) => {} Err(_) => {}}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        })
    };

    let on_show_final = {
        let file_bytes = file_bytes.clone();
        let exif = exif.clone();
        let final_img_url = final_img_url.clone();
        let final_img_ndt = final_img_ndt.clone();
        Callback::from(move |_: MouseEvent| {
            if let (Some(bytes), Some(eed)) = (file_bytes.as_ref(), exif.as_ref()) {
                let mut bytes = bytes.clone();
                if let Ok(()) = eed.metadata.write_to_vec(&mut bytes, FileExtension::JPEG) {
                    final_img_url.set(Some(format!("data:image/jpeg;base64,{}", base64::engine::general_purpose::STANDARD.encode(&bytes))));
                    final_img_ndt.set(Some(Local::now().naive_local()));
                }
            }
        })
    };

    let on_delete_all = {
        let exif = exif.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(eed) = exif.as_ref() {
                let mut eed = eed.clone();
                let metadata = eed.metadata.clone();
                for ifd in metadata.get_ifds() {
                    for tag in ifd.get_tags() {
                        eed.delete_tag(tag.clone());
                    }
                }
                exif.set(Some(eed));
            }
        })
    };

    {
        let show_toast = show_toast.clone();
        use_effect_with(
            show_toast.clone(),
            move |msg: &UseStateHandle<Option<String>>| {
                if msg.is_some() {
                    let show_toast = show_toast.clone();
                    let handle = gloo::timers::callback::Timeout::new(3000, move || {
                        show_toast.set(None);
                    });
                    handle.forget();
                }
                || ()
            }
        );
    }

    html! {
        <>
        {
            if *is_converting {
                html! {
                    <div class="overlay-loading">
                        <div class="text-center">
                            <div class="spinner-border text-primary" role="status" style="width: 3rem; height: 3rem;">
                                <span class="visually-hidden">{ "読み込み中..." }</span>
                            </div>
                            <p class="mt-3 fw-bold text-primary">{ "HEICをJPEGに変換中..." }</p>
                        </div>
                    </div>
                }
            } else { html! {} }
        }
        <div class="container py-4">
        <div class="row justify-content-center">
        <div class="col-12 col-md-10 col-lg-8 mx-auto">

        <div class="card shadow-sm mb-4">
        <div class="card-body text-center">
            <h1 class="h4 mb-2">{"JPEG/HEIC Exif情報編集ツール"}</h1>
            <p class="text-muted mb-0">{ "JPEG/HEIC画像をアップロードして、Exifデータを編集・JPEGで保存できます。" }</p>
        </div>
        </div>

        <div
            class="card shadow-sm mb-4"
            ondragover={on_drag_over}
            ondrop={on_drop}
        >
            <div class="card-body text-center">
            <h2 class="h5 mb-3">{ "ステップ1: 画像をアップロード" }</h2>
            <p class="mb-3">{ "HEIC または JPEG 形式の画像をドラッグ＆ドロップするか、ボタンで選択してください。" }</p>
            <div class="mx-auto" style="max-width: 360px;">
                <input 
                    id="fileUpload"
                    type="file" 
                    accept="image/jpeg,image/heic" 
                    ref={file_input} 
                    class="form-control mb-3"
                    onchange={on_file_change}
                    aria-describedby="fileHelp" />
                <div id="fileHelp" class="form-text">
                    { "対応形式: JPEG, HEIC" }
                </div>
            </div>
            {
                if let Some(file_size) = *file_size {
                    html! { <p class="text-muted small">{ format!("ファイルサイズ: {:.2} KB", file_size as f64 / 1024.0) }</p> }
                } else {
                    html! {}
                }
            }
            {
                /*if *is_converting {
                    html! {
                        <div class="mt-3">
                            <div class="spinner-border text-primary" role="status">
                                <span class="visually-hidden">{ "変換中..." }</span>
                            </div>
                            <div class="mt-2 text-muted small">{ "HEICをJPEGに変換中です..." }</div>
                        </div>
                    }
                } else */
                if let Some(url) = (*img_data_url).clone() {
                    html! {
                        <>
                        <div class="mb-3">
                            <img
                                src={url}
                                alt="アップロードされた画像のプレビュー"
                                class="img-fluid rounded shadow-sm d-block mx-auto"
                                style="max-height: 50vh; width: auto;"
                            />
                        </div>
                        <div class="mb-3 d-flex flex-column gap-3">
                            <button type="button" class="btn btn-primary w-100" onclick={on_download.clone()}>{ "編集後のファイルをダウンロード (JPEG)" }</button>
                            <p class="my-0">{"↓スマートフォンの場合は、こちらで表示した画像を長押ししてダウンロードしてください。"}</p>
                            <button type="button" class="btn btn-info w-100" onclick={on_show_final.clone()}>
                            { if (*final_img_url).is_some() { "編集後の画像を再表示する" } else { "編集後の画像を表示する" } }
                            </button>
                        </div>
                        {
                            if let Some(url) = (*final_img_url).clone() {
                                html! {
                                    <div class="card shadow-sm mb-3">
                                        <div class="card-body text-center">
                                            <h5 class="card-title">{ "編集後の画像" }</h5>
                                            { if let Some(ndt) = (*final_img_ndt).clone() {
                                                html! { <p>{format!("最終更新: {}", ndt.format("%Y年%m月%d日 %H時%M分%S秒").to_string())}</p> }
                                            } else {
                                                html! {}
                                            } }
                                            
                                            <p>{"※Exif情報を更新した場合は、再表示してください。"}</p>
                                            <img
                                                src={url}
                                                alt="編集後の画像のプレビュー"
                                                class="img-fluid rounded shadow-sm d-block mx-auto"
                                                style="max-height: 50vh; width: auto;"
                                            />
                                        </div>
                                    </div>
                                }
                            } else {
                                html! {}
                            }
                        }
                        </>
                    }
                } else {
                    html! {}
                }
            }
        </div>
        </div>
        { if let Some(_) = exif.as_ref() {
            html! {
                <div class="card shadow-sm mb-4">
                <div class="card-body">
                <h2 class="h5 mb-3 text-center">{ "ステップ2: Exif情報の確認・編集" }</h2>
                <p class="text-muted small text-center mb-4">
                    { "必要に応じてExif情報を削除または編集できます。編集タブを切り替えて内容を確認してください。" }
                </p>

                <div class="mb-3">
                    <button type="button" class="btn btn-danger w-100" onclick={on_delete_all.clone()}>{ "すべてのExif情報を削除" }</button>
                </div>

                <ul class="nav nav-tabs flex-nowrap mb-3">
                <TabItem<Tabs> tab={Tabs::BasicImageInfo} selected_tab={selected_tab.clone()} message={"基本情報"} icon={"image"} />
                <TabItem<Tabs> tab={Tabs::ExifCaptureInfo} selected_tab={selected_tab.clone()} message={"詳細情報"} icon={"database"} />
                <TabItem<Tabs> tab={Tabs::GPSInfo} selected_tab={selected_tab.clone()} message={"位置情報"} icon={"geo-alt"} />
                <TabItem<Tabs> tab={Tabs::InteropInfo} selected_tab={selected_tab.clone()} message={"相互運用性"} icon={"arrow-left-right"} />
                <TabItem<Tabs> tab={Tabs::ThumbnailInfo} selected_tab={selected_tab.clone()} message={"サムネイル情報"} icon={"search"} />
                <TabItem<Tabs> tab={Tabs::UserInfo} selected_tab={selected_tab.clone()} message={"ユーザ情報"} icon={"person-circle"} />
                <TabItem<Tabs> tab={Tabs::Uneditable} selected_tab={selected_tab.clone()} message={"変更不可"} icon={"slash-circle"} />
                </ul>

                <div class="mb-3">
                    <p>
                        <i class="bi bi-exclamation-triangle-fill text-warning ms-2" aria-hidden="true"></i>
                        { ": 編集によりファイルが破損しうる" }
                    </p>
                    <p>
                        <i class="bi bi-slash-circle-fill text-danger ms-2" aria-hidden="true"></i>
                        { ": 編集不能" }
                    </p>
                </div>

                {
                    match *selected_tab {
                        Tabs::BasicImageInfo =>  html! { <BasicImageInfo exif={exif.clone()} /> },
                        Tabs::ExifCaptureInfo =>  html! { <ExifCaptureInfo exif={exif.clone()} /> },
                        Tabs::GPSInfo => html! { <GPSInfo exif={exif.clone()} /> },
                        Tabs::InteropInfo => html! { <InteropInfo exif={exif.clone()} /> },
                        Tabs::ThumbnailInfo => html! { <ThumbnailInfo exif={exif.clone()} /> },
                        Tabs::UserInfo => html! { <UserInfo exif={exif.clone()} /> },
                        Tabs::Uneditable => html! { <Uneditable exif={exif.clone()} /> }
                    }
                }
                </div>
                </div>
            }
        } else { html! {} } }

        <footer class="text-center text-muted mt-5 mb-2 small">
            <p class="mb-1">{ "© 2025 "}<a href="https://j-impact.jp/">{"J-IMPACT"}</a>{". All rights reserved." }</p>
            <p class="mb-0">{ "本アプリは個人利用向けです。我々は、利用によるいかなる損害も責任を負いません。" }</p>
            <a
                href="https://github.com/doraneko94/exif-edit"
                target="_blank"
                rel="noopener noreferrer"
                class="text-muted text-decoration-none"
            >
            <i class="bi bi-github me-1"></i>{"GitHubでソースコードを見る"}
            </a>
        </footer>

        <div class="accordion mt-4" id="footerAccordion">
            <div class="accordion-item">
                <h2 class="accordion-header" id="headingInfo">
                    <button class="accordion-button collapsed" type="button"
                        data-bs-toggle="collapse"
                        data-bs-target="#collapseInfo"
                        aria-expanded="false"
                        aria-controls="collapseInfo">
                        { "このアプリについて" }
                    </button>
                </h2>
                <div id="collapseInfo" class="accordion-collapse collapse" aria-labelledby="headingInfo" data-bs-parent="#footerAccordion">
                    <div class="accordion-body small text-muted">
                        <ul class="mb-0">
                            <li>{ "対応形式: JPEG (.jpg), HEIC (.heic)" }</li>
                            <li>{ "最大ファイルサイズ: 1GBまで" }</li>
                            <li>{ "Exifデータを読み取り、編集、削除してJPEG形式で保存できます" }</li>
                            <li>{ "変換後の画像はローカルで処理され、外部に送信されません" }</li>
                            <li>{ "本アプリはオープンソースとして提供されている試験的なツールです" }</li>
                        </ul>
                    </div>
                </div>
            </div>
        </div>

        // エラーメッセージ表示（画面上部）
        {
            if let Some(msg) = (*show_error).clone() {
                html! {
                    <div class="alert alert-danger alert-dismissible fade show mt-3" role="alert">
                        { msg }
                        <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>
                    </div>
                }
            } else { html! {} }
        }

        // トースト表示（画面右下）
        {
            if let Some(msg) = (*show_toast).clone() {
                html! {
                    <div class="toast-container position-fixed bottom-0 end-0 p-3" style="z-index: 1055;">
                        <div class="toast align-items-center text-bg-success border-0 show" role="alert">
                            <div class="d-flex">
                                <div class="toast-body">
                                    { msg }
                                </div>
                                <button type="button" class="btn-close btn-close-white me-2 m-auto" data-bs-dismiss="toast" aria-label="Close"></button>
                            </div>
                        </div>
                    </div>
                }
            } else { html! {} }
        }
        </div>
        </div>
        </div>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
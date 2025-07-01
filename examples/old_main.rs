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

    let process_file = {
        let file_name = file_name.clone();
        let file_size = file_size.clone();
        let file_bytes = file_bytes.clone();
        let img_data_url = img_data_url.clone();
        let exif = exif.clone();
        let is_converting = is_converting.clone();

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

            let file_name = file_name.clone();
            let file_size = file_size.clone();
            let file_bytes = file_bytes.clone();
            let img_data_url = img_data_url.clone();
            let exif = exif.clone();
            let is_converting = is_converting.clone();

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
                    }
                }
                is_converting.set(false);
            });
        })
    };

    let on_file_change = {
        let file_input = file_input.clone();
        let process_file = process_file.clone();
        Callback::from(move |_| {
            if let Some(input) = file_input.cast::<HtmlInputElement>() {
                if let Some(files) = input.files() {
                    if let Some(file) = files.get(0) {
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

    html! {
        // <div class="container-fluid px-2">
        <div class="container py-4">
        // <div class="row">
        <div class="row justify-content-center">
        // <div class="col-12 col-md-10 col-lg-8 mx-auto">

        <div class="card shadow-sm mb-4">
        <div class="card-body text-center">
        <h1 class="h4 mb-3">{"JPEG/HEIC Exif情報編集ツール"}</h1>
        <p class="text-muted">{ "JPEG/HEIC画像をアップロードして、Exifデータを編集・JPEGで保存できます。" }</p>
        <div class="col-12 col-md-10 col-lg-8">
        <div
            class="border border-2 border-secondary rounded p-3 text-center mb-4"
            ondragover={on_drag_over}
            ondrop={on_drop}
            style="overflow-x: auto;"
        >
            <p>{ "画像をここにドラッグ＆ドロップするか、ファイル選択してください" }</p>
            <input 
                type="file" 
                accept="image/jpeg,image/heic" 
                ref={file_input} 
                onchange={on_file_change} />
            {
                if let Some(file_size) = *file_size {
                    html! { <p>{ format!("File size: {:.2} KB", file_size as f64 / 1024.0) }</p> }
                } else {
                    html! {}
                }
            }
            {
                if *is_converting {
                    html! {
                        <div class="text-center mt-3">
                            <div class="spinner-border text-primary" role="status">
                                <span class="visually-hidden">{ "変換中..." }</span>
                            </div>
                            <div class="mt-2">{ "HEICをJPEGに変換中です..." }</div>
                        </div>
                    }
                } else if let Some(url) = (*img_data_url).clone() {
                    html! {
                        <>
                        <div>
                            <img
                                src={url}
                                alt="Uploaded Image"
                                class="img-fluid mx-auto d-block"
                                style="max-height: 50vh;"
                            />
                        </div>
                        <div style="margin-top: 1em;">
                            <button type="button" class="btn btn-primary" style="width: 100%;" onclick={on_download.clone()}>{ "編集後のファイルをダウンロード（JPEGファイル）" }</button>
                        </div>
                        </>
                    }
                } else {
                    html! {}
                }
            }
        </div>
        </div>
        </div>
        { if let Some(_) = exif.as_ref() {
            html! {
                <>
                <div class="mb-3">
                    <button type="button" class="btn btn-danger" style="width: 100%;" onclick={on_delete_all.clone()}>{ "すべてのExif情報を削除" }</button>
                </div>

                <ul class="nav nav-tabs d-flex flex-wrap gap-2 mb-3">
                <TabItem<Tabs> tab={Tabs::BasicImageInfo} selected_tab={selected_tab.clone()} message={"BasicImageInfo"} />
                <TabItem<Tabs> tab={Tabs::ExifCaptureInfo} selected_tab={selected_tab.clone()} message={"ExifCaptureInfo"} />
                <TabItem<Tabs> tab={Tabs::GPSInfo} selected_tab={selected_tab.clone()} message={"GPSInfo"} />
                <TabItem<Tabs> tab={Tabs::InteropInfo} selected_tab={selected_tab.clone()} message={"InteropInfo"} />
                <TabItem<Tabs> tab={Tabs::ThumbnailInfo} selected_tab={selected_tab.clone()} message={"ThumbnailInfo"} />
                <TabItem<Tabs> tab={Tabs::UserInfo} selected_tab={selected_tab.clone()} message={"UserInfo"} />
                <TabItem<Tabs> tab={Tabs::Uneditable} selected_tab={selected_tab.clone()} message={"Uneditable"} />
                </ul>

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
                </>
            }
        } else { html! {} } }
        </div>
        </div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
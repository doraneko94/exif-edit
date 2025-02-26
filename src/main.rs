use gloo::file::callbacks::FileReader;
use gloo::file::File;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement};
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    let file_data = use_state(|| None);
    let file_size = use_state(|| None);
    let reader = use_state(|| None);

    let on_file_change = {
        let file_data = file_data.clone();
        let file_size = file_size.clone();
        let reader = reader.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
            if let Some(file) = input.files().and_then(|files| files.get(0)) {
                let file_name = file.name();
                let file_size_value = file.size();
                let file_type = file.type_();

                // 対応フォーマットチェック
                if file_type == "image/jpeg" || file_type == "image/png" || file_type == "image/tiff" {
                    file_size.set(Some(file_size_value));
                    let file_clone = File::from(file);
                    let file_reader = gloo::file::callbacks::read_as_data_url(&file_clone, {
                        let file_data = file_data.clone();
                        move |result| {
                            if let Ok(data) = result {
                                file_data.set(Some(data));
                            }
                        }
                    });
                    reader.set(Some(file_reader));
                }
            }
        })
    };

    html! {
        <div>
            <input type="file" accept="image/jpeg,image/png,image/tiff" onchange={on_file_change} />
            { if let Some(file_size) = *file_size {
                html! { <p>{ format!("File size: {:.2} KBです", file_size as f64 / 1024.0) }</p> }
            } else {
                html! {}
            }}
            { if let Some(data) = (*file_data).clone() {
                html! { <img src={data} alt="Uploaded Image" style="max-width: 100%; height: auto;" /> }
            } else {
                html! {}
            }}
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}


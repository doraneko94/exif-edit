use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TabItemProps<T: PartialEq + 'static> {
    pub tab: T,
    pub selected_tab: UseStateHandle<T>,
    pub message: &'static str,
    pub icon: &'static str,
}

#[function_component(TabItem)]
pub fn basic_image_info_item<T: Clone + Copy + PartialEq + Eq + 'static>(props: &TabItemProps<T>) -> Html {
    let on_tab_click = {
        let selected_tab = props.selected_tab.clone();
        let tab = props.tab;
        Callback::from(move |_| selected_tab.set(tab))
    };

    html! {
        <li class="nav-item">
            <button class={classes!("nav-link", (*props.selected_tab == props.tab).then_some("active"))}
                onclick={on_tab_click}>
                { props.message }
                <i class={format!("bi bi-{} me-1", props.icon)} aria-hidden="true"></i>
            </button>
        </li>
    }
}


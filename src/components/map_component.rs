use gloo_utils::document;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{Element, HtmlElement, Node};
use yew::prelude::*;

use leaflet::{DragEndEvent, Icon, IconOptions, LatLng, Map, MapOptions, Marker, MarkerOptions, TileLayer};

pub enum Msg {}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub lat: f64,
    pub lng: f64,
    pub on_marker_move: Callback<(f64, f64)>,
}

pub struct MapComponent {
    map: Option<Map>,
    red_marker: Option<Marker>,
    latlng: LatLng,
    container: HtmlElement,
}

impl MapComponent {
    fn render_map(&self) -> Html {
        let node: &Node = &self.container.clone().into();
        Html::VRef(node.clone())
    }

    fn update_view(&mut self, ctx: &Context<MapComponent>) {
        if let (Some(map), Some(red_marker)) = (&self.map, &self.red_marker) {
            let props = ctx.props();
            let marker_pos = LatLng::new(props.lat, props.lng);
            red_marker.set_lat_lng(&marker_pos);
            map.set_view(&marker_pos, 11.0);
        }
    }
}

impl Component for MapComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let container: Element = document().create_element("div").unwrap();
        let container: HtmlElement = container.dyn_into().unwrap();
        container.set_class_name("map");

        Self {
            map: None,
            red_marker: None,
            latlng: LatLng::new(props.lat, props.lng),
            container
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let map = Map::new_with_element(&self.container, &MapOptions::default());
            map.set_view(&self.latlng, 11.0);

            TileLayer::new("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png")
                .add_to(&map);

            let yellow_icon_options = IconOptions::new();
            yellow_icon_options.set_icon_url("https://maps.gstatic.com/mapfiles/ms2/micons/yellow-dot.png".to_string());
            yellow_icon_options.set_icon_size((32.0, 32.0).into());
            yellow_icon_options.set_icon_anchor((16.0, 32.0).into());
            let yellow_icon = Icon::new(&yellow_icon_options);
            let yellow_marker_options = MarkerOptions::new();
            yellow_marker_options.set_icon(yellow_icon);
            Marker::new_with_options(&self.latlng, &yellow_marker_options).add_to(&map);

            let red_icon_options = IconOptions::new();
            red_icon_options.set_icon_url("https://maps.gstatic.com/mapfiles/ms2/micons/red-dot.png".to_string());
            red_icon_options.set_icon_size((32.0, 32.0).into());
            red_icon_options.set_icon_anchor((16.0, 32.0).into());
            let red_icon = Icon::new(&red_icon_options);
            let red_marker_options = MarkerOptions::new();
            red_marker_options.set_draggable(true);
            red_marker_options.set_interactive(true);
            red_marker_options.set_icon(red_icon);

            let red_marker = Marker::new_with_options(&self.latlng, &red_marker_options);

            let on_marker_move = ctx.props().on_marker_move.clone();
            let red_marker_clone = red_marker.clone();

            let dragend_cb = Closure::wrap(Box::new(move |_: DragEndEvent| {
                let new_pos = red_marker_clone.get_lat_lng();
                on_marker_move.emit((new_pos.lat(), new_pos.lng()));
            }) as Box<dyn FnMut(DragEndEvent)>);
            red_marker.on("dragend", dragend_cb.as_ref().unchecked_ref());
            dragend_cb.forget();

            red_marker.add_to(&map);

            self.map = Some(map);
            self.red_marker = Some(red_marker);
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let new_pos = (ctx.props().lat, ctx.props().lng);
        let old_pos = (old_props.lat, old_props.lng);

        if new_pos.0 != old_pos.0 && new_pos.1 != old_pos.1 {
            self.update_view(ctx);
        }
        
        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="map-container component-container">
                { self.render_map() }
            </div>
        }
    }
}
use gloo_utils::document;
use leaflet::{DragEndEvent, LatLng, Map, MapOptions, Marker, MarkerOptions, Point, Polyline, TileLayer};
use web_sys::{js_sys::Array, wasm_bindgen::{prelude::Closure, JsCast}, Element, HtmlElement, Node};
use yew::{html, Callback, Component, Context, Html, Properties};

const ARROW_WING_LENGTH_RATE: f64 = 0.9;
const ARROW_WING_ANGLE: f64 = 0.1;

pub enum Msg {}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub lat: f64,
    pub lng: f64,
    pub azimuth: f64,
    pub on_marker_move: Callback<(f64, f64)>,
}

fn make_arrow(map: &Map, marker_pos: &LatLng, arrow_length: f64, azimuth_rad: f64) -> Array {
    let dx = arrow_length * azimuth_rad.sin();
    let dy = -arrow_length * azimuth_rad.cos();
    let arrow_wing_length = arrow_length * ARROW_WING_LENGTH_RATE;
    let dx_r = arrow_wing_length * (azimuth_rad + ARROW_WING_ANGLE).sin();
    let dy_r = -arrow_wing_length * (azimuth_rad + ARROW_WING_ANGLE).cos();
    let dx_l = arrow_wing_length * (azimuth_rad - ARROW_WING_ANGLE).sin();
    let dy_l = -arrow_wing_length * (azimuth_rad - ARROW_WING_ANGLE).cos();
    
    let marker_container = map.lat_lng_to_container_point(&marker_pos);
    let arrow_tip_container = Point::new(
        marker_container.x() + dx,
        marker_container.y() + dy,
    );
    let arrow_tip_latlng = map.container_point_to_lat_lng(&arrow_tip_container);
    let arrow_coords = Array::new();
    arrow_coords.push(&marker_pos);
    arrow_coords.push(&arrow_tip_latlng);
    arrow_coords.push(
        &map.container_point_to_lat_lng(&Point::new(
            marker_container.x() + dx_r,
            marker_container.y() + dy_r
        ))
    );
    arrow_coords.push(&arrow_tip_latlng);
    arrow_coords.push(
        &map.container_point_to_lat_lng(&Point::new(
            marker_container.x() + dx_l,
            marker_container.y() + dy_l
        ))
    );
    arrow_coords
}

pub struct MapComponent {
    map: Option<Map>,
    marker: Option<Marker>,
    arrow: Option<Polyline>,
    latlng: LatLng,
    container: HtmlElement,
}

impl MapComponent {
    fn render_map(&self) -> Html {
        let node: &Node = &self.container.clone().into();
        Html::VRef(node.clone())
    }

    fn update_view(&mut self, ctx: &Context<MapComponent>) {
        if let (Some(ref mut map), Some(ref mut marker)) = (&mut self.map, &mut self.marker) {
            let props = ctx.props();

            let marker_pos = LatLng::new(props.lat, props.lng);
            marker.set_lat_lng(&marker_pos);
            map.set_view(&marker_pos, 11.0);
            
            let size = map.get_size();
            let arrow_length = size.y() * 0.3;
            let azimuth_deg = props.azimuth;
            let azimuth_rad = azimuth_deg.to_radians();
            
            let arrow_coords = make_arrow(&map, &marker_pos, arrow_length, azimuth_rad);
            if let Some(ref mut arrow) = self.arrow {
                arrow.set_lat_lngs(&arrow_coords);
            } else {
                let new_arrow = Polyline::new(&arrow_coords);
                new_arrow.add_to(&map);
                self.arrow = Some(new_arrow);
            }
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
            marker: None,
            arrow: None,
            latlng: LatLng::new(props.lat, props.lng),
            container
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let map = Map::new_with_element(&self.container, &MapOptions::default());
            map.set_view(&self.latlng,11.0);
            TileLayer::new(
                "https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
            ).add_to(&map);

            let marker_options = MarkerOptions::new();
            marker_options.set_draggable(true);
            marker_options.set_interactive(true);
            let marker = Marker::new_with_options(&self.latlng, &marker_options);
            let on_marker_move = ctx.props().on_marker_move.clone();
            let marker_clone = marker.clone();
            let dragend_cb = Closure::wrap(Box::new(move |_: DragEndEvent| {
                let new_pos = marker_clone.get_lat_lng();
                on_marker_move.emit((new_pos.lat(), new_pos.lng()));
            }) as Box<dyn FnMut(DragEndEvent)>);
            marker.on("dragend", dragend_cb.as_ref().unchecked_ref());
            dragend_cb.forget();
            marker.add_to(&map);

            self.map = Some(map);
            self.marker = Some(marker);
        }
        self.update_view(ctx);
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.update_view(ctx);
        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="map-container component-container">
                {self.render_map()}
            </div>
        }
    }
}
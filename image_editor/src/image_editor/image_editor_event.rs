use cgmath::Point2;

pub enum PointerButton {
    Main,
    Second,
    Third,
    Fourth,
}

pub struct LocationInViewport(pub Point2<u32>);

pub enum ImageEditorEvent {
    PointerClick {
        button: PointerButton,
        location: LocationInViewport,
        pressure: f32,
    },
    PointerMoved {
        new_location: LocationInViewport,
        new_pressure: f32,
    },
}

#[macro_use]
extern crate cfg_if;
use flatgeobuf::*;
use geozero::svg::SvgWriter;
use wasm_bindgen::prelude::*;

cfg_if! {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function to get better error messages if we ever panic.
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        use console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        fn set_panic_hook() {}
    }
}

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

async fn fgb_svg(url: &str, width: u32, height: u32) -> Vec<u8> {
    let mut fgb = HttpFgbReader::open(url).await.unwrap();
    fgb.select_all().await.unwrap();
    let mut svg_data: Vec<u8> = Vec::new();
    let mut svg = SvgWriter::new(&mut svg_data, true);
    if let Some(envelope) = fgb.header().envelope() {
        svg.set_dimensions(
            envelope.get(0),
            envelope.get(1),
            envelope.get(2),
            envelope.get(3),
            width,
            height,
        );
    }
    fgb.process_features(&mut svg).await.unwrap();
    svg_data
}

// Called by our JS entry point to run the example
#[wasm_bindgen]
pub async fn run() -> Result<(), JsValue> {
    // If the `console_error_panic_hook` feature is enabled this will set a panic hook, otherwise
    // it will do nothing.
    set_panic_hook();

    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    // Manufacture the element we're gonna append
    let val = document.create_element("p")?;
    let svg_data = fgb_svg(
        "https://pkg.sourcepole.ch/countries.fgb",
        window.inner_width().unwrap().as_f64().unwrap() as u32,
        window.inner_height().unwrap().as_f64().unwrap() as u32,
    )
    .await;
    let svg_str = std::str::from_utf8(&svg_data).unwrap();
    val.set_inner_html(svg_str);

    body.append_child(&val)?;

    Ok(())
}

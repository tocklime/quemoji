use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use enigo::{Enigo, Keyboard, Settings};
use fltk::{
    app::{self, event_key},
    enums::{CallbackTrigger, Event, Key},
    group::Pack,
    input::Input,
    output::MultilineOutput,
    prelude::*,
    window::Window,
};
use lazy_static::lazy_static;
lazy_static! {
    // all the leaking here is to get &str in the slice, which is what rust_fuzzy_search wants.
    // ok to leak in static initialisation code because the whole point is it only happens once.
    static ref ALL_EMOJI_NAMES: &'static [&'static str] = Box::leak(
        emojis::iter()
            .flat_map(|x| x.shortcodes().map(move |c| {
                let x = Box::leak(format!("{x} {c}").into_boxed_str());
                &*x
            }))
            .collect::<Vec<&'static str>>()
            .into_boxed_slice()
    );
}

fn emoji_search(input: &str) -> Vec<&str> {
    let results = rust_fuzzy_search::fuzzy_search_best_n(input, &ALL_EMOJI_NAMES, 10);
    results.iter().map(|x| x.0).collect()
}

fn main() {
    let app = app::App::default();
    let output: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

    // Main window
    let mut wind = Window::new(100, 100, 400, 300, "Quemoji");
    wind.make_resizable(true);
    let flex = Pack::new(0, 0, 400, 300, "emoji search");

    // Input box at the top
    let mut input = Input::new(10, 10, 380, 30, "");
    input.set_trigger(CallbackTrigger::Changed);
    input.take_focus().expect("get focus");

    let mut label = MultilineOutput::new(10, 50, 830, 270, "results");

    flex.end();
    wind.end();
    wind.show();

    let oc = output.clone();
    input.set_callback(move |i| {
        let value = i.value();
        let results = emoji_search(&value);

        let mut m = oc.lock().unwrap();
        *m = results
            .first()
            .and_then(|x| x.split(' ').next())
            .map(|x| x.to_string());

        let joined = results.join("\n");
        label.set_value(&joined);
    });

    // quit and insert on enter
    input.handle(move |_, ev| {
        if ev == Event::KeyDown && event_key() == Key::Enter {
            app.quit();
        }
        false
    });

    app.run().unwrap();
    // window doesn't disappear until we flush it, and we need to restore focus to whatever was before.
    app::flush();
    let mut m = output.lock().unwrap();
    if let Some(x) = m.take() {
        let mut enigo = Enigo::new(&Settings::default()).unwrap();
        println!("Sending {x}");
        std::thread::sleep(Duration::from_millis(100)); //sometimes it takes a bit for focus to get sorted.
        enigo.text(&x).expect("send keystrokes");
    }
}

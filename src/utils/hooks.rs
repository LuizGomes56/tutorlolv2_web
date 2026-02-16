use wasm_bindgen::{JsCast, prelude::*};
use web_sys::{Element, Event, Node};
use yew::prelude::*;

#[hook]
pub fn use_setter<T: 'static>(value: &UseStateHandle<T>) -> Callback<T> {
    let value = value.clone();
    use_callback((), move |v, _| {
        value.set(v);
    })
}

#[hook]
pub fn use_mouseout<const N: usize>(callback: Callback<()>, exceptions: [NodeRef; N]) -> NodeRef {
    let node_ref = use_node_ref();

    unsafe {
        let node_ref = node_ref.clone();
        let callback = callback.clone();
        let exceptions = exceptions.clone();

        use_effect_with((node_ref.clone(), exceptions.clone()), move |_| {
            let closure = Closure::wrap(Box::new(move |event: Event| {
                let target = match event.target() {
                    Some(t) => t,
                    None => return,
                };

                let target_node = match target.dyn_ref::<Node>() {
                    Some(n) => n,
                    None => return,
                };

                let clicked_in_main = node_ref
                    .cast::<Element>()
                    .map(|el| el.contains(Some(target_node)))
                    .unwrap_or(false);

                let clicked_in_exceptions = exceptions.iter().any(|r| {
                    r.cast::<Element>()
                        .map(|el| el.contains(Some(target_node)))
                        .unwrap_or(false)
                });

                if !clicked_in_main && !clicked_in_exceptions {
                    callback.emit(());
                }
            }) as Box<dyn FnMut(Event)>);

            let window = web_sys::window().unwrap_unchecked();
            let document = window.document().unwrap_unchecked();

            document
                .add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())
                .unwrap_unchecked();

            move || {
                let _ = document.remove_event_listener_with_callback(
                    "mousedown",
                    closure.as_ref().unchecked_ref(),
                );
                drop(closure);
            }
        });
    }

    node_ref
}

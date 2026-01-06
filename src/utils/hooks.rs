use yew::prelude::*;

#[hook]
pub fn use_setter<T: 'static>(value: &UseStateHandle<T>) -> Callback<T> {
    let value = value.clone();
    use_callback((), move |v, _| {
        value.set(v);
    })
}

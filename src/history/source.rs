use std::{cell::RefCell, collections::BTreeMap, rc::Weak};
use tutorlolv2_protocol::*;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys::{self, Function, Promise, Reflect};

#[derive(Clone, Copy, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Command {
    ListHistory,
    GetHistoryDetails,
    GetHistoryFrames,
    SetHistoryShareMode,
    HideHistoryEntry,
    RepairHistory,
}

#[derive(Clone, Copy, strum::IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum Event {
    HistoryChanged,
}

fn error(message: &str) -> HistoryError {
    HistoryError {
        kind: HistoryErrorKind::Other,
        message: message.into(),
        retryable: true,
    }
}

pub fn member(value: &JsValue, key: &str) -> Result<JsValue, HistoryError> {
    Reflect::get(value, &key.into()).map_err(|_| error("Desktop connection unavailable"))
}

pub fn tauri() -> Result<JsValue, HistoryError> {
    member(
        &web_sys::window()
            .ok_or_else(|| error("Window unavailable"))?
            .into(),
        "__TAURI__",
    )
}

fn decode<T: serde::de::DeserializeOwned>(value: &JsValue) -> Result<T, HistoryError> {
    let text = js_sys::JSON::stringify(value)
        .map_err(|_| error("Invalid desktop response"))?
        .as_string()
        .ok_or_else(|| error("Invalid desktop response"))?;

    serde_json::from_str(&text).map_err(|_| error("Invalid desktop response"))
}

impl Command {
    pub async fn invoke<T: serde::de::DeserializeOwned>(
        self,
        args: serde_json::Value,
    ) -> Result<T, HistoryError> {
        decode(&self.invoke_raw(args).await?)
    }
    async fn invoke_raw(self, args: serde_json::Value) -> Result<JsValue, HistoryError> {
        let core = member(&tauri()?, "core")?;
        let invoke: Function = member(&core, "invoke")?
            .dyn_into()
            .map_err(|_| error("Desktop connection unavailable"))?;

        let args = js_sys::JSON::parse(&args.to_string()).map_err(|_| error("Invalid request"))?;
        let result = invoke
            .call2(&core, &JsValue::from_str(self.into()), &args)
            .map_err(|_| error("Desktop request failed"))?;

        let value = JsFuture::from(Promise::resolve(&result))
            .await
            .map_err(|value| {
                decode::<HistoryError>(&value)
                    .unwrap_or_else(|_| error("Desktop request failed. Retry."))
            })?;

        Ok(value)
    }
}

pub trait HistorySource {
    async fn list(offset: u32) -> Result<HistoryList, HistoryError>;
    async fn details(id: &str) -> Result<HistoryDetails, HistoryError>;
    async fn frames(id: &str, seq: u32) -> Result<Vec<u8>, HistoryError>;
    async fn share(id: &str, mode: HistoryShareMode) -> Result<ShareState, HistoryError>;
    async fn hide(id: &str) -> Result<(), HistoryError>;
    async fn repair() -> Result<(), HistoryError>;
}

pub struct IpcHistory;
impl HistorySource for IpcHistory {
    async fn list(offset: u32) -> Result<HistoryList, HistoryError> {
        Command::ListHistory
            .invoke(serde_json::json!({"offset":offset,"limit":50}))
            .await
    }

    async fn details(id: &str) -> Result<HistoryDetails, HistoryError> {
        Command::GetHistoryDetails
            .invoke(serde_json::json!({"recordingId":id}))
            .await
    }

    async fn frames(id: &str, seq: u32) -> Result<Vec<u8>, HistoryError> {
        owner_frames(id, seq).await
    }

    async fn share(id: &str, mode: HistoryShareMode) -> Result<ShareState, HistoryError> {
        Command::SetHistoryShareMode
            .invoke(serde_json::json!({"recordingId":id,"mode":mode}))
            .await
    }

    async fn hide(id: &str) -> Result<(), HistoryError> {
        Command::HideHistoryEntry
            .invoke(serde_json::json!({"recordingId":id}))
            .await
    }

    async fn repair() -> Result<(), HistoryError> {
        Command::RepairHistory.invoke(serde_json::json!({})).await
    }
}

/// Preserve UUID identity rather than treating a local identity as a server ID.
pub fn uuid(id: Id16) -> String {
    let s = id.iter().map(|v| format!("{v:02x}")).collect::<String>();
    format!(
        "{}-{}-{}-{}-{}",
        &s[..8],
        &s[8..12],
        &s[12..16],
        &s[16..20],
        &s[20..]
    )
}

pub fn identity(entry: &HistoryEntry) -> String {
    entry
        .recording_id
        .map(uuid)
        .unwrap_or_else(|| format!("local:{}", entry.local_id.map(uuid).unwrap_or_default()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_commands_are_typed() {
        let name: &str = Command::GetHistoryFrames.into();
        assert_eq!(name, "get_history_frames");
        let event: &str = Event::HistoryChanged.into();
        assert_eq!(event, "history-changed");
    }
    #[test]
    fn uuid_is_canonical() {
        assert_eq!(uuid([0; 16]), "00000000-0000-0000-0000-000000000000");
    }
}

// Native requests cannot be aborted, so concurrent consumers share the in-flight
// transport only. Completed buffers are never kept in this registry.
struct FrameRequest {
    result: Option<Result<Vec<u8>, HistoryError>>,
    waiters: Vec<std::task::Waker>,
}

thread_local! {
    static FRAME_REQUESTS: RefCell<BTreeMap<(String, u32), Weak<RefCell<FrameRequest>>>> = const {
        RefCell::new(BTreeMap::new())
    };
}

struct FrameFuture(std::rc::Rc<std::cell::RefCell<FrameRequest>>);

impl Future for FrameFuture {
    type Output = Result<Vec<u8>, HistoryError>;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut state = self.0.borrow_mut();

        if let Some(result) = &state.result {
            return std::task::Poll::Ready(result.clone());
        }

        if !state.waiters.iter().any(|w| w.will_wake(cx.waker())) {
            state.waiters.push(cx.waker().clone());
        }

        std::task::Poll::Pending
    }
}

async fn owner_frames(id: &str, seq: u32) -> Result<Vec<u8>, HistoryError> {
    let key = (id.to_owned(), seq);
    let (request, start) = FRAME_REQUESTS.with(|requests| {
        let mut requests = requests.borrow_mut();

        if let Some(request) = requests.get(&key).and_then(std::rc::Weak::upgrade) {
            (request, false)
        } else {
            let request = std::rc::Rc::new(std::cell::RefCell::new(FrameRequest {
                result: None,
                waiters: vec![],
            }));

            requests.insert(key.clone(), std::rc::Rc::downgrade(&request));
            (request, true)
        }
    });

    if start {
        let request = request.clone();

        yew::platform::spawn_local(async move {
            let result = async {
                let value = Command::GetHistoryFrames
                    .invoke_raw(serde_json::json!({"recordingId":key.0,"seq":seq}))
                    .await?;

                if value.is_instance_of::<js_sys::ArrayBuffer>()
                    || value.is_instance_of::<js_sys::Uint8Array>()
                {
                    let bytes = js_sys::Uint8Array::new(&value);

                    if bytes.length() > 64 * 1024 * 1024 {
                        return Err(error("Replay chunk exceeds the viewer limit"));
                    }

                    Ok(bytes.to_vec())
                } else {
                    decode::<Vec<u8>>(&value)
                }
            }
            .await;

            let waiters = {
                let mut state = request.borrow_mut();

                state.result = Some(result);
                std::mem::take(&mut state.waiters)
            };

            FRAME_REQUESTS.with(|requests| {
                requests.borrow_mut().remove(&key);
            });

            for waiter in waiters {
                waiter.wake();
            }
        });
    }

    FrameFuture(request).await
}

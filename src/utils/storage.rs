//! Shared versioned Bincode records for browser localStorage.
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use bincode::{Decode, Encode};

const MAGIC: [u8; 4] = *b"TLST";
const VERSION: u8 = 1;
const MAX_RECORD_BYTES: usize = 1_048_576;
pub(crate) const MAX_BASE64_LENGTH: usize = MAX_RECORD_BYTES.div_ceil(3) * 4;

#[derive(Encode, Decode)]
struct Record<T> {
    magic: [u8; 4],
    version: u8,
    kind: u8,
    value: T,
}

pub(crate) fn encode_record(kind: u8, value: &impl Encode) -> Option<String> {
    let record = Record {
        magic: MAGIC,
        version: VERSION,
        kind,
        value,
    };
    let bytes = bincode::encode_to_vec(record, bincode::config::standard()).ok()?;
    (bytes.len() <= MAX_RECORD_BYTES).then(|| BASE64.encode(bytes))
}

pub(crate) fn decode_record<T: Decode<()>>(kind: u8, raw: &str) -> Option<T> {
    if raw.len() > MAX_BASE64_LENGTH {
        return None;
    }

    let bytes = BASE64.decode(raw).ok()?;
    let (record, read): (Record<T>, _) = bincode::decode_from_slice(
        &bytes,
        bincode::config::standard().with_limit::<MAX_RECORD_BYTES>(),
    )
    .ok()?;

    (read == bytes.len()
        && record.magic == MAGIC
        && record.version == VERSION
        && record.kind == kind)
        .then_some(record.value)
}

pub(crate) trait RecordStorage {
    fn get(&self, key: &str) -> Option<String>;
    fn put(&mut self, key: &str, value: &str) -> bool;
    fn remove(&mut self, key: &str);
}

impl RecordStorage for web_sys::Storage {
    fn get(&self, key: &str) -> Option<String> {
        self.get_item(key).ok().flatten()
    }

    fn put(&mut self, key: &str, value: &str) -> bool {
        self.set_item(key, value).is_ok()
    }

    fn remove(&mut self, key: &str) {
        let _ = self.remove_item(key);
    }
}

pub(crate) fn read_record<T: Decode<()>>(
    storage: &mut impl RecordStorage,
    key: &str,
    kind: u8,
) -> Option<T> {
    let raw = storage.get(key)?;
    let value = decode_record(kind, &raw);

    if value.is_none() {
        storage.remove(key);
    }

    value
}

pub(crate) fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

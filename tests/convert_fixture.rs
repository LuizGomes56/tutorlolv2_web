//! Run explicitly: cargo test --offline --features tauri --test convert_fixture -- --ignored
#![cfg(feature = "tauri")]

use std::{fs, path::Path};
use tutorlolv2::riot::RiotRealtime;

#[test]
#[ignore = "writes example.bincode from example.json"]
fn convert_json_fixture_to_bincode() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let json = fs::read(root.join("example.json"))?;
    let input: RiotRealtime<'_> = serde_json::from_slice(&json)?;
    let config = bincode::config::standard();
    let bytes = bincode::encode_to_vec(&input, config)?;

    let (decoded, consumed): (RiotRealtime<'_>, _) =
        bincode::borrow_decode_from_slice(&bytes, config)?;
    assert_eq!(consumed, bytes.len());
    assert_eq!(decoded, input);

    fs::write(root.join("example.bincode"), &bytes)?;
    println!(
        "Converted example.json to example.bincode ({} bytes)",
        bytes.len()
    );
    Ok(())
}

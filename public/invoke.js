const invoke = window.__TAURI_INTERNALS__?.invoke;

export async function invoke_get_live_game() {
    if (!invoke || typeof invoke !== "function") {
        return new Uint8Array();
    }

    console.log("[call] invoke_get_live_game");

    /** @type {ArrayBuffer} */
    let data = await invoke?.("get_live_game");

    console.log("[data]", data);

    return new Uint8Array(data);
}

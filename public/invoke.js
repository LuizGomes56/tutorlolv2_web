const invoke = window.__TAURI_INTERNALS__?.invoke;

export async function invoke_get_live_game() {
    if (!invoke || typeof invoke !== "function") {
        return new Uint8Array();
    }

    console.log("[call] invoke_get_live_game");

    /** @type {ArrayBuffer} */
    let data = await invoke?.("get_live_game");

    return new Uint8Array(data);
}

export function blur_overlay() {
    invoke?.("blur_overlay");
}

export async function listen(event, callback) {
    const f = window?.__TAURI__?.event?.listen;

    if (!callback || !f) {
        return null;
    }

    return await f(event, (e) => {
        console.log(`Listener for ${event} triggered with`, e);
        callback();
    });
}


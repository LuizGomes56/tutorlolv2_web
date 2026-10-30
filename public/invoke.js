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
        callback(e.payload);
    });
}

function desktopInvoke(command, args) {
    if (typeof invoke !== "function")
        return Promise.reject(
            new Error(
                "Desktop controls are unavailable. Open this page in the desktop app.",
            ),
        );
    return invoke(command, args);
}

export function save_overlay_shortcut(shortcut) {
    return desktopInvoke("set_overlay_shortcut", { shortcut });
}

export function toggle_overlay() {
    return desktopInvoke("toggle_overlay");
}

export function close_overlay() {
    return desktopInvoke("blur_overlay");
}

let activeRecorders = 0;

// Subscribe before reading the snapshot. Never let a delayed snapshot overwrite a newer event.
function watchState(event, command, callback, onError) {
    let disposed = false;
    let unlisten;
    let changed = false;

    const listen = window.__TAURI__?.event?.listen;

    (async () => {
        try {
            if (typeof listen !== "function")
                throw new Error("Desktop events are unavailable.");

            const remove = await listen(event, (e) => {
                changed = true;
                if (!disposed) callback(e.payload);
            });

            if (disposed) {
                remove();
                return;
            }

            unlisten = remove;

            if (command) {
                const value = await desktopInvoke(command);

                if (!disposed && !changed) callback(value);
            }
        } catch (error) {
            if (!disposed) onError(String(error));
        }
    })();

    return () => {
        disposed = true;
        unlisten?.();
        unlisten = undefined;
    };
}

export function watch_overlay_focus(callback, onError) {
    let focused = false;
    let revision = 0;
    let disposed = false;
    let closing = false;

    const update = (value) => {
        revision++;
        focused = Boolean(value);
        callback(focused);
    };

    const stop = watchState("focus", "get_overlay_focus", update, onError);
    const keydown = (event) => {
        if (
            event.code !== "Escape" ||
            !focused ||
            closing ||
            activeRecorders ||
            event.repeat
        ) {
            return;
        }

        event.preventDefault();
        event.stopPropagation();

        closing = true;
        const closeRevision = revision;

        // The native hook handles Escape in the game. Also close locally when a
        // focused webview receives it, including inputs forwarded by keyboard tools.
        close_overlay()
            .then(() => {
                if (!disposed && focused && revision === closeRevision)
                    update(false);
            })
            .catch((error) => {
                if (!disposed) onError(String(error));
            })
            .finally(() => {
                closing = false;
            });
    };

    window.addEventListener("keydown", keydown, true);

    return () => {
        disposed = true;
        stop();
        window.removeEventListener("keydown", keydown, true);
    };
}
export function watch_overlay_shortcut(callback, onError) {
    return watchState(
        "overlay-shortcut-changed",
        "get_overlay_shortcut",
        callback,
        onError,
    );
}
export function watch_overlay_change(callback, onError) {
    return watchState("change", null, callback, onError);
}

export function format_shortcut(shortcut) {
    if (!shortcut) return "Loading...";

    const names = {
        Space: "Space",
        Backquote: "`",
        Minus: "-",
        Equal: "=",
        BracketLeft: "[",
        BracketRight: "]",
        Backslash: "\\",
        Semicolon: ";",
        Quote: "'",
        Comma: ",",
        Period: ".",
        Slash: "/",
    };

    const key =
        names[shortcut.code] ??
        shortcut.code.replace(/^Key|^Digit/, "").replace(/^Arrow/, "");

    return [
        shortcut.ctrl && "Ctrl",
        shortcut.alt && "Alt",
        shortcut.shift && "Shift",
        shortcut.meta && "Win",
        key,
    ]
        .filter(Boolean)
        .join(" + ");
}

// The backend lease expires after 30 seconds even if a webview disappears mid-recording.
export function start_shortcut_recording(
    onCaptured,
    onError,
    onFinished,
    onReady,
) {
    activeRecorders++;

    let disposed = false;
    let timer;

    const stop = () => {
        if (disposed) return;

        disposed = true;
        activeRecorders--;

        clearTimeout(timer);

        window.removeEventListener("keydown", keydown, true);
        window.removeEventListener("blur", cancelled);

        // Best effort: the native focus-loss handler and expiring lease also release recording.
        desktopInvoke("set_shortcut_recording", { recording: false }).catch(
            () => {},
        );
    };

    const cancelled = () => {
        stop();
        onFinished();
    };

    const keydown = (event) => {
        event.preventDefault();
        event.stopImmediatePropagation();

        if (
            event.repeat ||
            ["Control", "Shift", "Alt", "Meta", "AltGraph"].includes(event.key)
        )
            return;

        if (event.code === "Escape") {
            cancelled();
            return;
        }

        onCaptured({
            code: event.code,
            ctrl: event.ctrlKey,
            alt: event.altKey,
            shift: event.shiftKey,
            meta: event.metaKey,
        });

        cancelled();
    };

    // Cancellation must be armed before the asynchronous native startup.
    window.addEventListener("blur", cancelled);
    timer = setTimeout(cancelled, 25_000);

    desktopInvoke("set_shortcut_recording", { recording: true })
        .then(() => {
            if (disposed) return;
            if (window.document?.hasFocus && !window.document.hasFocus()) {
                cancelled();
                return;
            }

            window.addEventListener("keydown", keydown, true);
            onReady?.();
        })
        .catch((error) => {
            if (!disposed) {
                onError(String(error));
                cancelled();
            }
        });

    return stop;
}

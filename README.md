# TutorLoL frontend

The Yew frontend supports two calculation modes at compile time. Both use the same
shared input and output models. Local calculation is the default, so existing
build commands continue to calculate directly in WASM:

```sh
trunk serve
trunk build --release
```

Build without default features to use the calculation HTTP service and leave the
damage engine out of WASM:

```sh
trunk serve --no-default-features
trunk build --release --no-default-features
```

Remote builds send Bincode POST requests to `/calculator` and `/realtime`, under
`http://127.0.0.1:8082/api/games` by default. Override the base URL at build time:

```sh
TUTORLOL_API_URL=https://example.com/api/games trunk build --release --no-default-features
```

Start the library repository's calculation server before using a remote build.
Configure its CORS origins to include the frontend origin when hosted separately.
Requests and successful responses use `application/octet-stream` with
`bincode::config::standard()`. Calculator requests encode `InputGame` (decoded by
the service as its wire-compatible `OwnedInputGame`); responses encode
`OutputGame`. Realtime requests encode `RiotRealtime`; responses encode `Realtime`.
The client checks HTTP status, payload decoding, and complete payload consumption.
Failed calculator requests appear in the page, and replies from superseded inputs
cannot overwrite newer results.

Browser builds use the checked-in `example.bincode` fixture for `/livegame` in
either calculation mode. Enable the independent `tauri` feature for desktop
builds to fetch actual game data through the `get_live_game` JS bridge and enable
the `/overlay` route:

```sh
trunk build --release --features tauri
trunk build --release --no-default-features --features tauri
```

With `local-calculation`, the desktop bridge returns Riot JSON bytes, which the
frontend deserializes into a borrowed `RiotRealtime` and calculates in WASM.
Without `local-calculation`, the bridge must supply Bincode for the calculation
service. Desktop builds propagate
bridge errors instead of falling back to the sample. The overlay is excluded
from builds without `tauri`.

For Windows Tauri development with the frontend in WSL, use this `build` section
in `tauri.conf.json`:

```json
{
  "frontendDist": "\\\\wsl.localhost\\Ubuntu\\home\\luizg\\Projects\\tutorlolv2_web\\dist",
  "devUrl": "http://localhost:8000",
  "beforeDevCommand": "wsl.exe --cd /home/luizg/Projects/tutorlolv2_web --exec /usr/bin/env PATH=/home/luizg/.cargo/bin:/usr/local/bin:/usr/bin:/bin /home/luizg/.cargo/bin/trunk serve --port 8000 --features tauri --minify false",
  "beforeBuildCommand": "wsl.exe --cd /home/luizg/Projects/tutorlolv2_web --exec /usr/bin/env PATH=/home/luizg/.cargo/bin:/usr/local/bin:/usr/bin:/bin /home/luizg/.cargo/bin/trunk build --release --features tauri"
}
```

Keep development HTML unminified: Trunk's minifier can fold its reload hostname
placeholder into the WebSocket URL before the dev server replaces it. This
produces repeated connections to `ws://{{__TRUNK_ADDRESS__}}/...`. Release builds
remain minified and have no reload client. Restart Tauri development after
changing the command, then reload the overlay window.

The overlay window must use the app URL `overlay` (or
`WebviewUrl::App("overlay".into())` when created in Rust) and `transparent: true`.
The frontend renders `/overlay` without the sidebar or opaque page shell; all
other routes retain the normal application layout. If `/overlay` renders the
calculator itself, check the window's actual URL in DevTools and ensure another
Trunk process or build is not overwriting the same `dist` directory without the
`tauri` feature.

Formula documentation and hover rendering remain available in both modes through
the library's `client` feature. Calculation mode requires no runtime settings.

`/watch` and `/watch/:code` are available in every build. They read capability-only
sharing endpoints; local calculation remains the default and remote calculation
builds send selected decoded frames through the calculation service. Set the
sharing API base at build time with `TUTORLOLV2_API_URL=https://example.com/api`.
Otherwise it uses `TUTORLOL_API_URL` with the trailing `/games` removed, defaulting
to `http://127.0.0.1:8082/api`. Native sharing controls appear only on the main
`/livegame` page and use `get_recording_state`, `set_share_mode`, and the native
recording/share state events.

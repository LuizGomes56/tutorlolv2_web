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

The desktop bridge must return a Bincode `RiotRealtime` payload. With
`local-calculation`, the frontend calculates from that payload in WASM; without
it, the payload is sent to the calculation service. Desktop builds propagate
bridge errors instead of falling back to the sample. The overlay is excluded
from builds without `tauri`. No JSON parsing is required in either calculation
mode.

Formula documentation and hover rendering remain available in both modes through
the library's `client` feature. Calculation mode requires no runtime settings.

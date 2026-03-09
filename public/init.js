const init_events = (wasm) => {
    window.wasm = wasm || null;
    if (wasm && typeof wasm.cache_ptr === "function" && typeof wasm.cache_len === "function") {
        const ptr = wasm.cache_ptr();
        const len = wasm.cache_len();
        let buf = wasm.memory.buffer;
        let view = new Uint8Array(buf, ptr, len);

        function ensureView() {
            if (buf !== wasm.memory.buffer) {
                buf = wasm.memory.buffer;
                view = new Uint8Array(buf, ptr, len);
            }
            return view;
        }

        const textDecoder = new TextDecoder("utf-8");
        window.decodeCacheSlice = function (s, e) {
            const arrView = ensureView();
            return textDecoder.decode(arrView.subarray(s, e));
        };
    } else {
        if (typeof window.decodeCacheSlice !== "function") {
            window.decodeCacheSlice = function (s, e) {
                return [
                    `<pre>`,
                    `<span class="keyword">undefined function</span> `,
                    `<span class="function">decodeCacheSlice</span>`,
                    `<span class="bracket_1">(</span>`,
                    `<span class="variable">s</span>, `,
                    `<span class="variable">e</span>`,
                    `<span class="bracket_1">)</span> `,
                    `<span class="bracket_1">{</span>`,
                    `\n\t<span class="comment">/* Unable to load hover docs */</span>\n\t`,
                    `<span class="type">Wasm</span>::`,
                    `<span class="function">hoverdocs</span>`,
                    `<span class="bracket_2">(</span>`,
                    `<span class="number">${s}</span>..`,
                    `<span class="number">${e}</span>`,
                    `<span class="bracket_2">)</span>\n`,
                    `<span class="bracket_1">}</span>`,
                    `</pre>`
                ].join("");
            };
        }
    }
}
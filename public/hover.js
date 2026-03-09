const init_hover = () => {
    let shiftDown = false;
    const openTooltips = new WeakMap();

    function parseOffsets(host) {
        return (host.getAttribute("data_offset") || "")
            .split("|")
            .filter(Boolean)
            .map(range => range.split("..").map(Number))
            .filter(([s, e]) => Number.isFinite(s) && Number.isFinite(e));
    }

    function ensureHostPositioned(host) {
        if (!(host instanceof Element)) return;
        const cs = getComputedStyle(host);
        if (cs.position === "static") {
            host.style.position = "relative";
        }
    }

    function ensureAllDataOffsetHostsPositioned() {
        document.querySelectorAll("[data_offset]").forEach(ensureHostPositioned);
    }

    function getHoverPortalRoot() {
        let root = document.getElementById("hover-docs-portal-root");
        if (!root) {
            root = document.createElement("div");
            root.id = "hover-docs-portal-root";
            root.style.position = "fixed";
            root.style.inset = "0";
            root.style.pointerEvents = "none";
            root.style.zIndex = "2147483647";
            root.style.overflow = "visible";
            document.body.appendChild(root);
        }
        return root;
    }

    function closeTooltipForHost(host) {
        const entry = openTooltips.get(host);
        if (!entry) return;
        try {
            entry.cleanup?.();
        } catch (_) { }
    }

    function closeAllTooltips() {
        document.querySelectorAll(".hover-docs").forEach(node => {
            if (node && node.isConnected) node.remove();
        });
    }

    function showOn(host) {
        if (!(host instanceof Element)) return;

        const existing = openTooltips.get(host);
        if (existing?.node?.isConnected) return;

        const offsets = parseOffsets(host);

        ensureHostPositioned(host);

        const t = document.createElement("div");
        const extra = host.getAttribute("data-classes") || "";
        t.className =
            "flex flex-col absolute max-w-md max-h-96 overflow-auto p-2 " +
            "leading-6 text-base z-50 hover-docs border border-std-800 bg-std-900" +
            (extra ? " " + extra : "");
        t.style.pointerEvents = "auto";

        const data_idents = host.getAttribute("data_idents") || "";
        const idents = data_idents
            .split("|")
            .filter(Boolean)
            .map(entry => {
                const [ctx_var, ctx_value] = entry.split(":");
                return `<div class="whitespace-nowrap"><span class="variable">${ctx_var}</span> = <span class="number">${ctx_value ?? "?"}</span></div>`;
            })
            .join("");

        let innerHTML = idents.length ? `<div class="flex flex-col">${idents}</div>` : "";

        const code = document.createElement("code");
        code.className = "flex flex-col gap-2 text-[#D4D4D4] font-normal text-left text-wrap";

        for (const [s, e] of offsets) {
            innerHTML += window.decodeCacheSlice(s, e);
        }

        code.innerHTML = innerHTML;
        t.appendChild(code);

        host.appendChild(t);

        const tipRect = t.getBoundingClientRect();
        const hostRect = host.getBoundingClientRect();

        const dx = tipRect.left - hostRect.left;
        const dy = tipRect.top - hostRect.top;

        const portalRoot = getHoverPortalRoot();

        t.classList.remove("absolute");

        t.style.position = "fixed";
        t.style.left = `${Math.round(tipRect.left)}px`;
        t.style.top = `${Math.round(tipRect.top)}px`;
        t.style.margin = "0";
        t.style.zIndex = "2147483647";
        t.style.pointerEvents = "auto";

        portalRoot.appendChild(t);

        const updatePosition = () => {
            if (!host.isConnected || !t.isConnected) return;
            const r = host.getBoundingClientRect();
            t.style.left = `${Math.round(r.left + dx)}px`;
            t.style.top = `${Math.round(r.top + dy)}px`;
        };

        const isInsideSafe = (node) => {
            if (!(node instanceof Node)) return false;
            if (host.contains(node)) return true;
            if (t.contains(node)) return true;
            if (node instanceof Element && node.closest(".hoverdocs, .hover-docs")) return true;
            return false;
        };

        const onGlobalPointerOut = (ev) => {
            const from = ev.target;
            const to = ev.relatedTarget;

            if (!isInsideSafe(from)) return;
            if (to && isInsideSafe(to)) return;

            cleanup();
        };

        const onKeyDownClose = (ev) => {
            if (ev.key === "Escape") cleanup();
        };

        const onWindowBlur = () => cleanup();

        const onScrollOrResize = () => updatePosition();

        function cleanup() {
            const current = openTooltips.get(host);
            if (current?.node === t) {
                openTooltips.delete(host);
            }

            document.removeEventListener("pointerout", onGlobalPointerOut, true);
            document.removeEventListener("keydown", onKeyDownClose, true);
            window.removeEventListener("blur", onWindowBlur, true);
            window.removeEventListener("scroll", onScrollOrResize, true);
            window.removeEventListener("resize", onScrollOrResize, true);

            if (t.isConnected) t.remove();
        }

        document.addEventListener("pointerout", onGlobalPointerOut, true);
        document.addEventListener("keydown", onKeyDownClose, true);
        window.addEventListener("blur", onWindowBlur, true);
        window.addEventListener("scroll", onScrollOrResize, true);
        window.addEventListener("resize", onScrollOrResize, true);

        openTooltips.set(host, { node: t, cleanup });
        requestAnimationFrame(updatePosition);
    }
    ensureAllDataOffsetHostsPositioned();
    const mo = new MutationObserver((mutations) => {
        for (const m of mutations) {
            if (m.type !== "childList") continue;

            for (const n of m.addedNodes) {
                if (!(n instanceof Element)) continue;

                if (n.matches?.("[data_offset]")) ensureHostPositioned(n);
                n.querySelectorAll?.("[data_offset]").forEach(ensureHostPositioned);
            }

            for (const n of m.removedNodes) {
                if (!(n instanceof Element)) continue;
                if (n.matches?.("[data_offset]")) closeTooltipForHost(n);
                n.querySelectorAll?.("[data_offset]").forEach(closeTooltipForHost);
            }
        }
    });

    mo.observe(document.documentElement, {
        childList: true,
        subtree: true
    });

    window.addEventListener("keydown", (e) => {
        if (e.key !== "Shift") return;
        if (shiftDown) return;
        shiftDown = true;

        ensureAllDataOffsetHostsPositioned();

        const hovered = document.querySelectorAll("[data_offset]:hover");
        if (!hovered.length) return;

        const host = hovered[hovered.length - 1];
        showOn(host);
    }, true);

    window.addEventListener("keyup", (e) => {
        if (e.key === "Shift") {
            shiftDown = false;
            // Closes when shift is released
            // closeAllTooltips();
        }
    }, true);

    document.addEventListener(
        "mouseover",
        (e) => {
            if (!shiftDown) return;

            const el = e.target;
            if (!(el instanceof Element)) return;

            const host = el.closest?.("[data_offset]");
            if (!host) return;

            ensureHostPositioned(host);

            const from = e.relatedTarget;
            if (from && from instanceof Node && host.contains(from)) return;

            showOn(host);
        },
        { capture: true }
    );

    // debug info
    // window.__hoverDocs = {
    //     showOn,
    //     closeAllTooltips,
    //     ensureAllDataOffsetHostsPositioned
    // };
}
const init_hover = () => {
    let shiftDown = false;
    let currentHost = null;

    const tip = document.getElementById("hoverdocs");
    const code = document.getElementById("hoverdocs_code");

    if (!tip || !code) {
        console.warn("[hoverdocs] Missing #hoverdocs or #hoverdocs_code");
        return;
    }

    const baseTipClasses = new Set(tip.className.split(/\s+/).filter(Boolean));
    const extraClasses = new Set();

    function parseOffsets(host) {
        return (host.getAttribute("data_offset") || "")
            .split("|")
            .filter(Boolean)
            .map(r => r.split("..").map(Number))
            .filter(([s, e]) => Number.isFinite(s) && Number.isFinite(e));
    }

    function ensureHostPositioned(host) {
        if (!(host instanceof Element)) return;
        const cs = getComputedStyle(host);
        if (cs.position === "static") host.style.position = "relative";
    }

    function ensureAllDataOffsetHostsPositioned() {
        document.querySelectorAll("[data_offset]").forEach(ensureHostPositioned);
    }

    function clearExtraClasses() {
        for (const cls of extraClasses) tip.classList.remove(cls);
        extraClasses.clear();
    }

    function applyExtraClasses(host) {
        clearExtraClasses();
        const extra = (host.getAttribute("data-classes") || "").trim();
        if (!extra) return;

        for (const cls of extra.split(/\s+/).filter(Boolean)) {
            if (!baseTipClasses.has(cls)) {
                tip.classList.add(cls);
                extraClasses.add(cls);
            }
        }
    }

    function setTooltipContent(host) {
        const offsets = parseOffsets(host);

        const data_idents = host.getAttribute("data_idents") || "";
        const identsHtml = data_idents
            .split("|")
            .filter(Boolean)
            .map(entry => {
                const [ctx_var, ctx_value] = entry.split(":");
                return `<div class="whitespace-nowrap"><span class="variable">${ctx_var}</span> = <span class="number">${ctx_value ?? "?"}</span></div>`;
            })
            .join("");

        let html = identsHtml ? `<div class="flex flex-col">${identsHtml}</div>` : "";

        for (const [s, e] of offsets) {
            html += window.decodeCacheSlice(s, e);
        }

        return html;
    }

    function positionTooltip(host) {
        const r = host.getBoundingClientRect();
        tip.style.left = `${Math.round(r.left)}px`;
        tip.style.top = `${Math.round(r.bottom)}px`;
    }

    function showOn(host) {
        if (!(host instanceof Element)) return;
        ensureHostPositioned(host);

        applyExtraClasses(host);
        code.innerHTML = setTooltipContent(host);

        currentHost = host;
        tip.classList.remove("hidden");
        tip.style.display = "";

        positionTooltip(host);
    }

    function hideTooltip() {
        tip.classList.add("hidden");
        code.innerHTML = "";
        clearExtraClasses();
        currentHost = null;
    }

    function isInsideSafe(node) {
        if (!(node instanceof Node)) return false;
        if (currentHost && currentHost.contains(node)) return true;
        if (tip.contains(node)) return true;
        return false;
    }

    window.addEventListener("keydown", (e) => {
        if (e.key !== "Shift") return;
        if (shiftDown) return;
        shiftDown = true;

        ensureAllDataOffsetHostsPositioned();

        const hovered = document.querySelectorAll("[data_offset]:hover");
        if (!hovered.length) return;

        showOn(hovered[hovered.length - 1]);
    }, true);

    window.addEventListener("keyup", (e) => {
        if (e.key === "Shift") shiftDown = false;
    }, true);

    document.addEventListener("mouseover", (e) => {
        if (!shiftDown) return;

        const el = e.target;
        if (!(el instanceof Element)) return;

        const host = el.closest?.("[data_offset]");
        if (!host) return;

        const from = e.relatedTarget;
        if (from && from instanceof Node && host.contains(from)) return;

        showOn(host);
    }, { capture: true });

    document.addEventListener("pointerout", (ev) => {
        const from = ev.target;
        const to = ev.relatedTarget;
        if (!isInsideSafe(from)) return;
        if (to && isInsideSafe(to)) return;
        hideTooltip();
    }, true);

    window.addEventListener("blur", hideTooltip, true);

    const reposition = () => {
        if (!currentHost || tip.classList.contains("hidden")) return;
        positionTooltip(currentHost);
    };

    window.addEventListener("scroll", reposition, true);
    window.addEventListener("resize", reposition, true);

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
                if (currentHost && (n === currentHost || n.contains(currentHost))) hideTooltip();
            }
        }
    });

    mo.observe(document.documentElement, { childList: true, subtree: true });
};
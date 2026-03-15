export function mouse_events() {
    document.body.style.backgroundColor = "transparent";
    document.documentElement.style.backgroundColor = "transparent";

    if (globalThis.__mouse_events_initialized) return;
    globalThis.__mouse_events_initialized = true;

    let state = null;

    function clamp(value, min, max) {
        return Math.max(min, Math.min(max, value));
    }

    function getNumber(value, fallback = 0) {
        const n = Number(value);
        return Number.isFinite(n) ? n : fallback;
    }

    function getPanelExtraSize(panel) {
        const styles = getComputedStyle(panel);

        const extraX =
            parseFloat(styles.paddingLeft) +
            parseFloat(styles.paddingRight) +
            parseFloat(styles.borderLeftWidth) +
            parseFloat(styles.borderRightWidth);

        const extraY =
            parseFloat(styles.paddingTop) +
            parseFloat(styles.paddingBottom) +
            parseFloat(styles.borderTopWidth) +
            parseFloat(styles.borderBottomWidth);

        return { extraX, extraY };
    }

    function getPanelStorageKey(panel) {
        const id = panel.dataset.panelId || panel.id;
        if (!id) return null;
        return `panel-state:${id}`;
    }

    function setActivePanel(activePanel) {
        for (const panel of document.querySelectorAll('[data-panel="true"]')) {
            panel.dataset.active = panel === activePanel ? "true" : "false";
        }
    }

    function clearActivePanels() {
        for (const panel of document.querySelectorAll('[data-panel="true"]')) {
            panel.dataset.active = "false";
        }
    }

    function ensureMetrics(panel) {
        const content = panel.querySelector('[data-panel-content="true"]');
        if (!content) return null;

        let baseWidth = getNumber(panel.dataset.baseWidth, 0);
        let baseHeight = getNumber(panel.dataset.baseHeight, 0);

        if (!baseWidth || !baseHeight) {
            baseWidth = Math.max(content.scrollWidth, content.offsetWidth, 1);
            baseHeight = Math.max(content.scrollHeight, content.offsetHeight, 1);

            panel.dataset.baseWidth = String(baseWidth);
            panel.dataset.baseHeight = String(baseHeight);
        }

        return {
            content,
            baseWidth,
            baseHeight,
            scale: getNumber(panel.dataset.scale, 1),
        };
    }

    function applyScale(panel, scale) {
        const metrics = ensureMetrics(panel);
        if (!metrics) return;

        const { content, baseWidth, baseHeight } = metrics;
        const { extraX, extraY } = getPanelExtraSize(panel);

        panel.dataset.scale = String(scale);

        content.style.transformOrigin = "top left";
        content.style.transform = `scale(${scale})`;

        panel.style.width = `${baseWidth * scale + extraX}px`;
        panel.style.height = `${baseHeight * scale + extraY}px`;
    }

    function savePanelState(panel) {
        const key = getPanelStorageKey(panel);
        if (!key) return;

        const payload = {
            left: panel.style.left || "",
            top: panel.style.top || "",
            scale: getNumber(panel.dataset.scale, 1),
        };

        localStorage.setItem(key, JSON.stringify(payload));
    }

    function loadPanelState(panel) {
        const key = getPanelStorageKey(panel);
        if (!key) return;

        const raw = localStorage.getItem(key);
        if (!raw) {
            applyScale(panel, getNumber(panel.dataset.scale, 1));
            return;
        }

        try {
            const saved = JSON.parse(raw);

            if (saved.left || saved.top) {
                panel.style.position = "absolute";
            }

            if (saved.left) {
                panel.style.left = saved.left;
            }

            if (saved.top) {
                panel.style.top = saved.top;
            }

            const scale = clamp(getNumber(saved.scale, 1), 0.35, 2);
            applyScale(panel, scale);
        } catch {
            applyScale(panel, getNumber(panel.dataset.scale, 1));
        }
    }

    function restoreAllPanels() {
        for (const panel of document.querySelectorAll('[data-panel="true"]')) {
            loadPanelState(panel);
        }
    }

    function startDrag(panel, e) {
        const rect = panel.getBoundingClientRect();

        panel.style.position = "absolute";
        panel.style.left = `${rect.left + window.scrollX}px`;
        panel.style.top = `${rect.top + window.scrollY}px`;

        state = {
            mode: "drag",
            panel,
            pointerId: e.pointerId,
            offsetX: e.clientX - rect.left,
            offsetY: e.clientY - rect.top,
        };
    }

    function startScale(panel, handle, e) {
        const metrics = ensureMetrics(panel);
        if (!metrics) return;

        const rect = panel.getBoundingClientRect();

        state = {
            mode: "scale",
            panel,
            handle,
            pointerId: e.pointerId,
            startX: e.clientX,
            startY: e.clientY,
            startLeft: rect.left + window.scrollX,
            startTop: rect.top + window.scrollY,
            startScale: getNumber(panel.dataset.scale, 1),
            baseWidth: metrics.baseWidth,
            baseHeight: metrics.baseHeight,
        };
    }

    document.addEventListener("pointerdown", e => {
        if (e.button !== 0) return;

        const handleEl = e.target.closest?.('[data-resize-handle]');
        const panel = e.target.closest?.('[data-panel="true"]');

        if (!panel) {
            clearActivePanels();
            return;
        }

        setActivePanel(panel);
        e.preventDefault();

        if (handleEl) {
            e.stopPropagation();
            startScale(panel, handleEl.dataset.resizeHandle, e);
        } else {
            startDrag(panel, e);
        }
    });

    document.addEventListener("pointermove", e => {
        if (!state) return;
        if (e.pointerId !== state.pointerId) return;

        if (state.mode === "drag") {
            state.panel.style.left = `${e.clientX - state.offsetX + window.scrollX}px`;
            state.panel.style.top = `${e.clientY - state.offsetY + window.scrollY}px`;
            return;
        }

        if (state.mode === "scale") {
            const dx = e.clientX - state.startX;
            const dy = e.clientY - state.startY;

            const startWidth = state.baseWidth * state.startScale;
            const startHeight = state.baseHeight * state.startScale;

            let widthDelta = 0;
            let heightDelta = 0;

            if (state.handle.includes("e")) widthDelta = dx;
            if (state.handle.includes("w")) widthDelta = -dx;
            if (state.handle.includes("s")) heightDelta = dy;
            if (state.handle.includes("n")) heightDelta = -dy;

            const scaleX = (startWidth + widthDelta) / state.baseWidth;
            const scaleY = (startHeight + heightDelta) / state.baseHeight;

            let nextScale =
                Math.abs(scaleX - state.startScale) > Math.abs(scaleY - state.startScale)
                    ? scaleX
                    : scaleY;

            nextScale = clamp(nextScale, 0.35, 2);

            const newWidth = state.baseWidth * nextScale;
            const newHeight = state.baseHeight * nextScale;

            state.panel.style.left = state.handle.includes("w")
                ? `${state.startLeft + (startWidth - newWidth)}px`
                : `${state.startLeft}px`;

            state.panel.style.top = state.handle.includes("n")
                ? `${state.startTop + (startHeight - newHeight)}px`
                : `${state.startTop}px`;

            applyScale(state.panel, nextScale);
        }
    });

    function endInteraction(e) {
        if (!state) return;
        if (e.pointerId !== state.pointerId) return;

        savePanelState(state.panel);
        state = null;
    }

    document.addEventListener("pointerup", endInteraction);
    document.addEventListener("pointercancel", endInteraction);

    requestAnimationFrame(() => {
        requestAnimationFrame(() => {
            restoreAllPanels();
        });
    });

    const observer = new MutationObserver(() => {
        restoreAllPanels();
    });

    if (document.body) {
        observer.observe(document.body, {
            childList: true,
            subtree: true,
        });
    }
}

globalThis.mouse_events = mouse_events;

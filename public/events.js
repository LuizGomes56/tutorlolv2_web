const clamp = (value, min, max) => Math.max(min, Math.min(max, value));
const number = (value, fallback = 0) =>
    Number.isFinite(Number(value)) ? Number(value) : fallback;

// Project the pointer onto the panel diagonal, keeping the opposite corner fixed.
export function resize_geometry(start, dx, dy) {
    const west = start.handle.includes("w");
    const north = start.handle.includes("n");
    const delta =
        ((west ? -dx : dx) * start.baseWidth +
            (north ? -dy : dy) * start.baseHeight) /
        (start.baseWidth ** 2 + start.baseHeight ** 2);

    const scale = clamp(start.scale + delta, 0.35, 2);

    return {
        left: start.left + (west ? start.baseWidth * (start.scale - scale) : 0),
        top: start.top + (north ? start.baseHeight * (start.scale - scale) : 0),
        scale,
    };
}

export function mouse_events(savePanelRecord, loadPanelRecord) {
    if (globalThis.__mouse_events_initialized) return;
    globalThis.__mouse_events_initialized = true;

    let interaction = null;
    let restoreFrame = 0;
    const restored = new WeakSet();
    const contentOf = (panel) =>
        panel.querySelector(':scope > [data-panel-content="true"]');
    const storageKey = (panel) =>
        `panel-state:${panel.dataset.panelId || panel.id}`;

    function applyScale(panel, scale) {
        const content = contentOf(panel);
        if (!content) return;
        panel.dataset.scale = String(scale);
        // Unlike transform, zoom participates in intrinsic layout. The border follows
        // the actual content as tables, combo entries, fonts and images change size.
        content.style.zoom = String(scale);
    }

    function save(panel) {
        savePanelRecord(
            storageKey(panel),
            panel.style.left || "",
            panel.style.top || "",
            number(panel.dataset.scale, 1),
        );
    }

    function restorePanels() {
        restoreFrame = 0;
        for (const panel of document.querySelectorAll('[data-panel="true"]')) {
            if (restored.has(panel) || !contentOf(panel)) continue;
            const saved = loadPanelRecord(storageKey(panel));
            if (saved) {
                const [left, top, scale] = saved;
                if (left || top) panel.style.position = "absolute";
                if (left) panel.style.left = left;
                if (top) panel.style.top = top;
                applyScale(panel, clamp(number(scale, 1), 0.35, 2));
            } else {
                applyScale(panel, 1);
            }
            restored.add(panel);
        }
    }

    function scheduleRestore() {
        if (!restoreFrame) restoreFrame = requestAnimationFrame(restorePanels);
    }

    function select(panel) {
        for (const candidate of document.querySelectorAll(
            '[data-panel="true"]',
        )) {
            candidate.dataset.active = String(candidate === panel);
        }
    }

    function placeAbsolutely(panel, rect) {
        panel.style.position = "absolute";
        const parent = panel.offsetParent;
        const parentRect = parent?.getBoundingClientRect() ?? {
            left: 0,
            top: 0,
        };
        const left =
            rect.left -
            parentRect.left -
            (parent?.clientLeft ?? 0) +
            (parent?.scrollLeft ?? window.scrollX);
        const top =
            rect.top -
            parentRect.top -
            (parent?.clientTop ?? 0) +
            (parent?.scrollTop ?? window.scrollY);
        panel.style.left = `${left}px`;
        panel.style.top = `${top}px`;
        panel.style.right = "auto";
        return { left, top };
    }

    document.addEventListener("pointerdown", (event) => {
        if (event.button !== 0 || interaction) return;
        const panel = event.target.closest?.('[data-panel="true"]');
        if (!panel) {
            select(null);
            return;
        }
        if (panel.dataset.interactive !== "true") return;
        select(panel);
        // Selecting a control should never prevent its normal click/focus behavior.
        if (
            event.target.closest?.(
                'input, button, select, textarea, a, label, [contenteditable="true"], [role="button"]',
            )
        )
            return;

        const handle = event.target.closest?.("[data-resize-handle]");
        const content = contentOf(panel);
        if (!content) return;
        const rect = panel.getBoundingClientRect();
        const { left, top } = placeAbsolutely(panel, rect);
        const scale = number(panel.dataset.scale, 1);
        const contentRect = content.getBoundingClientRect();
        interaction = {
            mode: handle ? "scale" : "drag",
            panel,
            pointerId: event.pointerId,
            startX: event.clientX,
            startY: event.clientY,
            left,
            top,
            handle: handle?.dataset.resizeHandle,
            baseWidth: Math.max(contentRect.width / scale, 1),
            baseHeight: Math.max(contentRect.height / scale, 1),
            scale,
        };
        panel.setPointerCapture(event.pointerId);
        event.preventDefault();
        if (handle) event.stopPropagation();
    });

    document.addEventListener("pointermove", (event) => {
        if (!interaction || event.pointerId !== interaction.pointerId) return;
        if (
            !interaction.panel.isConnected ||
            interaction.panel.dataset.interactive !== "true"
        ) {
            finish(event);
            return;
        }
        const dx = event.clientX - interaction.startX;
        const dy = event.clientY - interaction.startY;
        if (interaction.mode === "drag") {
            interaction.panel.style.left = `${interaction.left + dx}px`;
            interaction.panel.style.top = `${interaction.top + dy}px`;
        } else {
            const result = resize_geometry(interaction, dx, dy);
            interaction.panel.style.left = `${result.left}px`;
            interaction.panel.style.top = `${result.top}px`;
            applyScale(interaction.panel, result.scale);
        }
    });

    function finish(event) {
        if (!interaction || event.pointerId !== interaction.pointerId) return;
        const { panel, pointerId } = interaction;
        interaction = null;
        save(panel);
        if (panel.hasPointerCapture(pointerId))
            panel.releasePointerCapture(pointerId);
    }
    document.addEventListener("pointerup", finish);
    document.addEventListener("pointercancel", finish);
    document.addEventListener("lostpointercapture", finish);

    scheduleRestore();
    const observer = new MutationObserver(scheduleRestore);
    if (document.body)
        observer.observe(document.body, { childList: true, subtree: true });
}

globalThis.mouse_events = mouse_events;

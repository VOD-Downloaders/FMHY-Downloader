// ── Config ──────────────────────────────────────────────────────────────────
const POLL_MS = 1500;   // status poll interval
const TICK_MS = 1000;   // elapsed counter tick

// ── State ────────────────────────────────────────────────────────────────────
// Map<id: number, {id, url, output, indexer, statusData, done, startTime, intervalId}>
const downloads = new Map();

// ── DOM refs ─────────────────────────────────────────────────────────────────
const sel        = document.getElementById('indexer-select');
const urlIn      = document.getElementById('url-input');
const nameIn     = document.getElementById('name-input');
const dlBtn      = document.getElementById('dl-btn');
const formStatus = document.getElementById('form-status');
const dlList     = document.getElementById('downloads-list');
const queueMeta  = document.getElementById('queue-meta');
const clearBtn   = document.getElementById('clear-btn');
const healthDot  = document.getElementById('health-dot');

// ── Background canvas (animated dot grid) ────────────────────────────────────
(function initCanvas() {
    const canvas = document.getElementById('bg-canvas');
    const ctx = canvas.getContext('2d');

    const SPACING = 36;
    const DOT_R   = 1;

    let W, H, cols, rows;
    let tick = 0;

    function resize() {
        W = canvas.width  = window.innerWidth;
        H = canvas.height = window.innerHeight;
        cols = Math.ceil(W / SPACING) + 1;
        rows = Math.ceil(H / SPACING) + 1;
    }

    function draw() {
        ctx.clearRect(0, 0, W, H);
        tick += 0.008;

        for (let r = 0; r < rows; r++) {
            for (let c = 0; c < cols; c++) {
                const x = c * SPACING;
                const y = r * SPACING;
                // Fade from top (strong) to bottom (invisible)
                const fy = 1 - Math.min(1, y / (H * 0.6));
                // Very subtle wave
                const wave = Math.sin(tick + c * 0.35 + r * 0.25) * 0.5 + 0.5;
                const alpha = fy * (0.08 + wave * 0.04);
                ctx.beginPath();
                ctx.arc(x, y, DOT_R, 0, Math.PI * 2);
                ctx.fillStyle = `rgba(0, 208, 156, ${alpha})`;
                ctx.fill();
            }
        }
        requestAnimationFrame(draw);
    }

    window.addEventListener('resize', resize);
    resize();
    draw();
})();

// ── Health check ─────────────────────────────────────────────────────────────
async function checkHealth() {
    try {
        const res = await fetch('/health');
        healthDot.className = 'health-dot ' + (res.ok ? 'ok' : 'err');
    } catch {
        healthDot.className = 'health-dot err';
    }
}
checkHealth();

// ── Boot: Load indexers ───────────────────────────────────────────────────────
(async function loadIndexers() {
    try {
        const res = await fetch('/api/indexers');
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        const { indexers = [] } = await res.json();

        sel.innerHTML = '';

        if (indexers.length === 0) {
            const o = document.createElement('option');
            o.value = '';
            o.textContent = 'No indexers configured';
            sel.appendChild(o);
            setStatus('No indexers are configured on the server.', 'error');
            return;
        }

        for (const ix of indexers) {
            const o = document.createElement('option');
            o.value       = ix.name;
            o.textContent = ix.name;
            sel.appendChild(o);
        }

        sel.disabled   = false;
        dlBtn.disabled = false;
    } catch (e) {
        sel.innerHTML = '<option value="">Failed to load</option>';
        setStatus('Could not reach server: ' + e.message, 'error');
        healthDot.className = 'health-dot err';
    }
})();

// ── Submit ────────────────────────────────────────────────────────────────────
dlBtn.addEventListener('click', async () => {
    const indexer = sel.value;
    const url     = urlIn.value.trim();
    const output  = nameIn.value.trim();

    if (!indexer) { setStatus('Select an indexer.',      'error'); return; }
    if (!url)     { setStatus('Enter a source URL.',     'error'); return; }
    if (!output)  { setStatus('Enter an output path.',   'error'); return; }

    dlBtn.disabled = true;
    setStatus('Requesting…', '');

    try {
        const res = await fetch('/api/download', {
            method:  'POST',
            headers: { 'Content-Type': 'application/json' },
            body:    JSON.stringify({ indexer_name: indexer, input_url: url, output_file: output }),
        });

        const body = await res.json().catch(() => ({}));

        if (!res.ok) {
            setStatus('Error: ' + (body.error ?? res.statusText), 'error');
            return;
        }

        const { id } = body;
        setStatus(`Queued — ID ${id}`, 'ok');

        urlIn.value  = '';
        nameIn.value = '';

        startTracking(id, url, output, indexer);

    } catch (e) {
        setStatus('Network error: ' + e.message, 'error');
    } finally {
        dlBtn.disabled = false;
    }
});

// ── Download tracking ─────────────────────────────────────────────────────────
function startTracking(id, url, output, indexer) {
    const entry = {
        id, url, output, indexer,
        statusData: null,
        done:       false,
        startTime:  Date.now(),
        intervalId: null,
    };
    downloads.set(id, entry);
    render();

    entry.intervalId = setInterval(() => poll(id), POLL_MS);
    poll(id); // immediate first poll
}

async function poll(id) {
    const entry = downloads.get(id);
    if (!entry || entry.done) return;

    try {
        const res = await fetch(`/api/downloadStatus/${id}`);
        if (!res.ok) {
            entry.statusData = { _fetchError: `HTTP ${res.status}` };
        } else {
            const body = await res.json();
            entry.statusData = body.status ?? null;
        }
    } catch (e) {
        entry.statusData = { _fetchError: e.message };
    }

    if (isTerminal(entry.statusData)) {
        entry.done = true;
        clearInterval(entry.intervalId);
    }

    updateItem(entry);
    refreshQueueMeta();
}

// A status is terminal when it signals completion or failure.
// Handles both unit-string variants ("Complete") and object variants ({"Failed": {...}}).
function isTerminal(s) {
    if (!s) return false;
    if (typeof s === 'string') return s === 'Complete' || s === 'Done' || s === 'Failed';
    if (typeof s === 'object') return 'Complete' in s || 'Done' in s || 'Failed' in s;
    return false;
}

// ── Clear done ────────────────────────────────────────────────────────────────
clearBtn.addEventListener('click', () => {
    for (const [id, entry] of downloads) {
        if (entry.done) {
            downloads.delete(id);
            dlList.querySelector(`[data-dl-id="${id}"]`)?.remove();
        }
    }
    if (dlList.children.length === 0) {
        dlList.innerHTML = '<p class="empty-state">Queue is empty.</p>';
    }
    refreshQueueMeta();
});

// ── Elapsed ticker (1s, surgical DOM update) ──────────────────────────────────
setInterval(() => {
    for (const [id, entry] of downloads) {
        const el = dlList.querySelector(`[data-dl-id="${id}"] .dl-elapsed`);
        if (!el) continue;
        const s = Math.round((Date.now() - entry.startTime) / 1000);
        el.textContent = entry.done ? `done in ${s}s` : `${s}s`;
    }
}, TICK_MS);

// ── Full render (initial insert only) ─────────────────────────────────────────
function render() {
    // Remove "empty" placeholder if present
    dlList.querySelector('.empty-state')?.remove();

    const entry = [...downloads.values()].at(-1); // most recently added
    if (!entry) return;

    const el = buildItem(entry);
    // Newest at top
    dlList.prepend(el);
    refreshQueueMeta();
}

// ── Surgical update for existing items ────────────────────────────────────────
function updateItem(entry) {
    let el = dlList.querySelector(`[data-dl-id="${entry.id}"]`);
    if (!el) {
        // Shouldn't happen, but fall back to building fresh
        el = buildItem(entry);
        dlList.prepend(el);
        return;
    }

    const { label, cls } = parseStatus(entry.statusData);

    // Badge
    const badge = el.querySelector('.dl-badge');
    if (badge) {
        badge.textContent = label;
        badge.className = `dl-badge ${cls}`;
    }

    // Done styling
    if (entry.done) el.classList.add('dl-done');

    // Pulse — remove when done
    if (entry.done) el.querySelector('.dl-pulse')?.remove();

    // Error message — add/update/remove as needed
    const footer = el.querySelector('.dl-item-footer');
    let errEl = el.querySelector('.dl-error-msg');

    if (hasError(entry.statusData)) {
        const msg = errorMsg(entry.statusData);
        if (!errEl) {
            errEl = document.createElement('div');
            errEl.className = 'dl-error-msg';
            footer.before(errEl);
        }
        errEl.textContent = msg;
    } else if (errEl) {
        errEl.remove();
    }
}

// ── Build a fresh item element (only called for new downloads) ────────────────
function buildItem(entry) {
    const el = document.createElement('div');
    el.className = 'dl-item' + (entry.done ? ' dl-done' : '');
    el.dataset.dlId = entry.id;

    const { label, cls } = parseStatus(entry.statusData);
    const elapsed = Math.round((Date.now() - entry.startTime) / 1000);

    el.innerHTML = `
        <div class="dl-item-header">
            <span class="dl-id">#${entry.id}</span>
            <span class="dl-badge ${cls}">${label}</span>
        </div>
        <div class="dl-meta">
            <span class="dl-indexer">${esc(entry.indexer)}</span>
            <span class="dl-sep">·</span>
            <span class="dl-output">${esc(entry.output)}</span>
        </div>
        <div class="dl-url">${esc(entry.url)}</div>
        ${hasError(entry.statusData) ? `<div class="dl-error-msg">${esc(errorMsg(entry.statusData))}</div>` : ''}
        <div class="dl-item-footer">
            ${!entry.done ? '<div class="dl-pulse"><span></span><span></span><span></span></div>' : ''}
            <span class="dl-elapsed">${entry.done ? 'done in ' + elapsed + 's' : elapsed + 's'}</span>
        </div>
    `;

    return el;
}

// ── Queue meta badge ──────────────────────────────────────────────────────────
function refreshQueueMeta() {
    const total  = downloads.size;
    const active = [...downloads.values()].filter(e => !e.done).length;
    const done   = total - active;

    if (total === 0) {
        queueMeta.hidden = true;
        clearBtn.hidden  = true;
        return;
    }

    queueMeta.hidden  = false;
    queueMeta.textContent = active > 0
        ? `${active} active`
        : 'all done';

    clearBtn.hidden = done === 0;
}

// ── Status parsing ────────────────────────────────────────────────────────────
// Rust serde serialises unit enum variants as plain strings: "Starting"
// Struct/tuple variants as objects: {"Failed": {"message": "..."}}
function parseStatus(s) {
    if (s === null || s === undefined) return { label: 'Starting',    cls: 'badge-starting' };

    if (typeof s === 'string') {
        switch (s) {
            case 'Starting':    return { label: 'Starting',    cls: 'badge-starting' };
            case 'Downloading': return { label: 'Downloading', cls: 'badge-downloading' };
            case 'Complete':
            case 'Done':        return { label: 'Complete',    cls: 'badge-done' };
            case 'Failed':      return { label: 'Failed',      cls: 'badge-error' };
            default:            return { label: s,             cls: 'badge-unknown' };
        }
    }

    if (typeof s === 'object') {
        if ('_fetchError' in s) return { label: 'Error',       cls: 'badge-error' };
        if ('Failed'      in s) return { label: 'Failed',      cls: 'badge-error' };
        if ('Complete'    in s ||
            'Done'        in s) return { label: 'Complete',    cls: 'badge-done' };
        if ('Downloading' in s) return { label: 'Downloading', cls: 'badge-downloading' };
        if ('Starting'    in s) return { label: 'Starting',    cls: 'badge-starting' };
        return { label: JSON.stringify(s), cls: 'badge-unknown' };
    }

    return { label: String(s), cls: 'badge-unknown' };
}

function hasError(s) {
    if (!s || typeof s !== 'object') return false;
    return '_fetchError' in s || 'Failed' in s;
}

// Extract a human-readable error string from Failed or fetch-error variants
function errorMsg(s) {
    if (!s || typeof s !== 'object') return '';

    if ('_fetchError' in s) return s._fetchError;

    const inner = s['Failed'];
    if (inner === undefined) return '';
    if (typeof inner === 'string') return inner;
    if (typeof inner === 'object' && inner !== null) {
        return inner.message ?? JSON.stringify(inner);
    }
    return String(inner);
}

// ── Helpers ───────────────────────────────────────────────────────────────────
function setStatus(msg, kind) {
    formStatus.textContent = msg;
    formStatus.className   = kind; // 'ok' | 'error' | ''
}

function esc(str) {
    return String(str)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;');
}

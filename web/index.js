// ── Config ───────────────────────────────────────────────────────────────────
const POLL_MS = 1500;
const TICK_MS = 1000;

// ── State ────────────────────────────────────────────────────────────────────
// Map<id: string, {id, url, output, indexer, statusData, done, startTime, intervalId}>
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
    const ctx    = canvas.getContext('2d');
    const SPACING = 36, DOT_R = 1;
    let W, H, cols, rows, tick = 0;

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
                const x  = c * SPACING;
                const y  = r * SPACING;
                const fy = 1 - Math.min(1, y / (H * 0.6));
                const wave  = Math.sin(tick + c * 0.35 + r * 0.25) * 0.5 + 0.5;
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
            sel.innerHTML = '<option value="">No indexers configured</option>';
            setStatus('No indexers are configured on the server.', 'error');
            return;
        }

        for (const ix of indexers) {
            const o = document.createElement('option');
            o.value = ix.name; o.textContent = ix.name;
            sel.appendChild(o);
        }
        sel.disabled = false; dlBtn.disabled = false;
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

    if (!indexer) { setStatus('Select an indexer.',    'error'); return; }
    if (!url)     { setStatus('Enter a source URL.',   'error'); return; }
    if (!output)  { setStatus('Enter an output path.', 'error'); return; }

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

        const { id } = body; // string — server serialises u64 as string to avoid JS precision loss
        setStatus(`Queued — ID ${id}`, 'ok');
        urlIn.value = ''; nameIn.value = '';
        startTracking(id, url, output, indexer);
    } catch (e) {
        setStatus('Network error: ' + e.message, 'error');
    } finally {
        dlBtn.disabled = false;
    }
});

// ── Download tracking ─────────────────────────────────────────────────────────
function startTracking(id, url, output, indexer) {
    const entry = { id, url, output, indexer, statusData: null, done: false, startTime: Date.now(), intervalId: null };
    downloads.set(id, entry);
    render(entry);

    entry.intervalId = setInterval(() => poll(id), POLL_MS);
    poll(id);
}

async function poll(id) {
    const entry = downloads.get(id);
    if (!entry || entry.done) return;

    try {
        const res = await fetch(`/api/downloadStatus/${id}`);
        entry.statusData = res.ok ? (await res.json()).status ?? null : { _fetchError: `HTTP ${res.status}` };
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

// ── Status logic ──────────────────────────────────────────────────────────────
// All 7 DownloadStatus variants + fetch error sentinel.
// Unit variants → plain strings e.g. "Starting"
// Struct variants → objects  e.g. {"Downloading": {"segment": 5, "total_segments": 100}}

function isTerminal(s) {
    if (!s) return false;
    if (typeof s === 'string') return s === 'Complete' || s === 'Failed';
    if (typeof s === 'object') return 'Complete' in s || 'Failed' in s || '_fetchError' in s;
    return false;
}

function parseStatus(s) {
    if (!s)                      return { label: 'Starting',          cls: 'badge-starting' };
    if (typeof s === 'string') {
        switch (s) {
            case 'Starting':         return { label: 'Starting',          cls: 'badge-starting' };
            case 'DownloadingIndex': return { label: 'Downloading Index', cls: 'badge-downloading' };
            case 'ParsingIndex':     return { label: 'Parsing Index',     cls: 'badge-downloading' };
            case 'Complete':         return { label: 'Complete',          cls: 'badge-done' };
            case 'Failed':           return { label: 'Failed',            cls: 'badge-error' };
            default:                 return { label: s,                   cls: 'badge-unknown' };
        }
    }
    if (typeof s === 'object') {
        if ('_fetchError'   in s) return { label: 'Error',            cls: 'badge-error' };
        if ('FindingIndex'  in s) return { label: 'Finding Index',    cls: 'badge-finding' };
        if ('Downloading'   in s) return { label: 'Downloading',      cls: 'badge-downloading' };
        if ('Failed'        in s) return { label: 'Failed',           cls: 'badge-error' };
        if ('Complete'      in s) return { label: 'Complete',         cls: 'badge-done' };
    }
    return { label: JSON.stringify(s), cls: 'badge-unknown' };
}

// Returns { frac: 0–1, mode: 'staged'|'determinate'|'complete'|'failed', label: string }
//
// The pipeline is staged into approximate progress fractions:
//   Starting        →  2%
//   FindingIndex    →  5%   (indeterminate retries)
//   DownloadingIndex→ 8%
//   ParsingIndex    →ː 12%
//   Downloading     → 12%–100%  (exact: 12 + segment/total * 88)
//   Complete        → 100%  green
//   Failed          → 100%  red
function getProgress(s) {
    if (!s)              return { frac: 0.02, mode: 'staged',      label: 'Starting…' };

    if (typeof s === 'string') {
        switch (s) {
            case 'Starting':         return { frac: 0.02, mode: 'staged',      label: 'Starting…' };
            case 'DownloadingIndex': return { frac: 0.08, mode: 'staged',      label: 'Downloading stream index' };
            case 'ParsingIndex':     return { frac: 0.12, mode: 'staged',      label: 'Parsing index' };
            case 'Complete':         return { frac: 1.00, mode: 'complete',    label: '' };
            case 'Failed':           return { frac: 1.00, mode: 'failed',      label: '' };
        }
    }

    if (typeof s === 'object') {
        if ('_fetchError' in s) {
            return { frac: 1.00, mode: 'failed', label: '' };
        }
        if ('FindingIndex' in s) {
            const attempt = s.FindingIndex?.attempt ?? 1;
            return { frac: 0.05, mode: 'staged', label: `Finding stream index (attempt ${attempt})` };
        }
        if ('Downloading' in s) {
            const seg   = s.Downloading?.segment ?? 0;
            const total = s.Downloading?.total_segments ?? 1;
            const raw   = total > 0 ? seg / total : 0;
            return {
                frac:  0.12 + raw * 0.88,
                mode:  'determinate',
                label: `${seg.toLocaleString()} / ${total.toLocaleString()} segments`,
            };
        }
        if ('Failed'   in s) return { frac: 1.00, mode: 'failed',   label: '' };
        if ('Complete' in s) return { frac: 1.00, mode: 'complete',  label: '' };
    }

    return { frac: 0.02, mode: 'staged', label: '' };
}

function hasError(s) {
    if (!s || typeof s !== 'object') return false;
    return '_fetchError' in s || 'Failed' in s;
}

function errorMsg(s) {
    if (!s || typeof s !== 'object') return '';
    if ('_fetchError' in s) return s._fetchError;
    const inner = s['Failed'];
    if (!inner) return '';
    if (typeof inner === 'string') return inner;
    if (typeof inner === 'object') return inner.message ?? JSON.stringify(inner);
    return String(inner);
}

// ── Clear done ────────────────────────────────────────────────────────────────
clearBtn.addEventListener('click', () => {
    for (const [id, entry] of downloads) {
        if (entry.done) {
            downloads.delete(id);
            dlList.querySelector(`[data-dl-id="${id}"]`)?.remove();
        }
    }
    if (dlList.children.length === 0) dlList.innerHTML = '<p class="empty-state">Queue is empty.</p>';
    refreshQueueMeta();
});

// ── Elapsed ticker ────────────────────────────────────────────────────────────
setInterval(() => {
    for (const [id, entry] of downloads) {
        const el = dlList.querySelector(`[data-dl-id="${id}"] .dl-elapsed`);
        if (!el) continue;
        const s = Math.round((Date.now() - entry.startTime) / 1000);
        el.textContent = entry.done ? `done in ${s}s` : `${s}s`;
    }
}, TICK_MS);

// ── Render: insert a new item (called once per download) ──────────────────────
function render(entry) {
    dlList.querySelector('.empty-state')?.remove();

    const el = buildItem(entry);
    dlList.prepend(el);

    // Animate the bar from 0% to its actual initial value now that it's in the DOM
    const prog = getProgress(entry.statusData);
    requestAnimationFrame(() => {
        const fill = el.querySelector('.dl-progress-fill');
        if (fill) fill.style.width = pct(prog.frac);
    });

    refreshQueueMeta();
}

// ── Build a fresh item element ────────────────────────────────────────────────
function buildItem(entry) {
    const el = document.createElement('div');
    el.className   = 'dl-item' + (entry.done ? ' dl-done' : '');
    el.dataset.dlId = entry.id;

    const { label, cls }         = parseStatus(entry.statusData);
    const { mode, label: pLabel } = getProgress(entry.statusData);
    const elapsed                 = Math.round((Date.now() - entry.startTime) / 1000);
    const errText                 = hasError(entry.statusData) ? errorMsg(entry.statusData) : '';

    // Progress fill starts at 0%; render() triggers the animated jump via rAF
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
        <div class="dl-progress ${mode}">
            <div class="dl-progress-fill" style="width: 0%"></div>
        </div>
        ${errText ? `<div class="dl-error-msg">${esc(errText)}</div>` : ''}
        <div class="dl-item-footer">
            <span class="dl-progress-label">${esc(pLabel)}</span>
            <span class="dl-elapsed">${entry.done ? 'done in ' + elapsed + 's' : elapsed + 's'}</span>
        </div>
    `;
    return el;
}

// ── Surgical update for existing items (called on every poll) ─────────────────
function updateItem(entry) {
    const el = dlList.querySelector(`[data-dl-id="${entry.id}"]`);
    if (!el) { render(entry); return; } // safety fallback

    const { label, cls }              = parseStatus(entry.statusData);
    const { frac, mode, label: pLabel } = getProgress(entry.statusData);

    // Badge
    const badge = el.querySelector('.dl-badge');
    if (badge) { badge.textContent = label; badge.className = `dl-badge ${cls}`; }

    // Progress bar: class drives color/shimmer, style.width drives the fill
    const progress = el.querySelector('.dl-progress');
    const fill     = el.querySelector('.dl-progress-fill');
    if (progress) progress.className = `dl-progress ${mode}`;
    if (fill)     fill.style.width   = pct(frac);

    // Progress label (segment count, attempt number, etc.)
    const pLabelEl = el.querySelector('.dl-progress-label');
    if (pLabelEl) pLabelEl.textContent = pLabel;

    // Done fade
    if (entry.done) el.classList.add('dl-done');

    // Error message: add, update, or remove
    const footer = el.querySelector('.dl-item-footer');
    let errEl = el.querySelector('.dl-error-msg');
    const errText = hasError(entry.statusData) ? errorMsg(entry.statusData) : '';

    if (errText) {
        if (!errEl) { errEl = document.createElement('div'); errEl.className = 'dl-error-msg'; footer.before(errEl); }
        errEl.textContent = errText;
    } else if (errEl) {
        errEl.remove();
    }
}

// ── Queue meta badge ──────────────────────────────────────────────────────────
function refreshQueueMeta() {
    const total  = downloads.size;
    const active = [...downloads.values()].filter(e => !e.done).length;
    const done   = total - active;

    if (total === 0) { queueMeta.hidden = true; clearBtn.hidden = true; return; }

    queueMeta.hidden      = false;
    queueMeta.textContent = active > 0 ? `${active} active` : 'all done';
    clearBtn.hidden       = done === 0;
}

// ── Helpers ───────────────────────────────────────────────────────────────────
function pct(frac) { return (Math.min(1, Math.max(0, frac)) * 100).toFixed(2) + '%'; }

function setStatus(msg, kind) { formStatus.textContent = msg; formStatus.className = kind; }

function esc(str) {
    return String(str)
        .replace(/&/g, '&amp;').replace(/</g, '&lt;')
        .replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

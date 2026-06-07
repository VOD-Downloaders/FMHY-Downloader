/* global bootstrap */

// Shared application logic, loaded on every page.
// Cross-page state persists via sessionStorage (each page is a full navigation).

const App = {
    // sessionStorage-backed state

    store: {
        get(key, fallback) {
            const raw = sessionStorage.getItem(key);

            if (raw === null) {
                return fallback;
            }

            try {
                return JSON.parse(raw);
            } catch {
                return fallback;
            }
        },
        set(key, value) {
            sessionStorage.setItem(key, JSON.stringify(value));
        },
    },

    get indexers() {
        return App.store.get("indexers", []);
    },
    set indexers(value) {
        App.store.set("indexers", value);
    },

    get currentStreams() {
        return App.store.get("currentStreams", []);
    },
    set currentStreams(value) {
        App.store.set("currentStreams", value);
    },

    get currentIndexer() {
        return App.store.get("currentIndexer", "");
    },
    set currentIndexer(value) {
        App.store.set("currentIndexer", value);
    },

    get currentUrl() {
        return App.store.get("currentUrl", "");
    },
    set currentUrl(value) {
        App.store.set("currentUrl", value);
    },

    get downloads() {
        return App.store.get("downloads", []);
    },
    set downloads(value) {
        App.store.set("downloads", value);
    },

    // API

    async fetchIndexers() {
        const res = await fetch("/api/indexers");

        if (!res.ok) {
            throw new Error(res.statusText);
        }

        const data = await res.json();

        return data.indexers || [];
    },

    async fetchIndexerSpecifications() {
        const res = await fetch("/api/indexers/specifications");

        if (!res.ok) {
            throw new Error(res.statusText);
        }

        const data = await res.json();

        return data.indexers || [];
    },

    async refreshIndexerSpecifications() {
        // POST /api/indexers/specifications/refresh fetches the latest specs from GitHub and
        // returns the freshly loaded list.
        const res = await fetch("/api/indexers/specifications/refresh", { method: "POST" });

        if (!res.ok) {
            const data = await res.json().catch(() => ({}));

            throw new Error(data.error || res.statusText);
        }

        const data = await res.json();

        return data.indexers || [];
    },

    async createIndexer(indexer) {
        // POST /api/indexers/create expects the indexer wrapped under an `indexer` key and
        // returns a status code with no body.
        const res = await fetch("/api/indexers/create", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ indexer: indexer }),
        });

        if (!res.ok) {
            const data = await res.json().catch(() => ({}));

            throw new Error(data.error || res.statusText);
        }
    },

    async deleteIndexer(name) {
        const res = await fetch("/api/indexers/delete", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ name: name }),
        });

        if (!res.ok) {
            const data = await res.json().catch(() => ({}));

            throw new Error(data.error || res.statusText);
        }
    },

    async fetchStreams(indexerName, url) {
        const res = await fetch("/api/streams", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ indexer_name: indexerName, input_url: url }),
        });

        if (!res.ok) {
            const data = await res.json().catch(() => ({}));

            throw new Error(data.error || res.statusText);
        }

        const data = await res.json();

        return data.streams || [];
    },

    async startDownload(indexerName, stream, outputFile) {
        const res = await fetch("/api/download", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
                indexer_name: indexerName,
                stream: stream,
                output_file: "/output/" + outputFile,
            }),
        });

        if (!res.ok) {
            const data = await res.json().catch(() => ({}));

            throw new Error(data.error || res.statusText);
        }

        const data = await res.json();

        return data.id;
    },

    // Loads indexers from the API and caches them. Returns the list.
    async loadIndexers() {
        try {
            const indexers = await App.fetchIndexers();

            App.indexers = indexers;

            return indexers;
        } catch {
            return App.indexers;
        }
    },

    // Helpers

    streamResolution(stream) {
        if (stream.width && stream.height) {
            return stream.width + "\xd7" + stream.height;
        }

        return "—";
    },

    streamTypeName(stream) {
        if (!stream.stream_type) {
            return "—";
        }

        return Object.keys(stream.stream_type)[0] || "—";
    },

    segmentCount(stream) {
        if (!stream.stream_type) {
            return 0;
        }

        const segments = Object.values(stream.stream_type)[0];

        return Array.isArray(segments) ? segments.length : 0;
    },

    escapeHtml(str) {
        return String(str)
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;");
    },

    // Per-page initialization

    pages: {
        async home() {
            await App.loadIndexers();

            document.getElementById("stat-indexers").textContent = App.indexers.length;
            document.getElementById("stat-downloads").textContent = App.downloads.length;
        },

        async search() {
            await App.loadIndexers();

            const select = document.getElementById("search-indexer");

            App.indexers.forEach((indexer) => {
                const opt = document.createElement("option");

                opt.value = indexer.name;
                opt.textContent = indexer.name + (indexer.uses_cloudflare ? " (CF)" : "");
                select.appendChild(opt);
            });

            document.getElementById("btn-get-streams").addEventListener("click", App.handleGetStreams);
        },

        async indexers() {
            await App.initIndexersPage();
        },

        streams() {
            App.renderStreams();
        },

        downloads() {
            App.renderDownloads();
        },

        proxy() {
            // TODO: FlareSolverr / proxy configuration UI
        },
    },

    // Search page

    async handleGetStreams() {
        const url = document.getElementById("search-url").value.trim();
        const indexerName = document.getElementById("search-indexer").value;
        const errorEl = document.getElementById("search-error");
        const spinnerEl = document.getElementById("search-spinner");
        const btn = document.getElementById("btn-get-streams");

        errorEl.classList.add("d-none");

        if (!url) {
            errorEl.textContent = "Enter a VOD URL.";
            errorEl.classList.remove("d-none");
            return;
        }

        if (!indexerName) {
            errorEl.textContent = "Select an indexer.";
            errorEl.classList.remove("d-none");
            return;
        }

        spinnerEl.classList.remove("d-none");
        btn.disabled = true;

        try {
            const streams = await App.fetchStreams(indexerName, url);

            App.currentStreams = streams;
            App.currentIndexer = indexerName;
            App.currentUrl = url;

            window.location.href = "streams.html";
        } catch (err) {
            errorEl.textContent = "Failed to retrieve streams: " + err.message;
            errorEl.classList.remove("d-none");
        } finally {
            spinnerEl.classList.add("d-none");
            btn.disabled = false;
        }
    },

    // Streams page

    renderStreams() {
        const streams = App.currentStreams;
        const sourceEl = document.getElementById("streams-source");
        const emptyEl = document.getElementById("streams-empty");
        const tableWrap = document.getElementById("streams-table-wrap");
        const tbody = document.getElementById("streams-tbody");

        sourceEl.textContent = App.currentUrl || "";

        if (!streams || streams.length === 0) {
            emptyEl.classList.remove("d-none");
            tableWrap.classList.add("d-none");
            return;
        }

        emptyEl.classList.add("d-none");
        tableWrap.classList.remove("d-none");
        tbody.innerHTML = "";

        streams.forEach((stream, i) => {
            const tr = document.createElement("tr");

            tr.innerHTML =
                "<td>" + (i + 1) + "</td>" +
                "<td>" + App.streamResolution(stream) + "</td>" +
                "<td>" + App.streamTypeName(stream) + "</td>" +
                "<td>" + App.segmentCount(stream) + "</td>" +
                "<td><button class=\"btn btn-sm btn-success\" data-stream-index=\"" + i + "\" type=\"button\">Download</button></td>";

            tbody.appendChild(tr);
        });

        tbody.querySelectorAll("[data-stream-index]").forEach((btn) => {
            btn.addEventListener("click", () => App.openDownloadModal(parseInt(btn.dataset.streamIndex, 10)));
        });

        document.getElementById("btn-confirm-download").addEventListener("click", App.handleConfirmDownload);
    },

    openDownloadModal(streamIndex) {
        const stream = App.currentStreams[streamIndex];

        if (!stream) {
            return;
        }

        App.pendingStreamIndex = streamIndex;

        document.getElementById("modal-stream-info").textContent =
            App.streamResolution(stream) + " — " + App.streamTypeName(stream) + ", " + App.segmentCount(stream) + " segments";

        document.getElementById("download-error").classList.add("d-none");
        document.getElementById("download-filename").value = "";

        if (!App.downloadModal) {
            App.downloadModal = new bootstrap.Modal(document.getElementById("modal-download"));
        }

        App.downloadModal.show();
    },

    async handleConfirmDownload() {
        const filename = document.getElementById("download-filename").value.trim();
        const errorEl = document.getElementById("download-error");
        const btn = document.getElementById("btn-confirm-download");

        errorEl.classList.add("d-none");

        if (!filename) {
            errorEl.textContent = "Enter an output filename.";
            errorEl.classList.remove("d-none");
            return;
        }

        const stream = App.currentStreams[App.pendingStreamIndex];

        btn.disabled = true;

        try {
            const id = await App.startDownload(App.currentIndexer, stream, filename);

            const downloads = App.downloads;

            downloads.push({
                id,
                outputFile: filename,
                indexerName: App.currentIndexer,
                status: "started",
            });
            App.downloads = downloads;

            window.location.href = "downloads.html";
        } catch (err) {
            errorEl.textContent = "Download failed: " + err.message;
            errorEl.classList.remove("d-none");
        } finally {
            btn.disabled = false;
        }
    },

    // Downloads page

    renderDownloads() {
        const downloads = App.downloads;
        const emptyEl = document.getElementById("downloads-empty");
        const table = document.getElementById("downloads-table");
        const tbody = document.getElementById("downloads-tbody");

        if (downloads.length === 0) {
            emptyEl.classList.remove("d-none");
            table.classList.add("d-none");
            return;
        }

        emptyEl.classList.add("d-none");
        table.classList.remove("d-none");
        tbody.innerHTML = "";

        downloads.forEach((dl) => {
            const tr = document.createElement("tr");

            tr.innerHTML =
                "<td><code>" + dl.id + "</code></td>" +
                "<td>/output/" + App.escapeHtml(dl.outputFile) + "</td>" +
                "<td>" + App.escapeHtml(dl.indexerName) + "</td>" +
                "<td><span class=\"badge bg-success\">" + App.escapeHtml(dl.status) + "</span></td>";

            tbody.appendChild(tr);
        });
        // TODO: poll GET /api/downloadStatus/{id} for live status updates
    },

    // Indexers page
    //
    // The form is shared between two flows:
    //   - "create": pick a specification, tweak the editable fields, create a new indexer.
    //   - "edit":   click an active indexer, load its values back into the form, save (overwrites).
    // Locked parts (method type, segment headers, byte removal, based_on) live in App.formLocked so
    // they survive editing without being shown in the UI.

    async initIndexersPage() {
        // Load specifications (to create from) and the already-active indexers (to list).
        try {
            App.specifications = await App.fetchIndexerSpecifications();
        } catch {
            App.specifications = [];
        }
        await App.loadIndexers();

        App.populateSpecSelect();

        document.getElementById("indexer-spec").addEventListener("change", (event) => {
            const index = event.target.value;

            if (index === "") {
                App.resetIndexerForm();
                return;
            }

            App.startCreateFromSpec(App.specifications[parseInt(index, 10)]);
        });

        document.getElementById("btn-save-indexer").addEventListener("click", App.handleSaveIndexer);
        document.getElementById("btn-cancel-edit").addEventListener("click", App.resetIndexerForm);
        document.getElementById("btn-refresh-specs").addEventListener("click", App.handleRefreshSpecs);

        App.renderActiveIndexers();
    },

    populateSpecSelect() {
        const specSelect = document.getElementById("indexer-spec");

        specSelect.innerHTML = "<option value=\"\">— select specification —</option>";
        App.specifications.forEach((spec, i) => {
            const opt = document.createElement("option");

            opt.value = String(i);
            opt.textContent = spec.name;
            specSelect.appendChild(opt);
        });
    },

    // Fill the URL dropdown with the given URLs and select `selected` (added if missing).
    setIndexerUrlOptions(urls, selected) {
        const urlSelect = document.getElementById("indexer-url");
        const all = [...urls];

        if (selected && !all.includes(selected)) {
            all.unshift(selected);
        }

        urlSelect.innerHTML = "";
        all.forEach((url) => {
            const opt = document.createElement("option");

            opt.value = url;
            opt.textContent = url;
            urlSelect.appendChild(opt);
        });

        if (selected) {
            urlSelect.value = selected;
        }
    },

    // Shared field population. `download` is a DownloadSpecification (from a spec or an indexer).
    fillIndexerFields(name, cloudflare, download) {
        document.getElementById("indexer-name").value = name;
        document.getElementById("indexer-cloudflare").checked = cloudflare;
        document.getElementById("indexer-method").value = download.method.type;
        document.getElementById("indexer-wait-time").value = download.method.wait_time;
        document.getElementById("indexer-retries").value = download.method.retries;
        document.getElementById("indexer-segment-timeout").value = download.segment_pre_download.segment_timeout;
        document.getElementById("indexer-segment-attempts").value = download.segment_pre_download.segment_attempts;

        document.getElementById("indexer-error").classList.add("d-none");
        document.getElementById("indexer-form").classList.remove("d-none");
    },

    startCreateFromSpec(spec) {
        App.formLocked = {
            methodType: spec.download.method.type,
            headers: spec.download.segment_pre_download.headers,
            postDownload: spec.download.segment_post_download,
            basedOn: spec.name,
        };

        App.setIndexerUrlOptions([spec.url, ...(spec.mirrors || [])], spec.url);
        App.fillIndexerFields(spec.name, spec.uses_cloudflare, spec.download);

        document.getElementById("indexer-form-title").textContent = "Create Indexer";
        document.getElementById("btn-save-indexer").textContent = "Create Indexer";
        document.getElementById("btn-cancel-edit").classList.add("d-none");
    },

    startEditIndexer(indexer) {
        App.formLocked = {
            methodType: indexer.download.method.type,
            headers: indexer.download.segment_pre_download.headers,
            postDownload: indexer.download.segment_post_download,
            basedOn: indexer.based_on,
        };

        // Offer the URLs of the spec it was based on (if still present), plus its current URL.
        const spec = App.specifications.find((item) => item.name === indexer.based_on);
        const urls = spec ? [spec.url, ...(spec.mirrors || [])] : [indexer.url];

        App.setIndexerUrlOptions(urls, indexer.url);
        App.fillIndexerFields(indexer.name, indexer.uses_cloudflare, indexer.download);

        // The spec dropdown is a create-only entry point; editing detaches from it.
        document.getElementById("indexer-spec").value = "";

        document.getElementById("indexer-form-title").textContent = "Edit Indexer — " + indexer.name;
        document.getElementById("btn-save-indexer").textContent = "Save Changes";
        document.getElementById("btn-cancel-edit").classList.remove("d-none");

        document.getElementById("indexer-form").scrollIntoView({ behavior: "smooth", block: "start" });
    },

    resetIndexerForm() {
        App.formLocked = null;
        document.getElementById("indexer-spec").value = "";
        document.getElementById("indexer-form").classList.add("d-none");
        document.getElementById("indexer-error").classList.add("d-none");
        document.getElementById("indexer-form-title").textContent = "Create Indexer";
        document.getElementById("btn-save-indexer").textContent = "Create Indexer";
        document.getElementById("btn-cancel-edit").classList.add("d-none");
    },

    async handleSaveIndexer() {
        const errorEl = document.getElementById("indexer-error");
        const btn = document.getElementById("btn-save-indexer");

        errorEl.classList.add("d-none");

        if (!App.formLocked) {
            return;
        }

        const name = document.getElementById("indexer-name").value.trim();
        const url = document.getElementById("indexer-url").value;

        if (name === "" || url === "") {
            errorEl.textContent = "Name and URL are required.";
            errorEl.classList.remove("d-none");
            return;
        }

        // Headers, byte removal, method type and based_on are locked (App.formLocked). Editable:
        // name, URL, Cloudflare, wait_time/retries and segment timeouts.
        const indexer = {
            name: name,
            url: url,
            uses_cloudflare: document.getElementById("indexer-cloudflare").checked,
            download: {
                method: {
                    type: App.formLocked.methodType,
                    wait_time: Number(document.getElementById("indexer-wait-time").value),
                    retries: Number(document.getElementById("indexer-retries").value),
                },
                segment_pre_download: {
                    segment_timeout: Number(document.getElementById("indexer-segment-timeout").value),
                    segment_attempts: Number(document.getElementById("indexer-segment-attempts").value),
                    headers: App.formLocked.headers,
                },
                segment_post_download: App.formLocked.postDownload,
            },
            based_on: App.formLocked.basedOn,
        };

        btn.disabled = true;

        try {
            await App.createIndexer(indexer);
            window.location.reload();
        } catch (err) {
            errorEl.textContent = "Failed to save indexer: " + err.message;
            errorEl.classList.remove("d-none");
        } finally {
            btn.disabled = false;
        }
    },

    async handleRefreshSpecs() {
        const btn = document.getElementById("btn-refresh-specs");
        const spinner = document.getElementById("refresh-specs-spinner");
        const status = document.getElementById("refresh-specs-status");

        status.classList.add("d-none", "text-danger", "text-success");
        spinner.classList.remove("d-none");
        btn.disabled = true;

        try {
            App.specifications = await App.refreshIndexerSpecifications();
            App.populateSpecSelect();

            status.textContent = "Retrieved " + App.specifications.length + " specification(s).";
            status.classList.add("text-success");
            status.classList.remove("d-none");
        } catch (err) {
            status.textContent = "Failed to retrieve specifications: " + err.message;
            status.classList.add("text-danger");
            status.classList.remove("d-none");
        } finally {
            spinner.classList.add("d-none");
            btn.disabled = false;
        }
    },

    renderActiveIndexers() {
        const indexers = App.indexers;
        const emptyEl = document.getElementById("indexers-empty");
        const table = document.getElementById("indexers-table");
        const tbody = document.getElementById("indexers-tbody");

        if (indexers.length === 0) {
            emptyEl.classList.remove("d-none");
            table.classList.add("d-none");
            return;
        }

        emptyEl.classList.add("d-none");
        table.classList.remove("d-none");
        tbody.innerHTML = "";

        indexers.forEach((indexer, i) => {
            const method = indexer.download && indexer.download.method ? indexer.download.method.type : "—";
            const tr = document.createElement("tr");

            tr.innerHTML =
                "<td>" + App.escapeHtml(indexer.name) + "</td>" +
                "<td>" + App.escapeHtml(indexer.url) + "</td>" +
                "<td>" + (indexer.uses_cloudflare ? "yes" : "no") + "</td>" +
                "<td>" + App.escapeHtml(method) + "</td>" +
                "<td class=\"text-end\">" +
                    "<button class=\"btn btn-sm btn-outline-primary me-2\" data-edit-index=\"" + i + "\" type=\"button\">Edit</button>" +
                    "<button class=\"btn btn-sm btn-outline-danger\" data-delete-index=\"" + i + "\" type=\"button\">Delete</button>" +
                "</td>";

            tbody.appendChild(tr);
        });

        tbody.querySelectorAll("[data-edit-index]").forEach((btn) => {
            btn.addEventListener("click", () => App.startEditIndexer(App.indexers[parseInt(btn.dataset.editIndex, 10)]));
        });

        tbody.querySelectorAll("[data-delete-index]").forEach((btn) => {
            btn.addEventListener("click", () => App.handleDeleteIndexer(App.indexers[parseInt(btn.dataset.deleteIndex, 10)]));
        });
    },

    async handleDeleteIndexer(indexer) {
        if (!window.confirm("Delete indexer \"" + indexer.name + "\"?")) {
            return;
        }

        try {
            await App.deleteIndexer(indexer.name);
            window.location.reload();
        } catch (err) {
            window.alert("Failed to delete indexer: " + err.message);
        }
    },
};

// Dispatch to the current page's init based on body[data-page].
document.addEventListener("DOMContentLoaded", () => {
    const page = document.body.dataset.page;
    const init = App.pages[page];

    if (init) {
        init();
    }
});

/* global bootstrap */

const App = {
    state: {
        indexers: [],
        currentStreams: [],
        currentIndexer: "",
        currentUrl: "",
        downloads: [],
        pendingStream: null,
        downloadModal: null,
    },

    async init() {
        App.navigate("home");

        document.getElementById("btn-get-streams").addEventListener("click", App.handleGetStreams);
        document.getElementById("btn-confirm-download").addEventListener("click", App.handleConfirmDownload);

        // Streams empty-state link
        const streamLink = document.querySelector("#streams-empty [data-page]");

        if (streamLink) {
            streamLink.addEventListener("click", (e) => {
                e.preventDefault();
                App.navigate(streamLink.dataset.page);
            });
        }

        await App.loadIndexers();
    },

    navigate(page) {
        document.querySelectorAll(".page").forEach((el) => el.classList.add("d-none"));

        const target = document.getElementById("page-" + page);

        if (target) {
            target.classList.remove("d-none");
        }

        const sidebar = document.querySelector("component-sidebar");

        if (sidebar) {
            sidebar.setActivePage(page);
        }

        if (page === "home") App.renderHome();
        if (page === "streams") App.renderStreams();
        if (page === "downloads") App.renderDownloads();
    },

    // API

    async loadIndexers() {
        try {
            const res = await fetch("/api/indexers");

            if (!res.ok) {
                throw new Error(res.statusText);
            }

            const data = await res.json();

            App.state.indexers = data.indexers || [];
        } catch {
            App.state.indexers = [];
        }

        App.populateIndexerSelect();
        App.renderHome();
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

    // Handlers

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

            App.state.currentStreams = streams;
            App.state.currentIndexer = indexerName;
            App.state.currentUrl = url;

            App.navigate("streams");
        } catch (err) {
            errorEl.textContent = "Failed to retrieve streams: " + err.message;
            errorEl.classList.remove("d-none");
        } finally {
            spinnerEl.classList.add("d-none");
            btn.disabled = false;
        }
    },

    handleDownloadClick(streamIndex) {
        const stream = App.state.currentStreams[streamIndex];

        if (!stream) {
            return;
        }

        App.state.pendingStream = { streamIndex, stream };

        document.getElementById("modal-stream-info").textContent =
            App.streamResolution(stream) + " — " + App.streamTypeName(stream) + ", " + App.segmentCount(stream) + " segments";

        document.getElementById("download-error").classList.add("d-none");
        document.getElementById("download-filename").value = "";

        if (!App.state.downloadModal) {
            App.state.downloadModal = new bootstrap.Modal(document.getElementById("modal-download"));
        }

        App.state.downloadModal.show();
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

        const { stream } = App.state.pendingStream;

        btn.disabled = true;

        try {
            const id = await App.startDownload(App.state.currentIndexer, stream, filename);

            App.state.downloads.push({
                id,
                outputFile: filename,
                indexerName: App.state.currentIndexer,
                status: "started",
            });

            App.state.downloadModal.hide();
            App.navigate("downloads");
        } catch (err) {
            errorEl.textContent = "Download failed: " + err.message;
            errorEl.classList.remove("d-none");
        } finally {
            btn.disabled = false;
        }
    },

    // Render

    populateIndexerSelect() {
        const select = document.getElementById("search-indexer");

        while (select.options.length > 1) {
            select.remove(1);
        }

        App.state.indexers.forEach((indexer) => {
            const opt = document.createElement("option");

            opt.value = indexer.name;
            opt.textContent = indexer.name + (indexer.uses_cloudflare ? " (CF)" : "");
            select.appendChild(opt);
        });
    },

    renderHome() {
        const indexerEl = document.getElementById("stat-indexers");
        const downloadEl = document.getElementById("stat-downloads");

        if (indexerEl) {
            indexerEl.textContent = App.state.indexers.length;
        }

        if (downloadEl) {
            downloadEl.textContent = App.state.downloads.length;
        }
    },

    renderStreams() {
        const streams = App.state.currentStreams;
        const sourceEl = document.getElementById("streams-source");
        const emptyEl = document.getElementById("streams-empty");
        const tableWrap = document.getElementById("streams-table-wrap");
        const tbody = document.getElementById("streams-tbody");

        sourceEl.textContent = App.state.currentUrl || "";

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
            btn.addEventListener("click", () => App.handleDownloadClick(parseInt(btn.dataset.streamIndex, 10)));
        });
    },

    renderDownloads() {
        const downloads = App.state.downloads;
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
};

App.init();

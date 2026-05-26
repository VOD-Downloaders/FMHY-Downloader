document.getElementById("dl-btn").addEventListener("click", async () => {
    const url    = document.getElementById("url-input").value.trim();
    const name   = document.getElementById("name-input").value.trim();
    const btn    = document.getElementById("dl-btn");

    if (!url)  { setStatus("Please enter a URL.",       "error"); return; }
    if (!name) { setStatus("Please enter a file name.", "error"); return; }

    btn.disabled = true;
    setStatus("Requesting download…", "");

    try {
        const res = await fetch("/api/download", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ input_url: url, output_file: name }),
        });

		setStatus(res.statusText);

        if (!res.ok) {
            // Server returned an error body like { "error": "..." }
            const err = await res.json().catch(() => ({ error: res.statusText }));
            setStatus("Error: " + err.error, "error");
            return;
        }
    } catch (e) {
        setStatus("Network error: " + e.message, "error");
    } finally {
        btn.disabled = false;
    }
});

function setStatus(msg, kind) {
    const el = document.getElementById("status");
    el.textContent  = msg;
    el.className    = kind;   // "ok" | "error" | ""
}

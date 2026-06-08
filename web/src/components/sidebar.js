// Reusable sidebar. Reused on every page via <component-sidebar></component-sidebar>.
// Active link determined from window.location.pathname, so no per-page wiring needed.

class Sidebar extends HTMLElement {
    static PAGES = [
        { href: "index.html", label: "Home" },
        { href: "indexers.html", label: "Indexers" },
        { href: "search.html", label: "Search" },
        { href: "streams.html", label: "Streams" },
        { href: "downloads.html", label: "Downloads" },
        { href: "proxy.html", label: "Proxy" },
    ];

    connectedCallback() {
        const current = this.currentPage();

        const items = Sidebar.PAGES.map((page) => {
            const active = page.href === current ? " active" : "";

            return `
                <li class="nav-item">
                    <a class="nav-link${active}" href="${page.href}">${page.label}</a>
                </li>`;
        }).join("");

        this.innerHTML = `
            <nav id="sidebar" class="d-flex flex-column h-100 py-3 px-2">
                <a class="sidebar-brand mb-3 px-1" href="index.html">FMHY Downloader</a>
                <ul class="nav nav-pills flex-column gap-1">
                    ${items}
                </ul>
            </nav>
        `;
    }

    currentPage() {
        const path = window.location.pathname;
        const file = path.substring(path.lastIndexOf("/") + 1);

        return file === "" ? "index.html" : file;
    }
}

customElements.define("component-sidebar", Sidebar);

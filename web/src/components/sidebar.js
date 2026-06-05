class Sidebar extends HTMLElement {
  connectedCallback() {
    this.innerHTML = `
    	<h1>AAAAA BBBBB CCCC</h1>
    `;
  }
}

customElements.define('component-sidebar', Sidebar);

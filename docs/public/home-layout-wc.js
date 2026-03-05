class WcLayoutGrid extends HTMLElement {
  static get observedAttributes() {
    return ["columns", "rows", "gap"];
  }

  connectedCallback() {
    this.syncLayoutVars();
  }

  attributeChangedCallback() {
    this.syncLayoutVars();
  }

  syncLayoutVars() {
    const columns = this.getAttribute("columns");
    const rows = this.getAttribute("rows");
    const gap = this.getAttribute("gap");

    if (columns) this.style.setProperty("--wc-columns", columns);
    if (rows) this.style.setProperty("--wc-rows", rows);
    if (gap) this.style.setProperty("--wc-gap", gap);
  }
}

class WcLayoutPanel extends HTMLElement {
  static get observedAttributes() {
    return ["radius", "elevation"];
  }

  connectedCallback() {
    this.syncPaintVars();
  }

  attributeChangedCallback() {
    this.syncPaintVars();
  }

  syncPaintVars() {
    const radius = this.getAttribute("radius");
    const elevation = this.getAttribute("elevation");

    if (radius) this.style.setProperty("--wc-panel-radius", radius);
    if (elevation) this.style.setProperty("--wc-panel-elevation", elevation);
  }
}

if (!customElements.get("wc-layout-grid")) {
  customElements.define("wc-layout-grid", WcLayoutGrid);
}

if (!customElements.get("wc-layout-panel")) {
  customElements.define("wc-layout-panel", WcLayoutPanel);
}

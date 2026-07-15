/**
 * Shared Wyvern HTTP dialog helpers (packaged UI → wyvern-host).
 */
(function (global) {
  "use strict";

  async function fetchDialog() {
    const res = await fetch("/api/dialog", { headers: { Accept: "application/json" } });
    if (!res.ok) {
      throw new Error("GET /api/dialog failed: " + res.status);
    }
    return res.json();
  }

  async function postResult(body) {
    const json = JSON.stringify(body);
    // sendBeacon is more reliable in embedded WKWebView than fetch POST (macOS wyvern-viewer).
    if (typeof navigator.sendBeacon === "function") {
      const blob = new Blob([json], { type: "application/json" });
      if (navigator.sendBeacon("/api/result", blob)) {
        return { ok: true };
      }
    }
    const res = await fetch("/api/result", {
      method: "POST",
      headers: { "Content-Type": "application/json", Accept: "application/json" },
      body: json,
    });
    if (!res.ok) {
      const text = await res.text();
      throw new Error("POST /api/result failed: " + res.status + " " + text);
    }
    return res.json();
  }

  function postResultBeacon(body) {
    try {
      const blob = new Blob([JSON.stringify(body)], { type: "application/json" });
      return navigator.sendBeacon("/api/result", blob);
    } catch (_) {
      return false;
    }
  }

  async function postPickerFile(body) {
    const res = await fetch("/api/picker/file", {
      method: "POST",
      headers: { "Content-Type": "application/json", Accept: "application/json" },
      body: JSON.stringify(body || {}),
    });
    if (!res.ok) {
      const text = await res.text();
      throw new Error("POST /api/picker/file failed: " + res.status + " " + text);
    }
    return res.json();
  }

  async function postPickerFolder(body) {
    const res = await fetch("/api/picker/folder", {
      method: "POST",
      headers: { "Content-Type": "application/json", Accept: "application/json" },
      body: JSON.stringify(body || {}),
    });
    if (!res.ok) {
      const text = await res.text();
      throw new Error("POST /api/picker/folder failed: " + res.status + " " + text);
    }
    return res.json();
  }

  var VIEWER_MAX_W = 800;
  var VIEWER_MAX_H = 600;
  var VIEWER_MIN_W = 200;
  var VIEWER_MIN_H = 96;
  /** Comfortable compact-dialog content cap (REQ-V008 ~480px). */
  var COMFORT_MAX_W = 480;
  /** Panel dialogs (markdown, question) may be wider. */
  var PANEL_MAX_W = 560;
  /** Chrome foundation frame default open size (chrome/style.css). */
  var FRAME_MIN_W = 480;
  var FRAME_MIN_H = 360;
  /** Target window aspect width:height for compact dialogs. */
  var COMFORT_ASPECT = 4 / 3;
  /** Padding guard against subpixel scrollbars after resize. */
  var MEASURE_BUFFER = 8;

  function clampViewerSize(w, h, minW) {
    minW = minW || VIEWER_MIN_W;
    return {
      w: Math.min(Math.max(w, minW), VIEWER_MAX_W),
      h: Math.min(Math.max(h, VIEWER_MIN_H), VIEWER_MAX_H),
    };
  }

  /** Tight content fit; 4:3 cozy box only for very small dialogs. */
  function fitComfortableAspect(contentW, contentH, minW) {
    var w = contentW + MEASURE_BUFFER;
    var h = contentH + MEASURE_BUFFER;
    if (w < 280 && h < 180) {
      if (w / h > COMFORT_ASPECT) {
        h = Math.ceil(w / COMFORT_ASPECT);
      } else {
        w = Math.ceil(h * COMFORT_ASPECT);
      }
    }
    var sized = clampViewerSize(w, h, minW);
    sized.w = Math.max(sized.w, contentW + MEASURE_BUFFER);
    sized.h = Math.max(sized.h, contentH + MEASURE_BUFFER);
    return clampViewerSize(sized.w, sized.h, minW);
  }

  /** Temporarily unconstrain layout so measure is not limited by bootstrap viewport. */
  function measureAtComfortWidth(root, maxW) {
    maxW = maxW || COMFORT_MAX_W;
    var html = document.documentElement;
    var body = document.body;
    var saved = {
      htmlOverflow: html.style.overflow,
      bodyWidth: body.style.width,
      bodyMaxWidth: body.style.maxWidth,
      rootWidth: root.style.width,
      rootMaxWidth: root.style.maxWidth,
    };

    html.style.overflow = "visible";
    body.style.width = "auto";
    body.style.maxWidth = "none";
    root.style.width = "max-content";
    root.style.maxWidth = maxW + "px";

    void root.offsetHeight;
    var contentW = Math.ceil(
      Math.max(root.scrollWidth, root.getBoundingClientRect().width)
    );
    var contentH = Math.ceil(
      Math.max(root.scrollHeight, root.getBoundingClientRect().height)
    );

    html.style.overflow = saved.htmlOverflow;
    body.style.width = saved.bodyWidth;
    body.style.maxWidth = saved.bodyMaxWidth;
    root.style.width = saved.rootWidth;
    root.style.maxWidth = saved.rootMaxWidth;

    return { contentW: contentW, contentH: contentH };
  }

  /** Tight fit without aspect expansion (panels / tall content). */
  function fitTight(contentW, contentH, minW) {
    return clampViewerSize(
      contentW + MEASURE_BUFFER,
      contentH + MEASURE_BUFFER,
      minW
    );
  }

  function saveStyle(el, keys) {
    var entry = { el: el };
    keys.forEach(function (key) {
      entry[key] = el.style[key];
    });
    return entry;
  }

  function restoreStyles(entries) {
    entries.forEach(function (entry) {
      var el = entry.el;
      Object.keys(entry).forEach(function (key) {
        if (key !== "el") {
          el.style[key] = entry[key];
        }
      });
    });
  }

  /** Unlock flex/scroll layout so intrinsic content height is measurable. */
  function measureIntrinsicPanel(root, maxW) {
    var html = document.documentElement;
    var body = document.body;
    var saved = [];
    saved.push(saveStyle(html, ["overflow"]));
    saved.push(saveStyle(body, ["width", "maxWidth", "height", "minHeight"]));
    saved.push(
      saveStyle(root, ["height", "minHeight", "maxHeight", "width", "maxWidth", "flex"])
    );

    html.style.overflow = "visible";
    body.style.width = "auto";
    body.style.maxWidth = "none";
    body.style.height = "auto";
    body.style.minHeight = "0";
    root.style.flex = "none";
    root.style.height = "auto";
    root.style.minHeight = "0";
    root.style.maxHeight = "none";
    root.style.width = "max-content";
    root.style.maxWidth = maxW + "px";

    var scrollers = root.querySelectorAll(".content, .cards");
    for (var i = 0; i < scrollers.length; i++) {
      var el = scrollers[i];
      saved.push(
        saveStyle(el, ["overflow", "overflowY", "flex", "height", "minHeight", "maxHeight"])
      );
      el.style.overflow = "visible";
      el.style.overflowY = "visible";
      el.style.flex = "none";
      el.style.height = "auto";
      el.style.minHeight = "0";
      el.style.maxHeight = "none";
    }

    void root.offsetHeight;
    var contentW = Math.ceil(
      Math.max(root.scrollWidth, root.getBoundingClientRect().width)
    );
    var contentH = Math.ceil(
      Math.max(root.scrollHeight, root.getBoundingClientRect().height)
    );
    restoreStyles(saved);
    return { contentW: contentW, contentH: contentH };
  }

  /** Path picker dialogs — wide initial window; CSS fills on manual resize. */
  function measureFillDialog(root) {
    var macos = document.documentElement.classList.contains("wyvern-macos");
    var minW = macos ? 320 : VIEWER_MIN_W;
    var measured = measureAtComfortWidth(root, VIEWER_MAX_W);
    var w = Math.max(measured.contentW + MEASURE_BUFFER, 420);
    var h = measured.contentH + MEASURE_BUFFER;
    return clampViewerSize(w, h, minW);
  }

  /** Measure a compact dialog (#dialog.dialog--compact) for embedded window auto-size. */
  function measureDialogBox(root) {
    var macos = document.documentElement.classList.contains("wyvern-macos");
    var minW = macos ? 220 : VIEWER_MIN_W;
    var measured = measureAtComfortWidth(root, COMFORT_MAX_W);
    return fitComfortableAspect(measured.contentW, measured.contentH, minW);
  }

  /** Markdown / question — full intrinsic height, no internal scroll in auto-size. */
  function measurePanelDialog(root) {
    var macos = document.documentElement.classList.contains("wyvern-macos");
    var minW = macos ? 220 : VIEWER_MIN_W;
    var measured = measureIntrinsicPanel(root, PANEL_MAX_W);
    return fitTight(measured.contentW, measured.contentH, minW);
  }

  /** Chrome foundation frame — default 480×360 floor, intrinsic if larger. */
  function measureFrameDialog(root) {
    var measured = measureIntrinsicPanel(root, VIEWER_MAX_W);
    var w = Math.max(measured.contentW + MEASURE_BUFFER, FRAME_MIN_W);
    var h = Math.max(measured.contentH + MEASURE_BUFFER, FRAME_MIN_H);
    return clampViewerSize(w, h, FRAME_MIN_W);
  }

  /** Measure full page or visible dialog root when no explicit size was given. */
  function measurePage() {
    var dialog = document.getElementById("dialog");
    if (dialog && !dialog.hidden) {
      if (dialog.classList.contains("dialog--fill")) {
        return measureFillDialog(dialog);
      }
      if (dialog.classList.contains("dialog--compact")) {
        return measureDialogBox(dialog);
      }
      if (dialog.classList.contains("dialog--panel")) {
        return measurePanelDialog(dialog);
      }
      if (dialog.classList.contains("dialog--frame")) {
        return measureFrameDialog(dialog);
      }
    }
    var root = document.scrollingElement || document.documentElement;
    var body = document.body;
    var w = Math.ceil(
      Math.max(body ? body.scrollWidth : 0, body ? body.offsetWidth : 0, root.clientWidth)
    );
    var h = Math.ceil(
      Math.max(body ? body.scrollHeight : 0, body ? body.offsetHeight : 0, root.clientHeight)
    );
    if (dialog && !dialog.hidden) {
      var cr = dialog.getBoundingClientRect();
      w = Math.max(w, Math.ceil(cr.width));
      h = Math.max(h, Math.ceil(cr.height));
    }
    return {
      w: Math.min(Math.max(w, VIEWER_MIN_W), VIEWER_MAX_W),
      h: Math.min(Math.max(h, VIEWER_MIN_H), VIEWER_MAX_H),
    };
  }

  /** Embedded-only: resize native window to measured content (default when size omitted). */
  function notifyResize() {
    if (typeof window.ipc === "undefined" || typeof window.ipc.postMessage !== "function") {
      return;
    }
    var size = measurePage();
    window.ipc.postMessage("resize:" + size.w + "x" + size.h);
  }

  function readMetaViewerSize() {
    var wMeta = document.querySelector('meta[name="wyvern:width"]');
    var hMeta = document.querySelector('meta[name="wyvern:height"]');
    var size = {};
    if (wMeta && wMeta.getAttribute("content")) {
      var w = parseInt(wMeta.getAttribute("content"), 10);
      if (!isNaN(w) && w > 0) size.width = w;
    }
    if (hMeta && hMeta.getAttribute("content")) {
      var h = parseInt(hMeta.getAttribute("content"), 10);
      if (!isNaN(h) && h > 0) size.height = h;
    }
    return size;
  }

  function viewerSizeFromPayload(payload) {
    var size = {};
    if (payload && payload.width) size.width = payload.width;
    if (payload && payload.height) size.height = payload.height;
    return size;
  }

  /** JSON from `/api/dialog` wins; else optional `<meta name="wyvern:width|height">`. */
  function resolveViewerSize(payload) {
    var fromJson = viewerSizeFromPayload(payload);
    if (fromJson.width || fromJson.height) return fromJson;
    return readMetaViewerSize();
  }

  function applyFixedViewerSize(size) {
    if (typeof window.ipc === "undefined" || typeof window.ipc.postMessage !== "function") {
      return false;
    }
    if (!size || !size.width || !size.height) {
      return false;
    }
    window.ipc.postMessage("resize:" + size.width + "x" + size.height);
    return true;
  }

  /** After dialog content is visible: explicit size, else auto-size to content. */
  function applyDialogLayout(payload) {
    var fixed = resolveViewerSize(payload);
    if (applyFixedViewerSize(fixed)) {
      return;
    }
    scheduleResize({ mode: "auto" });
  }

  function scheduleResize(options) {
    options = options || {};
    if (options.fixedSize) {
      applyFixedViewerSize(options.fixedSize);
      return;
    }
    if (options.mode !== "auto" && options.mode !== "dialog") {
      return;
    }
    requestAnimationFrame(function () {
      requestAnimationFrame(function () {
        notifyResize();
        // Refine once after first paint (fonts/wrap settle in narrow bootstrap window).
        setTimeout(function () {
          notifyResize();
        }, 0);
      });
    });
  }

  /** Tag embedded viewer shell; macOS gets traffic-light safe-zone CSS vars. */
  function applyEmbeddedChrome() {
    if (typeof window.ipc === "undefined" || typeof window.ipc.postMessage !== "function") {
      return;
    }
    var root = document.documentElement;
    root.classList.add("wyvern-embedded");
    var ua = (navigator.userAgent || "").toLowerCase();
    if (ua.indexOf("mac os x") !== -1 || ua.indexOf("macintosh") !== -1) {
      root.classList.add("wyvern-macos");
    }
  }

  applyEmbeddedChrome();

  /** Fetch wizard state and populate `window.wyvern` (REQ-0024). */
  async function wyvernWizardState() {
    const res = await fetch("/api/wizard/state", {
      headers: { Accept: "application/json" },
    });
    if (!res.ok) {
      const text = await res.text();
      throw new Error("GET /api/wizard/state failed: " + res.status + " " + text);
    }
    const state = await res.json();
    global.wyvern = {
      config: state.config,
      page: state.page,
      page_data: state.page_data,
      stack: state.stack,
    };
    return state;
  }

  function collectCurrentPageDataFallback() {
    if (typeof global.collectCurrentPageData === "function") {
      return global.collectCurrentPageData();
    }
    return {};
  }

  /** Advance to `next`; on success perform a full page reload to `url`. */
  async function wyvernWizardNext(data, next) {
    const res = await fetch("/api/wizard/navigate", {
      method: "POST",
      headers: { "Content-Type": "application/json", Accept: "application/json" },
      body: JSON.stringify({ action: "next", data: data, next: next }),
    });
    if (!res.ok) {
      const text = await res.text();
      throw new Error("POST /api/wizard/navigate (next) failed: " + res.status + " " + text);
    }
    const body = await res.json();
    if (body && body.ok && body.url) {
      global.location = body.url;
    }
    return body;
  }

  /**
   * Move back. Omit `data` (or pass `{}`) to preserve the current entry via the
   * meaningful-payload predicate. When omitted, uses `collectCurrentPageData()`
   * if the page author defined it, otherwise `{}`.
   */
  async function wyvernWizardBack(data) {
    var payload = arguments.length === 0 ? collectCurrentPageDataFallback() : data;
    const res = await fetch("/api/wizard/navigate", {
      method: "POST",
      headers: { "Content-Type": "application/json", Accept: "application/json" },
      body: JSON.stringify({ action: "back", data: payload }),
    });
    if (!res.ok) {
      const text = await res.text();
      throw new Error("POST /api/wizard/navigate (back) failed: " + res.status + " " + text);
    }
    const body = await res.json();
    if (body && body.ok && body.url) {
      global.location = body.url;
    }
    return body;
  }

  /**
   * Terminal finish. `opts` = `{ button, data, stack }` where `stack` is the full
   * visited stack (`window.wyvern.stack` + current `{ page, data }`).
   */
  async function wyvernWizardFinish(opts) {
    opts = opts || {};
    const json = JSON.stringify({
      button: opts.button,
      data: opts.data,
      stack: opts.stack,
    });
    if (typeof navigator.sendBeacon === "function") {
      const blob = new Blob([json], { type: "application/json" });
      if (navigator.sendBeacon("/api/wizard/finish", blob)) {
        return { ok: true };
      }
    }
    const res = await fetch("/api/wizard/finish", {
      method: "POST",
      headers: { "Content-Type": "application/json", Accept: "application/json" },
      body: json,
    });
    if (!res.ok) {
      const text = await res.text();
      throw new Error("POST /api/wizard/finish failed: " + res.status + " " + text);
    }
    return res.json();
  }

  /** Production bootstrap: wizard pages load state into `window.wyvern`. */
  function bootstrapWizardIfNeeded() {
    try {
      var path = (global.location && global.location.pathname) || "";
      if (path.indexOf("/wizard/") !== 0) {
        return;
      }
      wyvernWizardState().catch(function (err) {
        if (typeof console !== "undefined" && console.warn) {
          console.warn("wyvern wizard bootstrap failed", err);
        }
      });
    } catch (_) {
      /* ignore */
    }
  }

  bootstrapWizardIfNeeded();

  global.wyvernWizardState = wyvernWizardState;
  global.wyvernWizardNext = wyvernWizardNext;
  global.wyvernWizardBack = wyvernWizardBack;
  global.wyvernWizardFinish = wyvernWizardFinish;

  global.WyvernApi = {
    fetchDialog: fetchDialog,
    postResult: postResult,
    postResultBeacon: postResultBeacon,
    postPickerFile: postPickerFile,
    postPickerFolder: postPickerFolder,
    notifyResize: notifyResize,
    scheduleResize: scheduleResize,
    resolveViewerSize: resolveViewerSize,
    applyDialogLayout: applyDialogLayout,
    applyEmbeddedChrome: applyEmbeddedChrome,
    wyvernWizardState: wyvernWizardState,
    wyvernWizardNext: wyvernWizardNext,
    wyvernWizardBack: wyvernWizardBack,
    wyvernWizardFinish: wyvernWizardFinish,
  };
})(typeof window !== "undefined" ? window : globalThis);

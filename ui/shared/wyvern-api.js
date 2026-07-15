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

  // Hard caps used only when no viewport bounds are available (browser /
  // --viewer none). Embedded path (ADR-0020 / REQ-V008) clamps to
  // available viewport × 0.92 via wyvern:viewport-bounds instead.
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
  /** Dialog auto-size slack (~25%; REQ-V008 / ADR-0020). */
  var DIALOG_SLACK = 1.25;
  /** Clamp sized window to this fraction of available viewport. */
  var VIEWPORT_CLAMP = 0.92;
  /** Refinement window after first resize (fonts / async assets). */
  var RESIZE_REFINE_MS = 300;

  /** Last `wyvern:viewport-bounds` detail from the embedded viewer. */
  var lastViewport = null;

  function normalizeViewport(viewport) {
    if (!viewport) {
      return null;
    }
    var w = Number(viewport.available_width);
    var h = Number(viewport.available_height);
    if (!isFinite(w) || !isFinite(h) || w <= 0 || h <= 0) {
      return null;
    }
    return { available_width: Math.round(w), available_height: Math.round(h) };
  }

  function rememberViewport(viewport) {
    var normalized = normalizeViewport(viewport);
    if (normalized) {
      lastViewport = normalized;
    }
    return lastViewport;
  }

  function onViewportBoundsEvent(event) {
    if (event && event.detail) {
      rememberViewport(event.detail);
    }
  }

  if (typeof global.addEventListener === "function") {
    global.addEventListener("wyvern:viewport-bounds", onViewportBoundsEvent);
  }
  if (global.__wyvernViewportBounds) {
    rememberViewport(global.__wyvernViewportBounds);
  }

  function postResize(w, h) {
    if (typeof window.ipc === "undefined" || typeof window.ipc.postMessage !== "function") {
      return false;
    }
    window.ipc.postMessage("resize:" + Math.round(w) + "x" + Math.round(h));
    return true;
  }

  function markDialogClamped(clamped) {
    var dialog = document.getElementById("dialog");
    if (!dialog) {
      return;
    }
    if (clamped) {
      dialog.classList.add("dialog--clamped");
    } else {
      dialog.classList.remove("dialog--clamped");
    }
  }

  function markWorkspaceRoot(isWorkspace) {
    var root =
      document.getElementById("dialog") ||
      document.querySelector("[data-testid='workspace-canvas']") ||
      document.body;
    if (!root) {
      return;
    }
    if (isWorkspace) {
      root.classList.add("dialog--workspace");
    } else {
      root.classList.remove("dialog--workspace");
    }
  }

  /**
   * Dialog fit: measure × slack, clamp to viewport × 0.92, scroll overflow.
   * `measure` accepts `{contentW,contentH}` or `{w,h}` (treated as content).
   */
  function applyDialogFitWithSlack(measure, viewport, slack) {
    measure = measure || {};
    slack = typeof slack === "number" && slack > 0 ? slack : DIALOG_SLACK;
    var contentW = Number(
      measure.contentW != null ? measure.contentW : measure.w != null ? measure.w : VIEWER_MIN_W
    );
    var contentH = Number(
      measure.contentH != null ? measure.contentH : measure.h != null ? measure.h : VIEWER_MIN_H
    );
    if (!isFinite(contentW) || contentW <= 0) contentW = VIEWER_MIN_W;
    if (!isFinite(contentH) || contentH <= 0) contentH = VIEWER_MIN_H;

    var w = Math.ceil(contentW * slack);
    var h = Math.ceil(contentH * slack);
    var vp = normalizeViewport(viewport) || lastViewport;
    var clamped = false;
    if (vp) {
      var maxW = Math.floor(vp.available_width * VIEWPORT_CLAMP);
      var maxH = Math.floor(vp.available_height * VIEWPORT_CLAMP);
      if (w > maxW) {
        w = maxW;
        clamped = true;
      }
      if (h > maxH) {
        h = maxH;
        clamped = true;
      }
    } else {
      // Browser / no-bounds fallback: Phase B hard caps (800×600). Embedded
      // viewer path uses viewport × 0.92 above (ADR-0020 / REQ-V008).
      if (w > VIEWER_MAX_W) {
        w = VIEWER_MAX_W;
        clamped = true;
      }
      if (h > VIEWER_MAX_H) {
        h = VIEWER_MAX_H;
        clamped = true;
      }
    }
    w = Math.max(VIEWER_MIN_W, w);
    h = Math.max(VIEWER_MIN_H, h);
    markDialogClamped(clamped);
    postResize(w, h);
    return { w: w, h: h, clamped: clamped };
  }

  /**
   * Workspace layout: command size → estimated_size → fill viewport (ADR-0006 opaque).
   */
  function applyWorkspaceLayout(state, viewport) {
    state = state || {};
    var vp = normalizeViewport(viewport) || lastViewport;
    markWorkspaceRoot(true);
    markDialogClamped(false);

    var w = null;
    var h = null;
    if (state.width && state.height) {
      w = Number(state.width);
      h = Number(state.height);
    } else {
      var est =
        (state.config && state.config.estimated_size) ||
        (global.wyvern && global.wyvern.config && global.wyvern.config.estimated_size) ||
        null;
      if (est && est.width && est.height) {
        w = Number(est.width);
        h = Number(est.height);
      }
    }

    if (!isFinite(w) || w <= 0 || !isFinite(h) || h <= 0) {
      if (vp) {
        w = Math.floor(vp.available_width * VIEWPORT_CLAMP);
        h = Math.floor(vp.available_height * VIEWPORT_CLAMP);
      } else {
        w = VIEWER_MAX_W;
        h = VIEWER_MAX_H;
      }
    } else if (vp) {
      w = Math.min(w, Math.floor(vp.available_width * VIEWPORT_CLAMP));
      h = Math.min(h, Math.floor(vp.available_height * VIEWPORT_CLAMP));
    }

    w = Math.max(VIEWER_MIN_W, Math.round(w));
    h = Math.max(VIEWER_MIN_H, Math.round(h));
    postResize(w, h);
    return { w: w, h: h, layout: "workspace" };
  }

  /** Resolve per-page layout: page.layout → config.layout → dialog. */
  function resolveWizardLayout(state) {
    state = state || {};
    var pageLayout = state.page && state.page.layout;
    if (pageLayout === "workspace" || pageLayout === "dialog") {
      return pageLayout;
    }
    var configLayout = state.config && state.config.layout;
    if (configLayout === "workspace" || configLayout === "dialog") {
      return configLayout;
    }
    return "dialog";
  }

  /**
   * Canonical wizard sizing entry: workspace or dialog-fit-with-slack.
   */
  function applyWizardLayout(state, viewport) {
    state = state || global.wyvern || {};
    var vp = normalizeViewport(viewport) || lastViewport;
    var layout = resolveWizardLayout(state);
    if (layout === "workspace") {
      return applyWorkspaceLayout(state, vp);
    }
    markWorkspaceRoot(false);
    var measure = measureNaturalContent();
    return applyDialogFitWithSlack(measure, vp, DIALOG_SLACK);
  }

  /** Natural content measure (no artificial viewer max during measure). */
  function measureNaturalContent() {
    var dialog = document.getElementById("dialog");
    if (dialog && !dialog.hidden) {
      if (dialog.classList.contains("dialog--fill")) {
        var fill = measureAtComfortWidth(dialog, Math.max(VIEWER_MAX_W, 1200));
        return {
          contentW: Math.max(fill.contentW + MEASURE_BUFFER, 420),
          contentH: fill.contentH + MEASURE_BUFFER,
        };
      }
      if (dialog.classList.contains("dialog--compact")) {
        var compact = measureAtComfortWidth(dialog, COMFORT_MAX_W);
        return {
          contentW: compact.contentW + MEASURE_BUFFER,
          contentH: compact.contentH + MEASURE_BUFFER,
        };
      }
      if (dialog.classList.contains("dialog--panel")) {
        var panel = measureIntrinsicPanel(dialog, PANEL_MAX_W);
        return {
          contentW: panel.contentW + MEASURE_BUFFER,
          contentH: panel.contentH + MEASURE_BUFFER,
        };
      }
      if (dialog.classList.contains("dialog--frame")) {
        var frame = measureIntrinsicPanel(dialog, VIEWER_MAX_W);
        return {
          contentW: Math.max(frame.contentW + MEASURE_BUFFER, FRAME_MIN_W),
          contentH: Math.max(frame.contentH + MEASURE_BUFFER, FRAME_MIN_H),
        };
      }
      var loose = measureAtComfortWidth(dialog, Math.max(VIEWER_MAX_W, 1200));
      return {
        contentW: loose.contentW + MEASURE_BUFFER,
        contentH: loose.contentH + MEASURE_BUFFER,
      };
    }
    var root = document.scrollingElement || document.documentElement;
    var body = document.body;
    var w = Math.ceil(
      Math.max(body ? body.scrollWidth : 0, body ? body.offsetWidth : 0, root.clientWidth)
    );
    var h = Math.ceil(
      Math.max(body ? body.scrollHeight : 0, body ? body.offsetHeight : 0, root.clientHeight)
    );
    return {
      contentW: Math.max(w, VIEWER_MIN_W),
      contentH: Math.max(h, VIEWER_MIN_H),
    };
  }

  function runWithResizeRefinement(applyFn) {
    requestAnimationFrame(function () {
      requestAnimationFrame(function () {
        applyFn();
        var done = false;
        function refineOnce() {
          if (done) return;
          done = true;
          applyFn();
        }
        if (document.fonts && document.fonts.ready && typeof document.fonts.ready.then === "function") {
          document.fonts.ready.then(function () {
            applyFn();
          });
        }
        setTimeout(applyFn, 0);
        setTimeout(refineOnce, Math.min(50, RESIZE_REFINE_MS));
        setTimeout(applyFn, Math.floor(RESIZE_REFINE_MS * 0.5));
        setTimeout(applyFn, RESIZE_REFINE_MS);
      });
    });
  }

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
    if (lastViewport) {
      applyDialogFitWithSlack(measureNaturalContent(), lastViewport, DIALOG_SLACK);
      return;
    }
    var size = measurePage();
    postResize(size.w, size.h);
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
    runWithResizeRefinement(function () {
      notifyResize();
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
      wyvernWizardState()
        .then(function (state) {
          runWithResizeRefinement(function () {
            applyWizardLayout(state, lastViewport);
          });
        })
        .catch(function (err) {
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
    applyDialogFitWithSlack: applyDialogFitWithSlack,
    applyWorkspaceLayout: applyWorkspaceLayout,
    applyWizardLayout: applyWizardLayout,
    measureNaturalContent: measureNaturalContent,
    applyEmbeddedChrome: applyEmbeddedChrome,
    wyvernWizardState: wyvernWizardState,
    wyvernWizardNext: wyvernWizardNext,
    wyvernWizardBack: wyvernWizardBack,
    wyvernWizardFinish: wyvernWizardFinish,
  };
})(typeof window !== "undefined" ? window : globalThis);

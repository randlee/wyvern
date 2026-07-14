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
    const res = await fetch("/api/result", {
      method: "POST",
      headers: { "Content-Type": "application/json", Accept: "application/json" },
      body: JSON.stringify(body),
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

  global.WyvernApi = {
    fetchDialog: fetchDialog,
    postResult: postResult,
    postResultBeacon: postResultBeacon,
    postPickerFile: postPickerFile,
    postPickerFolder: postPickerFolder,
  };
})(typeof window !== "undefined" ? window : globalThis);

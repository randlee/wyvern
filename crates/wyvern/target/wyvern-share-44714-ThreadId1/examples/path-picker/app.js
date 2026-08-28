/**
 * Path picker — collect file/folder path strings via native pickers.
 * Page JS must not read or write the filesystem (finish data is strings only).
 */
(function (global) {
  "use strict";

  function emptySelection() {
    return { file_paths: [], folder_paths: [] };
  }

  function asStringArray(value) {
    if (!Array.isArray(value)) {
      return [];
    }
    var out = [];
    value.forEach(function (item) {
      if (typeof item === "string" && item) {
        out.push(item);
      }
    });
    return out;
  }

  function uniqueAppend(list, extras) {
    extras.forEach(function (item) {
      if (list.indexOf(item) === -1) {
        list.push(item);
      }
    });
    return list;
  }

  function seedFromConfig() {
    var config = global.wyvern && global.wyvern.config;
    var seed = config && config.seed_paths ? config.seed_paths : {};
    return {
      file_paths: asStringArray(seed.file_paths),
      folder_paths: asStringArray(seed.folder_paths)
    };
  }

  function selectionFromStack() {
    var stack = (global.wyvern && Array.isArray(global.wyvern.stack)
      ? global.wyvern.stack
      : []) || [];
    for (var i = stack.length - 1; i >= 0; i -= 1) {
      var data = stack[i] && stack[i].data;
      if (data && (Array.isArray(data.file_paths) || Array.isArray(data.folder_paths))) {
        return {
          file_paths: asStringArray(data.file_paths),
          folder_paths: asStringArray(data.folder_paths)
        };
      }
    }
    var pageData = global.wyvern && global.wyvern.page_data;
    if (pageData && (Array.isArray(pageData.file_paths) || Array.isArray(pageData.folder_paths))) {
      return {
        file_paths: asStringArray(pageData.file_paths),
        folder_paths: asStringArray(pageData.folder_paths)
      };
    }
    return seedFromConfig();
  }

  function listItems(root) {
    if (!root) {
      return [];
    }
    var items = root.querySelectorAll("li");
    var out = [];
    for (var i = 0; i < items.length; i += 1) {
      var text = items[i].textContent;
      if (text) {
        out.push(text);
      }
    }
    return out;
  }

  function renderList(testId, paths) {
    var root = document.querySelector("[data-testid='" + testId + "']");
    if (!root) {
      return;
    }
    root.innerHTML = "";
    paths.forEach(function (path) {
      var li = document.createElement("li");
      li.textContent = path;
      root.appendChild(li);
    });
  }

  function showError(err) {
    var el = document.querySelector("[data-testid='wizard-error']");
    if (!el) {
      return;
    }
    el.hidden = false;
    el.textContent = String(err && err.message ? err.message : err);
  }

  function collectFromDomOrStack() {
    var fileRoot = document.querySelector("[data-testid='file-list']");
    var folderRoot = document.querySelector("[data-testid='folder-list']");
    if (fileRoot || folderRoot) {
      return {
        file_paths: listItems(fileRoot),
        folder_paths: listItems(folderRoot)
      };
    }
    return selectionFromStack();
  }

  function bindBrowse() {
    var fileBtn = document.querySelector("[data-testid='browse-file']");
    var folderBtn = document.querySelector("[data-testid='browse-folder']");
    if (!fileBtn && !folderBtn) {
      return;
    }
    var api = global.WyvernApi;
    if (!api) {
      return;
    }
    if (fileBtn) {
      fileBtn.addEventListener("click", function () {
        api.postPickerFile({ multiple: true }).then(function (picked) {
          if (!picked || !picked.ok || !Array.isArray(picked.paths)) {
            return;
          }
          var current = collectFromDomOrStack();
          renderList("file-list", uniqueAppend(current.file_paths, asStringArray(picked.paths)));
        }).catch(showError);
      });
    }
    if (folderBtn) {
      folderBtn.addEventListener("click", function () {
        api.postPickerFolder({}).then(function (picked) {
          if (!picked || !picked.ok || !Array.isArray(picked.paths)) {
            return;
          }
          var current = collectFromDomOrStack();
          renderList("folder-list", uniqueAppend(current.folder_paths, asStringArray(picked.paths)));
        }).catch(showError);
      });
    }
  }

  function fillSources() {
    if (!document.querySelector("[data-testid='path-sources']")) {
      return;
    }
    var data = selectionFromStack();
    renderList("file-list", data.file_paths);
    renderList("folder-list", data.folder_paths);
  }

  function fillReview() {
    if (!document.querySelector("[data-testid='path-review']")) {
      return;
    }
    var data = selectionFromStack();
    renderList("review-file-paths", data.file_paths);
    renderList("review-folder-paths", data.folder_paths);
  }

  global.collectCurrentPageData = function () {
    return collectFromDomOrStack();
  };

  global.wizardNextDescriptor = {
    id: "review",
    title: "Review paths",
    html: "pages/review.html"
  };

  function boot() {
    if (typeof global.wyvernWizardState === "function") {
      return global.wyvernWizardState().then(function () {
        fillSources();
        fillReview();
        bindBrowse();
      });
    }
    fillSources();
    fillReview();
    bindBrowse();
    return Promise.resolve();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () {
      boot();
    });
  } else {
    boot();
  }
})(typeof window !== "undefined" ? window : globalThis);

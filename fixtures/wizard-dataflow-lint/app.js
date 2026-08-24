(function (global) {
  "use strict";

  global.collectCurrentPageData = function () {
    var el = document.querySelector("[data-testid='field-label']");
    return { label: el ? String(el.value || "") : "" };
  };

  global.wizardNextDescriptor = {
    id: "two",
    title: "Step two",
    html: "pages/two.html"
  };
})(typeof window !== "undefined" ? window : globalThis);

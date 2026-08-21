(function (global) {
  "use strict";
  global.collectCurrentPageData = function () {
    return { label: "x" };
  };
  global.wizardNextDescriptor = {
    id: "two",
    title: "Two",
    html: "pages/two.html"
  };
})(typeof window !== "undefined" ? window : globalThis);

/**
 * Template picker — collect finish data from config.templates only.
 * Page JS must not scan the catalog directory (REQ-0125).
 */
(function (global) {
  "use strict";

  function templatesFromConfig() {
    var config = global.wyvern && global.wyvern.config;
    var list = config && Array.isArray(config.templates) ? config.templates : [];
    return list;
  }

  function testIdFor(id) {
    return String(id || "").replace(/\//g, "-");
  }

  function emptySelection() {
    return { template_id: "", variables: {}, output_path: "" };
  }

  function defaultsFor(tpl) {
    var variables = {};
    var vars = (tpl && Array.isArray(tpl.variables)) ? tpl.variables : [];
    vars.forEach(function (item) {
      if (item && item.name) {
        variables[item.name] = item.default == null ? "" : String(item.default);
      }
    });
    return {
      template_id: tpl && tpl.id ? String(tpl.id) : "",
      variables: variables,
      output_path: tpl && tpl.default_output_path ? String(tpl.default_output_path) : ""
    };
  }

  function selectionFromStack() {
    var stack = (global.wyvern && Array.isArray(global.wyvern.stack)
      ? global.wyvern.stack
      : []) || [];
    for (var i = stack.length - 1; i >= 0; i -= 1) {
      var data = stack[i] && stack[i].data;
      if (data && data.template_id) {
        return {
          template_id: String(data.template_id),
          variables: data.variables && typeof data.variables === "object" ? data.variables : {},
          output_path: data.output_path ? String(data.output_path) : ""
        };
      }
    }
    var pageData = global.wyvern && global.wyvern.page_data;
    if (pageData && pageData.template_id) {
      return {
        template_id: String(pageData.template_id),
        variables: pageData.variables && typeof pageData.variables === "object"
          ? pageData.variables
          : {},
        output_path: pageData.output_path ? String(pageData.output_path) : ""
      };
    }
    return emptySelection();
  }

  function findTemplate(id) {
    var list = templatesFromConfig();
    for (var i = 0; i < list.length; i += 1) {
      if (list[i] && list[i].id === id) {
        return list[i];
      }
    }
    return null;
  }

  function collectFormSelection() {
    var prior = selectionFromStack();
    var tpl = findTemplate(prior.template_id);
    var base = tpl ? defaultsFor(tpl) : emptySelection();
    if (prior.template_id) {
      base.template_id = prior.template_id;
    }
    var pathEl = document.querySelector("[data-testid='field-output-path']");
    if (pathEl) {
      base.output_path = pathEl.value;
    } else if (prior.output_path) {
      base.output_path = prior.output_path;
    }
    var variables = {};
    Object.keys(base.variables).forEach(function (name) {
      variables[name] = base.variables[name];
    });
    Object.keys(prior.variables || {}).forEach(function (name) {
      variables[name] = prior.variables[name];
    });
    var fields = document.querySelectorAll("[data-template-var]");
    for (var i = 0; i < fields.length; i += 1) {
      var name = fields[i].getAttribute("data-template-var");
      if (name) {
        variables[name] = fields[i].value;
      }
    }
    return {
      template_id: base.template_id,
      variables: variables,
      output_path: base.output_path
    };
  }

  function highlightSelected(id) {
    var cards = document.querySelectorAll("[data-template-id]");
    for (var i = 0; i < cards.length; i += 1) {
      if (cards[i].getAttribute("data-template-id") === id) {
        cards[i].classList.add("is-selected");
      } else {
        cards[i].classList.remove("is-selected");
      }
    }
  }

  function renderGrid() {
    var root = document.querySelector("[data-testid='template-grid']");
    if (!root) {
      return;
    }
    root.innerHTML = "";
    var list = templatesFromConfig();
    var selected = selectionFromStack();
    list.forEach(function (tpl) {
      if (!tpl || !tpl.id) {
        return;
      }
      var card = document.createElement("button");
      card.type = "button";
      card.className = "template-card";
      card.setAttribute("data-template-id", tpl.id);
      card.dataset.testid = "template-card-" + testIdFor(tpl.id);
      var label = document.createElement("strong");
      label.textContent = tpl.label || tpl.id;
      var idLine = document.createElement("span");
      idLine.className = "template-card__id";
      idLine.textContent = tpl.id;
      card.appendChild(label);
      card.appendChild(idLine);
      card.addEventListener("click", function () {
        var data = defaultsFor(tpl);
        highlightSelected(tpl.id);
        if (typeof global.wyvernWizardNext === "function") {
          global.wyvernWizardNext(data, {
            id: "form",
            title: "Customize template",
            html: "pages/form.html"
          }).catch(function (err) {
            var el = document.querySelector("[data-testid='wizard-error']");
            if (el) {
              el.hidden = false;
              el.textContent = String(err && err.message ? err.message : err);
            }
          });
        }
      });
      root.appendChild(card);
    });
    if (selected.template_id) {
      highlightSelected(selected.template_id);
    }
  }

  function fillForm() {
    var heading = document.querySelector("[data-testid='form-template-id']");
    if (!heading) {
      return;
    }
    var prior = selectionFromStack();
    var tpl = findTemplate(prior.template_id);
    var data = tpl ? defaultsFor(tpl) : emptySelection();
    if (prior.template_id) {
      data.template_id = prior.template_id;
    }
    if (prior.output_path) {
      data.output_path = prior.output_path;
    }
    Object.keys(prior.variables || {}).forEach(function (name) {
      data.variables[name] = prior.variables[name];
    });
    heading.textContent = data.template_id || "—";
    var pathEl = document.querySelector("[data-testid='field-output-path']");
    if (pathEl) {
      pathEl.value = data.output_path;
    }
    var holder = document.querySelector("[data-testid='variable-fields']");
    if (!holder) {
      return;
    }
    holder.innerHTML = "";
    var names = Object.keys(data.variables);
    names.forEach(function (name) {
      var label = document.createElement("label");
      label.className = "field";
      label.appendChild(document.createTextNode(name));
      var input = document.createElement("input");
      input.type = "text";
      input.setAttribute("data-template-var", name);
      input.dataset.testid = "field-var-" + name;
      input.value = data.variables[name];
      label.appendChild(input);
      holder.appendChild(label);
    });
  }

  function fillReview() {
    var idEl = document.querySelector("[data-testid='review-template-id']");
    if (!idEl) {
      return;
    }
    var data = collectFormSelection();
    idEl.textContent = data.template_id || "—";
    var pathEl = document.querySelector("[data-testid='review-output-path']");
    if (pathEl) {
      pathEl.textContent = data.output_path || "—";
    }
    var varsEl = document.querySelector("[data-testid='review-variables']");
    if (varsEl) {
      varsEl.textContent = JSON.stringify(data.variables || {});
    }
  }

  global.collectCurrentPageData = function () {
    if (document.querySelector("[data-testid='template-grid']")) {
      return selectionFromStack();
    }
    return collectFormSelection();
  };

  if (document.querySelector("[data-testid='template-grid']")) {
    global.wizardNextDescriptor = {
      id: "form",
      title: "Customize template",
      html: "pages/form.html"
    };
  } else if (document.querySelector("[data-testid='template-form']")) {
    global.wizardNextDescriptor = {
      id: "review",
      title: "Review template",
      html: "pages/review.html"
    };
  }

  function boot() {
    if (typeof global.wyvernWizardState === "function") {
      return global.wyvernWizardState().then(function () {
        renderGrid();
        fillForm();
        fillReview();
      });
    }
    renderGrid();
    fillForm();
    fillReview();
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

(function () {
  "use strict";

  const dialogEl = document.getElementById("dialog");
  const titleEl = document.getElementById("title");
  const cardsEl = document.getElementById("cards");
  const hintEl = document.getElementById("validation-hint");
  const submitBtn = document.getElementById("submit-btn");
  const errorEl = document.getElementById("error");

  let submitted = false;
  /** Verbatim questions for stdout echo (no preview_html). */
  let questionsEcho = [];
  /** Cards used for answer collection (question prompt keys). */
  let cards = [];

  function showError(err) {
    errorEl.hidden = false;
    errorEl.textContent = String(err && err.message ? err.message : err);
  }

  function stripPreviewHtml(questions) {
    return (questions || []).map(function (card) {
      var options = (card.options || []).map(function (opt) {
        var out = {
          label: opt.label,
          description: opt.description,
        };
        if (opt.preview != null) {
          out.preview = opt.preview;
        }
        return out;
      });
      return {
        question: card.question,
        header: card.header,
        options: options,
        multiSelect: !!card.multiSelect,
      };
    });
  }

  function dismissBody() {
    return {
      button: "dismissed",
      questions: questionsEcho,
      answers: {},
      response: "",
    };
  }

  function collectAnswers() {
    var answers = {};
    for (var i = 0; i < cards.length; i++) {
      var card = cards[i];
      var name = "q" + i;
      var selected = [];
      var nodes = document.querySelectorAll(
        'input[name="' + name + '"]:checked',
      );
      for (var j = 0; j < nodes.length; j++) {
        selected.push(nodes[j].value);
      }
      if (selected.length === 0) {
        return null;
      }
      answers[card.question] = selected.join(", ");
    }
    return answers;
  }

  async function submit() {
    if (submitted) return;
    var answers = collectAnswers();
    if (!answers) {
      hintEl.hidden = false;
      return;
    }
    hintEl.hidden = true;
    submitted = true;
    try {
      await WyvernApi.postResult({
        questions: questionsEcho,
        answers: answers,
        response: "",
      });
    } catch (err) {
      submitted = false;
      showError(err);
    }
  }

  function onBeforeUnload() {
    if (submitted) return;
    WyvernApi.postResultBeacon(dismissBody());
  }

  function renderCards(list) {
    cardsEl.innerHTML = "";
    cards = list || [];
    cards.forEach(function (card, qi) {
      var section = document.createElement("section");
      section.className = "question-card";
      section.setAttribute("data-index", String(qi));
      section.setAttribute("data-testid", "question-card-" + qi);

      var header = document.createElement("div");
      header.className = "card-header";
      header.textContent = card.header || "";
      section.appendChild(header);

      var prompt = document.createElement("div");
      prompt.className = "card-prompt";
      prompt.textContent = card.question || "";
      section.appendChild(prompt);

      var inputType = card.multiSelect ? "checkbox" : "radio";
      var groupName = "q" + qi;
      (card.options || []).forEach(function (opt, oi) {
        var id = "q" + qi + "-opt" + oi;
        var hasPreview = !!(opt.preview_html || opt.preview);
        var label = document.createElement("label");
        label.className = hasPreview ? "option-row has-preview" : "option-row";
        label.setAttribute("for", id);
        label.setAttribute("data-testid", "option-q" + qi + "-o" + oi);

        var input = document.createElement("input");
        input.type = inputType;
        input.id = id;
        input.name = groupName;
        input.value = opt.label || "";
        label.appendChild(input);

        var text = document.createElement("span");
        text.className = "option-text";
        var labelSpan = document.createElement("span");
        labelSpan.className = "option-label";
        labelSpan.textContent = opt.label || "";
        text.appendChild(labelSpan);
        var desc = document.createElement("div");
        desc.className = "option-description";
        desc.textContent = opt.description || "";
        text.appendChild(desc);
        label.appendChild(text);

        if (opt.preview_html) {
          var preview = document.createElement("div");
          preview.className = "option-preview";
          preview.setAttribute("data-testid", "preview-q" + qi + "-o" + oi);
          // preview_html is server-sanitized (pulldown-cmark + ammonia).
          preview.innerHTML = opt.preview_html;
          label.appendChild(preview);
        }

        section.appendChild(label);
      });

      cardsEl.appendChild(section);
    });
  }

  submitBtn.addEventListener("click", function () {
    submit();
  });

  WyvernApi.fetchDialog()
    .then(function (payload) {
      if (payload.type !== "question") {
        throw new Error("expected question dialog, got " + payload.type);
      }
      document.title = payload.title || "Wyvern Question";
      titleEl.textContent = payload.title || "Question";
      questionsEcho = stripPreviewHtml(payload.questions);
      renderCards(payload.questions || []);
      dialogEl.hidden = false;
      window.addEventListener("beforeunload", onBeforeUnload);
    })
    .catch(showError);
})();

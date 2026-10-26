import { EditorState, StateEffect, StateField } from "@codemirror/state";
import { Decoration, EditorView, drawSelection, keymap, placeholder } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { groupFor, rules } from "./review.js";

const VIEWPORT_BUFFER = 1200;
const setHighlights = StateEffect.define();
const highlights = StateField.define({
  create: () => Decoration.none,
  update(value, transaction) {
    if (transaction.docChanged) value = Decoration.none;
    for (const effect of transaction.effects) {
      if (!effect.is(setHighlights)) continue;
      const marks = effect.value.filter((item) => item.start < item.end && item.start >= 0 && item.end <= transaction.state.doc.length).map((item) => {
        const group = groupFor(item.rule);
        const span = ["monotony", "repeated-starter", "tense-drift", "pov-drift", "long-sentence"].includes(item.rule);
        return Decoration.mark({
          class: `cm-finding finding-${group?.id || "style"}${span ? " cm-finding-span" : ""}`,
          attributes: { "data-rule": item.rule, title: `${rules[item.rule]?.label || item.rule}: ${item.message}. Click for details.` },
        }).range(item.start, item.end);
      });
      value = Decoration.set(marks, true);
    }
    return value;
  },
  provide: (field) => EditorView.decorations.from(field),
});

export function createEditor(parent, text, callbacks) {
  const view = new EditorView({
    parent,
    state: EditorState.create({
      doc: text,
      extensions: [
        history(),
        drawSelection(),
        EditorView.lineWrapping,
        EditorView.theme({}, { dark: true }),
        placeholder("Write, paste, or open your manuscript…"),
        highlights,
        keymap.of([
          { key: "Alt-Enter", run: () => callbacks.inspect(view.state.selection.main.head) },
          { key: "Escape", run: () => callbacks.close() },
          ...defaultKeymap,
          ...historyKeymap,
        ]),
        EditorView.contentAttributes.of({
          "aria-label": "Manuscript editor",
          "aria-description": "Colored underlines mark observations. Click a marked passage, or press Alt+Enter at the cursor, for details.",
          spellcheck: "false",
        }),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) callbacks.change();
          if (update.viewportChanged || update.geometryChanged || update.docChanged) callbacks.viewport();
        }),
        EditorView.domEventHandlers({
          scroll: () => { callbacks.viewport(); },
          click(event) {
            if (!event.target.closest?.(".cm-finding")) return false;
            const position = view.posAtCoords({ x: event.clientX, y: event.clientY });
            if (position !== null) callbacks.inspect(position);
            return false;
          },
        }),
      ],
    }),
  });
  return {
    view,
    text: () => view.state.doc.toString(),
    length: () => view.state.doc.length,
    replace(text) { view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: text }, selection: { anchor: 0 } }); },
    highlight(items) { view.dispatch({ effects: setHighlights.of(items) }); },
    jump(from, to = from) {
      from = Math.max(0, Math.min(from, view.state.doc.length));
      to = Math.max(from, Math.min(to, view.state.doc.length));
      view.dispatch({ selection: { anchor: from, head: to }, effects: EditorView.scrollIntoView(from, { y: "center" }) });
      view.focus();
    },
    range() {
      const rect = view.scrollDOM.getBoundingClientRect();
      const content = view.contentDOM.getBoundingClientRect();
      const x = Math.max(rect.left + 8, Math.min(rect.right - 8, content.left + Math.min(content.width / 2, 120)));
      const first = view.posAtCoords({ x, y: rect.top + 8 }, false);
      const last = view.posAtCoords({ x, y: rect.bottom - 8 }, false);
      const visibleFrom = Math.min(first, last);
      const visibleTo = Math.max(first, last);
      const from = Math.max(0, visibleFrom - VIEWPORT_BUFFER);
      const to = Math.min(view.state.doc.length, visibleTo + VIEWPORT_BUFFER);
      return { from, to, visibleFrom, visibleTo, fromLine: view.state.doc.lineAt(visibleFrom).number, toLine: view.state.doc.lineAt(visibleTo).number };
    },
  };
}

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

use gtk::cairo;
use gtk::gdk;
use gtk::glib;
use gtk::prelude::*;
use ink_stroke_modeler_rs::{ModelerInput, ModelerInputEventType, ModelerParams, StrokeModeler};
use uuid::Uuid;

use crate::engine::OpenDocument;
use crate::engine::document::{
    A4, Annotation, AnnotationKind, Color, DEFAULT_PATTERN_SPACING, Document, Page, PageKind,
    PagePattern, ShapeAnnotation, ShapeKind, StrokeAnnotation, TextAnnotation, TextRun, TextStyle,
};
use crate::engine::pdf::PdfDocument;

const PAGE_GAP: f64 = 16.0;
const MIN_ZOOM: f64 = 0.1;
const MAX_ZOOM: f64 = 6.0;
const ZOOM_STEP: f64 = 1.25;
const TEXT_SIZE: f64 = 16.0;
const MIN_BOX_WIDTH: f64 = 4.0;
/// Squared pixel distance a press must move before it counts as a drag (not a click).
const DRAG_THRESHOLD_SQ: f64 = 9.0;
// Canvas backdrop behind the pages, per theme (r, g, b).
const CANVAS_BG_DARK: (f64, f64, f64) = (0.18, 0.18, 0.20);
const CANVAS_BG_LIGHT: (f64, f64, f64) = (0.86, 0.86, 0.88);
// Bounding-box colors (r, g, b, a).
const BOX_ACTIVE: (f64, f64, f64, f64) = (0.20, 0.51, 0.92, 1.0);
const BOX_ANNOTATION: (f64, f64, f64, f64) = (0.55, 0.55, 0.60, 0.9);
const SELECTION_FILL: (f64, f64, f64, f64) = (0.20, 0.51, 0.92, 0.30);

/// A single character with its own style (color, font, highlight, weight).
#[derive(Clone)]
struct Glyph {
    ch: char,
    style: TextStyle,
}

/// A text box currently being typed. `x`/`y` are the top-left anchor in page points.
struct TextEdit {
    page: usize,
    x: f64,
    y: f64,
    size: f64,
    glyphs: Vec<Glyph>,
    /// Caret position as a glyph (character) index.
    cursor: usize,
    /// Selection anchor (glyph index); the selection spans anchor..cursor.
    anchor: Option<usize>,
    id: Uuid,
    /// The annotation this edit replaces, kept so Escape can restore it.
    original: Option<Annotation>,
}

impl TextEdit {
    /// The selected glyph range, if any (non-empty).
    fn selection(&self) -> Option<(usize, usize)> {
        self.anchor
            .map(|a| (a.min(self.cursor), a.max(self.cursor)))
            .filter(|(s, e)| s != e)
    }

    fn delete_selection(&mut self) -> bool {
        if let Some((s, e)) = self.selection() {
            self.glyphs.drain(s..e);
            self.cursor = s;
            self.anchor = None;
            true
        } else {
            false
        }
    }

    fn insert(&mut self, ch: char, style: TextStyle) {
        self.delete_selection();
        self.glyphs.insert(self.cursor, Glyph { ch, style });
        self.cursor += 1;
        self.anchor = None;
    }

    fn backspace(&mut self) {
        if self.delete_selection() {
            return;
        }
        if self.cursor > 0 {
            self.glyphs.remove(self.cursor - 1);
            self.cursor -= 1;
        }
    }

    fn delete_forward(&mut self) {
        if self.delete_selection() {
            return;
        }
        if self.cursor < self.glyphs.len() {
            self.glyphs.remove(self.cursor);
        }
    }

    fn set_cursor(&mut self, pos: usize, extend: bool) {
        if extend {
            if self.anchor.is_none() {
                self.anchor = Some(self.cursor);
            }
        } else {
            self.anchor = None;
        }
        self.cursor = pos.min(self.glyphs.len());
    }

    fn line_start(&self, pos: usize) -> usize {
        let mut i = pos;
        while i > 0 && self.glyphs[i - 1].ch != '\n' {
            i -= 1;
        }
        i
    }

    fn line_end(&self, pos: usize) -> usize {
        let mut i = pos;
        while i < self.glyphs.len() && self.glyphs[i].ch != '\n' {
            i += 1;
        }
        i
    }

    fn move_left(&mut self, extend: bool) {
        let p = self.cursor.saturating_sub(1);
        self.set_cursor(p, extend);
    }

    fn move_right(&mut self, extend: bool) {
        let p = (self.cursor + 1).min(self.glyphs.len());
        self.set_cursor(p, extend);
    }

    /// Word boundary to the left of `pos`: skip whitespace, then the word itself.
    fn word_left(&self, pos: usize) -> usize {
        let mut i = pos;
        while i > 0 && self.glyphs[i - 1].ch.is_whitespace() {
            i -= 1;
        }
        while i > 0 && !self.glyphs[i - 1].ch.is_whitespace() {
            i -= 1;
        }
        i
    }

    /// Word boundary to the right of `pos`: skip whitespace, then the word itself.
    fn word_right(&self, pos: usize) -> usize {
        let mut i = pos;
        while i < self.glyphs.len() && self.glyphs[i].ch.is_whitespace() {
            i += 1;
        }
        while i < self.glyphs.len() && !self.glyphs[i].ch.is_whitespace() {
            i += 1;
        }
        i
    }

    fn move_word_left(&mut self, extend: bool) {
        let p = self.word_left(self.cursor);
        self.set_cursor(p, extend);
    }

    fn move_word_right(&mut self, extend: bool) {
        let p = self.word_right(self.cursor);
        self.set_cursor(p, extend);
    }

    fn move_home(&mut self, extend: bool) {
        let p = self.line_start(self.cursor);
        self.set_cursor(p, extend);
    }

    fn move_end(&mut self, extend: bool) {
        let p = self.line_end(self.cursor);
        self.set_cursor(p, extend);
    }

    fn move_up(&mut self, extend: bool) {
        self.move_line(-1, extend);
    }

    fn move_down(&mut self, extend: bool) {
        self.move_line(1, extend);
    }

    /// Moves the caret one line up/down, keeping the same character column.
    fn move_line(&mut self, dir: isize, extend: bool) {
        let start = self.line_start(self.cursor);
        let col = self.cursor - start;
        if dir < 0 {
            if start == 0 {
                return;
            }
            let prev_start = self.line_start(start - 1);
            let prev_end = start - 1; // the '\n' ending the previous line
            self.set_cursor((prev_start + col).min(prev_end), extend);
        } else {
            let end = self.line_end(self.cursor);
            if end >= self.glyphs.len() {
                return;
            }
            let next_start = end + 1;
            let next_end = self.line_end(next_start);
            self.set_cursor((next_start + col).min(next_end), extend);
        }
    }

    fn select_all(&mut self) {
        self.anchor = Some(0);
        self.cursor = self.glyphs.len();
    }

    /// Applies `f` to every selected glyph's style; returns whether a selection existed.
    fn style_selection(&mut self, f: impl Fn(&mut TextStyle)) -> bool {
        if let Some((s, e)) = self.selection() {
            for g in &mut self.glyphs[s..e] {
                f(&mut g.style);
            }
            true
        } else {
            false
        }
    }

    /// Merges consecutive same-style glyphs into runs for storage.
    fn to_runs(&self) -> Vec<TextRun> {
        let mut runs: Vec<TextRun> = Vec::new();
        for g in &self.glyphs {
            match runs.last_mut() {
                Some(last) if last.style == g.style => last.text.push(g.ch),
                _ => runs.push(TextRun { text: g.ch.to_string(), style: g.style.clone() }),
            }
        }
        runs
    }

    fn is_blank(&self) -> bool {
        self.glyphs.iter().all(|g| g.ch.is_whitespace())
    }
}

/// An annotation lifted out of the model while being dragged (any kind).
/// `orig` is the untranslated snapshot taken at drag start, so each frame's
/// position is `translate(orig, clamped total offset)` - never accumulated.
struct DragState {
    page: usize,
    annotation: Annotation,
    orig: AnnotationKind,
}

const PEN_WIDTH: f64 = 3.0;
const SHAPE_WIDTH: f64 = 3.0;
const ERASER_WIDTH: f64 = 10.0;
/// Smallest allowed page dimension in points.
const MIN_PAGE: f64 = 50.0;

/// Live smoothing state for the pen: Google's ink-stroke-modeler (the same one
/// Rnote uses) turns raw input events into a dejittered, upsampled point stream.
struct PenModel {
    modeler: StrokeModeler,
    started: Instant,
    last_time: f64,
    last_pos: (f64, f64),
    /// Predicted tail so the preview keeps up with the cursor despite the
    /// spring-model lag; never committed to the document.
    prediction: Vec<(f64, f64)>,
}

impl PenModel {
    fn new(pos: (f64, f64)) -> Self {
        // Upstream suggested params assume centimeters; speeds/distances are
        // rescaled to PDF points (1 cm ≈ 28.35 pt).
        let params = ModelerParams {
            wobble_smoother_speed_floor: 37.0,
            wobble_smoother_speed_ceiling: 41.0,
            sampling_end_of_stroke_stopping_distance: 0.03,
            ..ModelerParams::suggested()
        };
        Self {
            modeler: StrokeModeler::new(params).expect("static modeler params are valid"),
            started: Instant::now(),
            last_time: 0.0,
            last_pos: pos,
            prediction: Vec::new(),
        }
    }

    /// Feeds one input event and returns the newly modeled points.
    fn feed(&mut self, event_type: ModelerInputEventType, pos: (f64, f64), time: f64) -> Vec<(f64, f64)> {
        self.last_time = time;
        self.last_pos = pos;
        self.modeler
            .update(ModelerInput { event_type, pos, time, pressure: 1.0 })
            .map(|out| out.iter().map(|r| r.pos).collect())
            .unwrap_or_default()
    }

    fn refresh_prediction(&mut self) {
        self.prediction = self
            .modeler
            .predict()
            .map(|out| out.iter().map(|r| r.pos).collect())
            .unwrap_or_default();
    }
}

/// An in-progress drawing/erasing gesture (page-local coordinates in points).
enum Draw {
    Stroke { page: usize, points: Vec<(f64, f64)>, color: Color, width: f64, model: Box<PenModel> },
    Shape { page: usize, shape: ShapeKind, start: (f64, f64), end: (f64, f64), color: Color, width: f64 },
    /// Erasing: `baseline` is the page snapshot before this drag, recorded on commit
    /// if anything was actually removed.
    Erase { baseline: Vec<Page>, changed: bool },
}

/// Max snapshots kept per direction (older ones are dropped).
const HISTORY_LIMIT: usize = 100;

/// Undo/redo history. Each entry is a snapshot of the page list (annotations +
/// page structure); the embedded PDF bytes are never copied, so history stays
/// cheap even for large documents.
#[derive(Default)]
struct History {
    undo: Vec<Vec<Page>>,
    redo: Vec<Vec<Page>>,
}

impl History {
    /// Records `snapshot` as a new undo point and invalidates the redo stack.
    fn record(&mut self, snapshot: Vec<Page>) {
        self.undo.push(snapshot);
        if self.undo.len() > HISTORY_LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    /// Steps one entry: pops the target stack (redo if `forward`, else undo) and
    /// pushes `current` onto the opposite one. Returns the snapshot to apply, or
    /// `None` (leaving `current` untouched) when the target stack is empty.
    fn step(&mut self, current: Vec<Page>, forward: bool) -> Option<Vec<Page>> {
        let (from, to) = if forward {
            (&mut self.redo, &mut self.undo)
        } else {
            (&mut self.undo, &mut self.redo)
        };
        let prev = from.pop()?;
        to.push(current);
        Some(prev)
    }
}

/// The transient bits drawn on top of the cached page surfaces.
struct Overlay<'a> {
    text_mode: bool,
    editing: Option<&'a TextEdit>,
    dragging: Option<&'a DragState>,
    selected: Option<(usize, Uuid)>,
    drawing: Option<&'a Draw>,
    lasso_selected: Option<&'a (usize, Vec<Uuid>)>,
    lasso_op: Option<&'a LassoOp>,
}

struct State {
    doc: Option<Document>,
    pdf: Option<PdfDocument>,
    zoom: f64,
    /// Rendered pages keyed by index; cleared on zoom or structure change.
    cache: HashMap<usize, cairo::ImageSurface>,
    text_mode: bool,
    editing: Option<TextEdit>,
    /// Page nearest the viewport center; gets the accent frame and anchors insert/delete.
    current: usize,
    /// Candidate for a box drag: (page, annotation index).
    drag_start: Option<(usize, usize)>,
    dragging: Option<DragState>,
    /// Widget-space start point while drag-selecting text inside the edited box.
    text_drag: Option<(f64, f64)>,
    /// Currently selected annotation (page, id) in move/select mode.
    selected: Option<(usize, Uuid)>,
    /// Font size for new text boxes (and the one being edited).
    text_size: f64,
    /// Style for newly typed characters (color, font, weight). Also applied to the
    /// current selection when a style control changes.
    text_style: TextStyle,
    /// Undo/redo snapshots.
    history: History,
    /// Page snapshot taken when the current edit session began, so the whole
    /// session (typing + styling) collapses into one undo entry on commit.
    edit_baseline: Option<Vec<Page>>,
    /// The active tool (drives drawing, erasing, and the cursor).
    tool: Tool,
    /// In-progress pen stroke / shape / erase gesture.
    draw_op: Option<Draw>,
    /// Widget-space start point of the active draw gesture (offsets are relative to it).
    draw_origin: (f64, f64),
    pen_color: Color,
    pen_width: f64,
    shape_kind: ShapeKind,
    shape_color: Color,
    shape_width: f64,
    eraser_width: f64,
    /// Ruling applied to newly inserted blank pages (and, when changed via
    /// `set_blank_pattern`/`set_pattern_spacing`, retroactively to the current
    /// blank page).
    blank_pattern: PagePattern,
    blank_pattern_spacing: f64,
    /// Multi-selected annotations for the Lasso tool: (page, ids).
    lasso_selected: Option<(usize, Vec<Uuid>)>,
    /// Widget-space point where a Lasso-tool press began.
    lasso_press: Option<(f64, f64)>,
    /// (page, x, y) in page-local points where the press landed - promoted to
    /// `lasso_op` once the drag exceeds the threshold (mirrors `drag_start` ->
    /// `dragging`).
    lasso_start: Option<(usize, f64, f64)>,
    /// The in-progress Lasso gesture, once the drag has exceeded the threshold.
    lasso_op: Option<LassoOp>,
}

/// Where an insert/delete acts relative to the current page.
#[derive(Clone, Copy)]
pub enum Relative {
    Before,
    After,
}

/// The selectable tools.
#[derive(Clone, Copy, PartialEq)]
pub enum Tool {
    Select,
    Text,
    Pen,
    Eraser,
    Shape,
    /// Rectangle multi-select: drag a marquee to select strokes/shapes/text,
    /// then move or bulk-restyle the group, or delete it.
    Lasso,
    Markdown,
    Pages,
}

/// An in-progress Lasso-tool gesture.
enum LassoOp {
    /// Marquee rectangle from `start` to `current` (page-local points).
    Marquee { page: usize, start: (f64, f64), current: (f64, f64) },
    /// Group-dragging the lifted annotations; `orig` is the pre-drag snapshot,
    /// parallel to `lifted` by index, and translated fresh from it each frame.
    Move { page: usize, orig: Vec<AnnotationKind>, lifted: Vec<Annotation> },
}

#[derive(Clone)]
pub struct Canvas {
    pub root: gtk::ScrolledWindow,
    area: gtk::DrawingArea,
    state: Rc<RefCell<State>>,
}

impl Canvas {
    pub fn new() -> Self {
        let area = gtk::DrawingArea::builder().focusable(true).build();
        let root = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .child(&area)
            .build();

        // Stop the ScrolledWindow from scrolling to the top whenever the canvas
        // grabs keyboard focus (e.g. on click to place text).
        if let Some(viewport) = root.child().and_downcast::<gtk::Viewport>() {
            viewport.set_scroll_to_focus(false);
        }

        let state = Rc::new(RefCell::new(State {
            doc: None,
            pdf: None,
            zoom: 1.0,
            cache: HashMap::new(),
            text_mode: false,
            editing: None,
            current: 0,
            drag_start: None,
            dragging: None,
            text_drag: None,
            selected: None,
            text_size: TEXT_SIZE,
            text_style: TextStyle::default(),
            history: History::default(),
            edit_baseline: None,
            tool: Tool::Select,
            draw_op: None,
            draw_origin: (0.0, 0.0),
            pen_color: Color::BLACK,
            pen_width: PEN_WIDTH,
            shape_kind: ShapeKind::Rectangle,
            shape_color: Color::BLACK,
            shape_width: SHAPE_WIDTH,
            eraser_width: ERASER_WIDTH,
            blank_pattern: PagePattern::Plain,
            blank_pattern_spacing: DEFAULT_PATTERN_SPACING,
            lasso_selected: None,
            lasso_press: None,
            lasso_start: None,
            lasso_op: None,
        }));

        {
            let state = state.clone();
            area.set_draw_func(move |_area, ctx, width, _height| {
                draw(&state, ctx, width);
            });
        }

        let canvas = Self { root, area, state };
        canvas.attach_input();
        canvas
    }

    fn attach_input(&self) {
        let click = gtk::GestureClick::new();
        {
            let this = self.clone();
            click.connect_pressed(move |_, n_press, x, y| this.on_click(n_press, x, y));
        }
        self.area.add_controller(click);

        let keys = gtk::EventControllerKey::new();
        {
            let this = self.clone();
            keys.connect_key_pressed(move |_, keyval, _, state| this.on_key(keyval, state));
        }
        self.area.add_controller(keys);

        let drag = gtk::GestureDrag::new();
        {
            let this = self.clone();
            drag.connect_drag_begin(move |_, x, y| this.on_drag_begin(x, y));
        }
        {
            let this = self.clone();
            drag.connect_drag_update(move |_, ox, oy| this.on_drag_update(ox, oy));
        }
        {
            let this = self.clone();
            drag.connect_drag_end(move |_, _, _| this.on_drag_end());
        }
        self.area.add_controller(drag);

        // Track which page is in view so the current-page frame follows scrolling.
        let this = self.clone();
        self.root.vadjustment().connect_value_changed(move |_| this.recompute_current());

        // Repaint the backdrop when the light/dark theme changes.
        let area = self.area.clone();
        adw::StyleManager::default().connect_dark_notify(move |_| area.queue_draw());

        // Ctrl + mouse wheel zooms instead of scrolling. Capture phase so we claim
        // the event before the ScrolledWindow scrolls it.
        let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        scroll.set_propagation_phase(gtk::PropagationPhase::Capture);
        let this = self.clone();
        scroll.connect_scroll(move |ctrl, _dx, dy| {
            if ctrl.current_event_state().contains(gdk::ModifierType::CONTROL_MASK) {
                if dy < 0.0 {
                    this.zoom_in();
                } else if dy > 0.0 {
                    this.zoom_out();
                }
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        self.root.add_controller(scroll);
    }

    pub fn set_open_document(&self, open: OpenDocument) {
        self.set_open_document_with_zoom(open, 1.0);
    }

    /// Like `set_open_document`, but restores a specific zoom instead of resetting
    /// to 1.0 (used when switching back to a tab that had its own zoom level).
    pub fn set_open_document_with_zoom(&self, open: OpenDocument, zoom: f64) {
        {
            let mut st = self.state.borrow_mut();
            st.editing = None;
            st.drag_start = None;
            st.dragging = None;
            st.text_drag = None;
            st.selected = None;
            st.history = History::default();
            st.edit_baseline = None;
            st.draw_op = None;
            st.doc = Some(open.model);
            st.pdf = open.pdf;
            st.zoom = zoom;
            st.cache.clear();
        }
        self.update_layout();
    }

    /// Snapshot of the current document model (for saving).
    pub fn document(&self) -> Option<Document> {
        self.commit_editing();
        self.state.borrow().doc.clone()
    }

    /// Takes the document + pdf handle out of the canvas (e.g. when switching away
    /// from a tab), leaving the canvas without an open document. Unlike
    /// `document()`, this does not clone: the pdf handle isn't `Clone`.
    pub fn take_open_document(&self) -> Option<OpenDocument> {
        self.commit_editing();
        let mut st = self.state.borrow_mut();
        let model = st.doc.take()?;
        let pdf = st.pdf.take();
        Some(OpenDocument { model, pdf })
    }

    /// Pushes the current page state onto the undo stack before a mutation.
    fn record_change(&self) {
        let mut st = self.state.borrow_mut();
        let snapshot = st.doc.as_ref().map(|d| d.pages.clone());
        if let Some(snapshot) = snapshot {
            st.history.record(snapshot);
        }
    }

    /// Remembers the page state at the start of an edit session so the whole
    /// session becomes a single undo entry (recorded on commit, if it changed).
    fn begin_edit_session(&self) {
        let mut st = self.state.borrow_mut();
        st.edit_baseline = st.doc.as_ref().map(|d| d.pages.clone());
    }

    /// Reverts to the previous page snapshot.
    pub fn undo(&self) {
        self.commit_editing();
        let changed = self.swap_history(false);
        if changed {
            self.update_layout();
        }
    }

    /// Re-applies a snapshot undone by `undo`.
    pub fn redo(&self) {
        self.commit_editing();
        let changed = self.swap_history(true);
        if changed {
            self.update_layout();
        }
    }

    /// Moves one step through history (redo if `forward`, else undo), swapping the
    /// stored snapshot in for the current pages. Returns whether anything changed.
    fn swap_history(&self, forward: bool) -> bool {
        let mut st = self.state.borrow_mut();
        let State { doc, history, selected, cache, .. } = &mut *st;
        let Some(doc) = doc.as_mut() else {
            return false;
        };
        let Some(prev) = history.step(doc.pages.clone(), forward) else {
            return false;
        };
        doc.pages = prev;
        *selected = None;
        cache.clear();
        true
    }

    pub fn set_text_mode(&self, on: bool) {
        if !on {
            self.commit_editing();
        }
        self.state.borrow_mut().text_mode = on;
        // Focus is grabbed on click (placing/editing text), not here, so toggling
        // the tool does not make the ScrolledWindow jump to the top.
        self.area.queue_draw();
    }

    /// Selects the active tool.
    pub fn set_tool(&self, tool: Tool) {
        // Leaving text editing commits it; other tools have no pending state here.
        self.set_text_mode(tool == Tool::Text);
        self.state.borrow_mut().tool = tool;
        self.update_cursor();
    }

    /// Sets the font size for new text boxes and the one currently being edited.
    pub fn set_text_size(&self, size: f64) {
        {
            let mut st = self.state.borrow_mut();
            st.text_size = size;
            if let Some(ed) = st.editing.as_mut() {
                ed.size = size;
            }
        }
        self.update_cursor();
        self.area.queue_draw();
    }

    pub fn text_size(&self) -> f64 {
        self.state.borrow().text_size
    }

    /// If the current selection's kind matches `applies_to`, applies `f` to it
    /// (one undo entry). No-op otherwise - used so a color/width/kind control
    /// also restyles the current stroke/shape selection, not just future ones.
    fn apply_to_selected_if(&self, applies_to: fn(&AnnotationKind) -> bool, f: impl FnOnce(&mut AnnotationKind)) {
        let Some((page, id)) = self.state.borrow().selected else {
            return;
        };
        let applies = self
            .state
            .borrow()
            .doc
            .as_ref()
            .and_then(|d| d.pages.get(page))
            .and_then(|p| p.annotations.iter().find(|a| a.id == id))
            .is_some_and(|a| applies_to(&a.kind));
        if !applies {
            return;
        }
        self.record_change();
        let mut st = self.state.borrow_mut();
        if let Some(annotation) =
            st.doc.as_mut().and_then(|d| d.pages.get_mut(page)).and_then(|p| p.annotations.iter_mut().find(|a| a.id == id))
        {
            f(&mut annotation.kind);
        }
        st.cache.remove(&page);
        drop(st);
        self.area.queue_draw();
    }

    pub fn set_pen_color(&self, color: Color) {
        self.state.borrow_mut().pen_color = color;
        self.apply_to_selected_if(
            |k| matches!(k, AnnotationKind::Stroke(_)),
            move |k| {
                if let AnnotationKind::Stroke(s) = k {
                    s.color = color;
                }
            },
        );
    }

    pub fn pen_color(&self) -> Color {
        self.state.borrow().pen_color
    }

    pub fn set_pen_width(&self, width: f64) {
        self.state.borrow_mut().pen_width = width;
        self.apply_to_selected_if(
            |k| matches!(k, AnnotationKind::Stroke(_)),
            move |k| {
                if let AnnotationKind::Stroke(s) = k {
                    s.width = width;
                }
            },
        );
    }

    pub fn pen_width(&self) -> f64 {
        self.state.borrow().pen_width
    }

    pub fn set_shape_kind(&self, kind: ShapeKind) {
        self.state.borrow_mut().shape_kind = kind;
        self.apply_to_selected_if(
            |k| matches!(k, AnnotationKind::Shape(_)),
            move |k| {
                if let AnnotationKind::Shape(s) = k {
                    s.shape = kind;
                }
            },
        );
    }

    pub fn shape_kind(&self) -> ShapeKind {
        self.state.borrow().shape_kind
    }

    pub fn set_shape_color(&self, color: Color) {
        self.state.borrow_mut().shape_color = color;
        self.apply_to_selected_if(
            |k| matches!(k, AnnotationKind::Shape(_)),
            move |k| {
                if let AnnotationKind::Shape(s) = k {
                    s.color = color;
                }
            },
        );
    }

    pub fn shape_color(&self) -> Color {
        self.state.borrow().shape_color
    }

    pub fn set_shape_width(&self, width: f64) {
        self.state.borrow_mut().shape_width = width;
        self.apply_to_selected_if(
            |k| matches!(k, AnnotationKind::Shape(_)),
            move |k| {
                if let AnnotationKind::Shape(s) = k {
                    s.width = width;
                }
            },
        );
    }

    pub fn shape_width(&self) -> f64 {
        self.state.borrow().shape_width
    }

    pub fn set_eraser_width(&self, width: f64) {
        self.state.borrow_mut().eraser_width = width;
        self.update_cursor();
    }

    pub fn eraser_width(&self) -> f64 {
        self.state.borrow().eraser_width
    }

    /// Sets the font color: becomes the color for newly typed characters and, while
    /// editing, recolors the current selection.
    pub fn set_text_color(&self, color: Color) {
        self.apply_style(move |s| s.color = color);
    }

    pub fn text_color(&self) -> Color {
        self.state.borrow().text_style.color
    }

    /// Sets the font family (see `set_text_color` for the new-text/selection split).
    pub fn set_text_font(&self, font: String) {
        self.apply_style(move |s| s.font = font.clone());
    }

    pub fn text_font(&self) -> String {
        self.state.borrow().text_style.font.clone()
    }

    pub fn toggle_bold(&self) {
        self.toggle_style(|s| s.bold, |s, on| s.bold = on);
    }

    pub fn toggle_italic(&self) {
        self.toggle_style(|s| s.italic, |s, on| s.italic = on);
    }

    pub fn toggle_underline(&self) {
        self.toggle_style(|s| s.underline, |s, on| s.underline = on);
    }

    pub fn toggle_strikethrough(&self) {
        self.toggle_style(|s| s.strikethrough, |s, on| s.strikethrough = on);
    }

    /// Flips a boolean style attribute. With a selection: turns it on unless every
    /// selected glyph already has it (then off), editor-style. Without a selection:
    /// flips it for newly typed characters.
    fn toggle_style(&self, get: impl Fn(&TextStyle) -> bool, set: impl Fn(&mut TextStyle, bool)) {
        {
            let mut st = self.state.borrow_mut();
            let State { editing, text_style, .. } = &mut *st;
            match editing.as_mut().and_then(|ed| ed.selection().map(|sel| (ed, sel))) {
                Some((ed, (s, e))) => {
                    let target = !ed.glyphs[s..e].iter().all(|g| get(&g.style));
                    for g in &mut ed.glyphs[s..e] {
                        set(&mut g.style, target);
                    }
                    set(text_style, target);
                }
                None => {
                    let target = !get(text_style);
                    set(text_style, target);
                }
            }
        }
        self.area.queue_draw();
    }

    /// Applies the highlight (marker) color to the current selection. The marker is
    /// selection-only, so with nothing selected this does nothing.
    pub fn set_highlight(&self, color: Color) {
        {
            let mut st = self.state.borrow_mut();
            if let Some(ed) = st.editing.as_mut() {
                ed.style_selection(|s| s.highlight = Some(color));
            }
        }
        self.area.queue_draw();
    }

    /// Removes the highlight from the current selection.
    pub fn clear_highlight(&self) {
        {
            let mut st = self.state.borrow_mut();
            if let Some(ed) = st.editing.as_mut() {
                ed.style_selection(|s| s.highlight = None);
            }
        }
        self.area.queue_draw();
    }

    /// Updates the current typing style and applies the same change to the selection.
    fn apply_style(&self, f: impl Fn(&mut TextStyle)) {
        {
            let mut st = self.state.borrow_mut();
            f(&mut st.text_style);
            if let Some(ed) = st.editing.as_mut() {
                ed.style_selection(&f);
            }
        }
        self.area.queue_draw();
    }

    pub fn current_index(&self) -> usize {
        let st = self.state.borrow();
        let count = st.doc.as_ref().map(|d| d.pages.len()).unwrap_or(0);
        st.current.min(count.saturating_sub(1))
    }

    pub fn page_count(&self) -> usize {
        self.state.borrow().doc.as_ref().map(|d| d.pages.len()).unwrap_or(0)
    }

    /// Inserts a blank page before or after the current page.
    pub fn insert_blank_page(&self, rel: Relative) {
        self.cancel_editing();
        self.record_change();
        let current = self.current_index();
        {
            let mut st = self.state.borrow_mut();
            let (w, h) = st
                .doc
                .as_ref()
                .and_then(|d| d.pages.get(current).or_else(|| d.pages.last()))
                .map(|p| (p.width, p.height))
                .unwrap_or(A4);
            let pattern = st.blank_pattern;
            let spacing = st.blank_pattern_spacing;
            let doc = st.doc.get_or_insert_with(Document::new);
            let at = match rel {
                _ if doc.pages.is_empty() => 0,
                Relative::Before => current,
                Relative::After => current + 1,
            };
            doc.insert_blank_page(at, w, h, Color::WHITE, pattern, spacing);
            st.cache.clear();
        }
        self.update_layout();
    }

    /// Removes the current page.
    pub fn delete_current_page(&self) {
        self.remove_page(self.current_index());
    }

    /// Removes the page before or after the current one, if it exists.
    pub fn delete_page(&self, rel: Relative) {
        let current = self.current_index();
        let target = match rel {
            Relative::Before => current.checked_sub(1),
            Relative::After => Some(current + 1),
        };
        if let Some(index) = target {
            self.remove_page(index);
        }
    }

    fn remove_page(&self, index: usize) {
        self.cancel_editing();
        let valid = self.state.borrow().doc.as_ref().is_some_and(|d| index < d.pages.len());
        if !valid {
            return;
        }
        self.record_change();
        {
            let mut st = self.state.borrow_mut();
            if let Some(doc) = st.doc.as_mut() {
                doc.pages.remove(index);
            }
            st.selected = None;
            st.cache.clear();
        }
        self.update_layout();
    }

    pub fn zoom(&self) -> f64 {
        self.state.borrow().zoom
    }

    pub fn set_zoom(&self, zoom: f64) {
        self.commit_editing();
        {
            let mut st = self.state.borrow_mut();
            let new = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
            if (new - st.zoom).abs() < f64::EPSILON {
                return;
            }
            st.zoom = new;
            st.cache.clear();
        }
        self.update_cursor();
        self.update_layout();
    }

    pub fn zoom_in(&self) {
        self.set_zoom(self.zoom() * ZOOM_STEP);
    }

    pub fn zoom_out(&self) {
        self.set_zoom(self.zoom() / ZOOM_STEP);
    }

    /// Size (width, height) in points of the current page.
    pub fn current_page_size(&self) -> Option<(f64, f64)> {
        let idx = self.current_index();
        let st = self.state.borrow();
        st.doc.as_ref()?.pages.get(idx).map(|p| (p.width, p.height))
    }

    /// Whether the current page is a resizable blank page (not a rendered PDF page).
    pub fn current_page_is_blank(&self) -> bool {
        let idx = self.current_index();
        let st = self.state.borrow();
        matches!(
            st.doc.as_ref().and_then(|d| d.pages.get(idx)).map(|p| &p.kind),
            Some(PageKind::Blank { .. })
        )
    }

    /// Ruling used for newly inserted blank pages.
    pub fn blank_pattern(&self) -> PagePattern {
        self.state.borrow().blank_pattern
    }

    /// Spacing (in PDF points) used for newly inserted blank pages' ruling.
    pub fn pattern_spacing(&self) -> f64 {
        self.state.borrow().blank_pattern_spacing
    }

    /// Sets the ruling for newly inserted blank pages; if the current page is
    /// itself blank, restyles it too. Earlier pages are left untouched.
    pub fn set_blank_pattern(&self, pattern: PagePattern) {
        self.update_blank_style(|p, _| *p = pattern);
    }

    /// Sets the ruling spacing for newly inserted blank pages; if the current
    /// page is itself blank, respaces it too. Earlier pages are left untouched.
    pub fn set_pattern_spacing(&self, spacing: f64) {
        self.update_blank_style(|_, s| *s = spacing.max(2.0));
    }

    /// Applies `f` to the blank-page defaults, and — if the current page is
    /// itself blank — to that page's own pattern/spacing too (one undo entry).
    fn update_blank_style(&self, f: impl FnOnce(&mut PagePattern, &mut f64)) {
        let idx = self.current_index();
        let is_blank = self.current_page_is_blank();
        if is_blank {
            self.record_change();
        }
        {
            let mut st = self.state.borrow_mut();
            let state: &mut State = &mut st;
            f(&mut state.blank_pattern, &mut state.blank_pattern_spacing);
            if is_blank {
                let (pattern, spacing) = (st.blank_pattern, st.blank_pattern_spacing);
                if let Some(Page {
                    kind: PageKind::Blank { pattern: p, pattern_spacing: s, .. },
                    ..
                }) = st.doc.as_mut().and_then(|d| d.pages.get_mut(idx))
                {
                    *p = pattern;
                    *s = spacing;
                }
                st.cache.remove(&idx);
            }
        }
        if is_blank {
            self.update_layout();
        }
    }

    /// Applies a pattern/spacing to every blank page in the document (not just
    /// the current one), and makes it the default for future ones too. One
    /// undo entry for the whole change.
    pub fn apply_blank_style_to_all(&self, pattern: PagePattern, spacing: f64) {
        self.record_change();
        {
            let mut st = self.state.borrow_mut();
            let state: &mut State = &mut st;
            state.blank_pattern = pattern;
            state.blank_pattern_spacing = spacing;
            if let Some(doc) = state.doc.as_mut() {
                for page in &mut doc.pages {
                    if let PageKind::Blank { pattern: p, pattern_spacing: s, .. } = &mut page.kind {
                        *p = pattern;
                        *s = spacing;
                    }
                }
            }
            state.cache.clear();
        }
        self.update_layout();
    }

    /// Resizes the current page (blank pages only). One undo entry per call.
    pub fn resize_current_page(&self, width: f64, height: f64) {
        if !self.current_page_is_blank() {
            return;
        }
        let idx = self.current_index();
        self.record_change();
        {
            let mut st = self.state.borrow_mut();
            if let Some(p) = st.doc.as_mut().and_then(|d| d.pages.get_mut(idx)) {
                p.width = width.max(MIN_PAGE);
                p.height = height.max(MIN_PAGE);
            }
            st.cache.remove(&idx);
        }
        self.update_layout();
    }

    /// Resets the current blank page to the nearest real PDF page size (the closest
    /// preceding one, else the closest following one; A4 if there is no PDF page).
    pub fn reset_current_page_size(&self) {
        if !self.current_page_is_blank() {
            return;
        }
        let idx = self.current_index();
        let (w, h) = self.nearest_pdf_page_size(idx);
        self.resize_current_page(w, h);
    }

    fn nearest_pdf_page_size(&self, idx: usize) -> (f64, f64) {
        let st = self.state.borrow();
        let Some(doc) = st.doc.as_ref() else {
            return A4;
        };
        let preceding = (0..idx).rev();
        let following = (idx + 1)..doc.pages.len();
        for i in preceding.chain(following) {
            if let PageKind::Pdf { .. } = doc.pages[i].kind {
                return (doc.pages[i].width, doc.pages[i].height);
            }
        }
        A4
    }

    /// Index of the page whose area contains the viewport's vertical center.
    fn compute_current(&self) -> usize {
        let st = self.state.borrow();
        let Some(doc) = st.doc.as_ref() else {
            return 0;
        };
        if doc.pages.is_empty() {
            return 0;
        }
        let z = st.zoom;
        let adj = self.root.vadjustment();
        let center = adj.value() + adj.page_size() / 2.0;

        let mut top = PAGE_GAP;
        for (i, page) in doc.pages.iter().enumerate() {
            let ph = page.height * z;
            if center < top + ph + PAGE_GAP / 2.0 {
                return i;
            }
            top += ph + PAGE_GAP;
        }
        doc.pages.len() - 1
    }

    fn recompute_current(&self) {
        let index = self.compute_current();
        let changed = {
            let mut st = self.state.borrow_mut();
            let changed = st.current != index;
            st.current = index;
            changed
        };
        if changed {
            self.area.queue_draw();
        }
    }

    /// Widget-space point -> (page index, page-local point in points).
    fn page_hit(&self, x: f64, y: f64) -> Option<(usize, f64, f64)> {
        let st = self.state.borrow();
        let doc = st.doc.as_ref()?;
        hit_test(&doc.pages, st.zoom, self.area.width() as f64, x, y)
    }

    /// Widget-space point -> (page index, annotation index) if it lands on a text box.
    fn annotation_hit(&self, x: f64, y: f64) -> Option<(usize, usize)> {
        let (page, lx, ly) = self.page_hit(x, y)?;
        let st = self.state.borrow();
        let doc = st.doc.as_ref()?;
        let index = annotation_at(doc.pages.get(page)?, lx, ly)?;
        Some((page, index))
    }

    fn on_click(&self, n_press: i32, x: f64, y: f64) {
        self.area.grab_focus();

        // Drawing/erasing tools act via the drag gesture, not clicks.
        if matches!(self.state.borrow().tool, Tool::Pen | Tool::Shape | Tool::Eraser) {
            return;
        }

        // Lasso: a plain click (no real drag - that's handled in on_drag_*)
        // either selects just the one annotation under it, or deselects.
        if self.state.borrow().tool == Tool::Lasso {
            self.commit_editing();
            let hit = self.annotation_hit(x, y);
            let selected = hit.and_then(|(page, index)| self.annotation_id(page, index).map(|id| (page, vec![id])));
            self.state.borrow_mut().lasso_selected = selected;
            self.area.queue_draw();
            return;
        }

        // Clicking inside the box being edited just moves the caret.
        if let Some((page, lx, ly)) = self.page_hit(x, y)
            && self.click_in_editing(page, lx, ly)
        {
            return;
        }

        // Any other click finishes an active edit.
        self.commit_editing();

        let text_mode = self.state.borrow().text_mode;
        let hit = self.annotation_hit(x, y);

        if text_mode {
            if let Some((page, index)) = hit {
                self.start_edit_existing(page, index);
            } else if let Some((page, lx, ly)) = self.page_hit(x, y) {
                self.start_new_text(page, lx, ly);
            }
        } else if n_press == 2 {
            if let Some((page, index)) = hit {
                self.start_edit_existing(page, index);
            }
        } else {
            let selected = hit.and_then(|(page, index)| self.annotation_id(page, index).map(|id| (page, id)));
            self.set_selected(selected);
        }
    }

    fn set_selected(&self, selected: Option<(usize, Uuid)>) {
        self.state.borrow_mut().selected = selected;
        self.area.queue_draw();
    }

    fn annotation_id(&self, page: usize, index: usize) -> Option<Uuid> {
        let st = self.state.borrow();
        st.doc.as_ref()?.pages.get(page)?.annotations.get(index).map(|a| a.id)
    }

    fn delete_selected(&self) {
        let selected = self.state.borrow().selected;
        let Some((page, id)) = selected else {
            return;
        };
        self.record_change();
        {
            let mut st = self.state.borrow_mut();
            if let Some(p) = st.doc.as_mut().and_then(|d| d.pages.get_mut(page)) {
                p.annotations.retain(|a| a.id != id);
            }
            st.selected = None;
            st.cache.remove(&page);
        }
        self.area.queue_draw();
    }

    /// Whether a box is being edited and (page, lx, ly) lands inside it. The caret
    /// itself is placed by the drag gesture's `drag_begin` (which fires on press,
    /// before this click handler), so this only reports the hit and must not move
    /// the caret - doing so would clear the selection anchor that `drag_begin` set.
    fn click_in_editing(&self, page: usize, lx: f64, ly: f64) -> bool {
        let st = self.state.borrow();
        let Some(ed) = st.editing.as_ref() else {
            return false;
        };
        if ed.page != page {
            return false;
        }
        let (w, h) = measure_glyphs(ed.size, &ed.glyphs);
        lx >= ed.x && lx <= ed.x + w && ly >= ed.y && ly <= ed.y + h
    }

    fn start_new_text(&self, page: usize, lx: f64, ly: f64) {
        self.begin_edit_session();
        {
            let mut st = self.state.borrow_mut();
            st.selected = None;
            let size = st.text_size;
            // The I-beam cursor is centered on the click, so center the text line on
            // it too (place the box top half a line up) — the text then sits inside
            // the caret stroke instead of below it.
            let y = (ly - text_line_height(size) / 2.0).max(0.0);
            st.editing = Some(TextEdit {
                page,
                x: lx,
                y,
                size,
                glyphs: Vec::new(),
                cursor: 0,
                anchor: None,
                id: Uuid::new_v4(),
                original: None,
            });
        }
        self.area.queue_draw();
    }

    /// Lifts an existing annotation into the editor (removing it from the model).
    fn start_edit_existing(&self, page: usize, index: usize) {
        self.begin_edit_session();
        let removed = {
            let mut st = self.state.borrow_mut();
            match st.doc.as_mut() {
                Some(doc) if page < doc.pages.len() && index < doc.pages[page].annotations.len() => {
                    Some(doc.pages[page].annotations.remove(index))
                }
                _ => None,
            }
        };
        let Some(annotation) = removed else {
            return;
        };
        let (x, y, size, glyphs, id) = match &annotation.kind {
            AnnotationKind::Text(t) => (t.x, t.y, t.size, ann_glyphs(t), annotation.id),
            // Only text is editable; restore anything else and bail.
            _ => {
                let mut st = self.state.borrow_mut();
                if let Some(p) = st.doc.as_mut().and_then(|d| d.pages.get_mut(page)) {
                    p.annotations.insert(index.min(p.annotations.len()), annotation);
                }
                return;
            }
        };
        {
            let mut st = self.state.borrow_mut();
            let cursor = glyphs.len();
            st.selected = None;
            st.editing = Some(TextEdit {
                page,
                x,
                y,
                size,
                glyphs,
                cursor,
                anchor: None,
                id,
                original: Some(annotation),
            });
            st.cache.remove(&page);
        }
        self.area.grab_focus();
        self.area.queue_draw();
    }

    /// Mutates the active edit (if any) and requests a redraw.
    fn edit_mut(&self, f: impl FnOnce(&mut TextEdit)) {
        if let Some(ed) = self.state.borrow_mut().editing.as_mut() {
            f(ed);
        }
        self.area.queue_draw();
    }

    fn on_key(&self, keyval: gdk::Key, state: gdk::ModifierType) -> glib::Propagation {
        // Undo/redo work in every mode (any active edit is committed first).
        if state.contains(gdk::ModifierType::CONTROL_MASK) {
            let shift = state.contains(gdk::ModifierType::SHIFT_MASK);
            match keyval {
                gdk::Key::z | gdk::Key::Z if shift => {
                    self.redo();
                    return glib::Propagation::Stop;
                }
                gdk::Key::z | gdk::Key::Z => {
                    self.undo();
                    return glib::Propagation::Stop;
                }
                gdk::Key::y | gdk::Key::Y => {
                    self.redo();
                    return glib::Propagation::Stop;
                }
                gdk::Key::plus | gdk::Key::KP_Add | gdk::Key::equal => {
                    self.zoom_in();
                    return glib::Propagation::Stop;
                }
                gdk::Key::minus | gdk::Key::KP_Subtract => {
                    self.zoom_out();
                    return glib::Propagation::Stop;
                }
                _ => {}
            }
        }

        if self.state.borrow().editing.is_none() {
            // Not editing: Delete/Backspace removes the Lasso group, or else the
            // single selected box.
            if matches!(keyval, gdk::Key::Delete | gdk::Key::KP_Delete | gdk::Key::BackSpace) {
                if self.state.borrow().lasso_selected.is_some() {
                    self.delete_lasso_selected();
                    return glib::Propagation::Stop;
                }
                if self.state.borrow().selected.is_some() {
                    self.delete_selected();
                    return glib::Propagation::Stop;
                }
            }
            return glib::Propagation::Proceed;
        }

        let extend = state.contains(gdk::ModifierType::SHIFT_MASK);
        let ctrl = state.contains(gdk::ModifierType::CONTROL_MASK);

        match keyval {
            gdk::Key::Escape => self.cancel_editing(),
            gdk::Key::Return | gdk::Key::KP_Enter => {
                if ctrl {
                    self.commit_editing();
                } else {
                    let style = self.state.borrow().text_style.clone();
                    self.edit_mut(move |ed| ed.insert('\n', style));
                }
            }
            gdk::Key::BackSpace => self.edit_mut(TextEdit::backspace),
            gdk::Key::Delete | gdk::Key::KP_Delete => self.edit_mut(TextEdit::delete_forward),
            gdk::Key::Left | gdk::Key::KP_Left => {
                self.edit_mut(move |ed| if ctrl { ed.move_word_left(extend) } else { ed.move_left(extend) })
            }
            gdk::Key::Right | gdk::Key::KP_Right => {
                self.edit_mut(move |ed| if ctrl { ed.move_word_right(extend) } else { ed.move_right(extend) })
            }
            gdk::Key::Up | gdk::Key::KP_Up => self.edit_mut(move |ed| ed.move_up(extend)),
            gdk::Key::Down | gdk::Key::KP_Down => self.edit_mut(move |ed| ed.move_down(extend)),
            gdk::Key::Home | gdk::Key::KP_Home => self.edit_mut(move |ed| ed.move_home(extend)),
            gdk::Key::End | gdk::Key::KP_End => self.edit_mut(move |ed| ed.move_end(extend)),
            gdk::Key::a | gdk::Key::A if ctrl => self.edit_mut(TextEdit::select_all),
            _ => match keyval.to_unicode() {
                Some(ch) if !ch.is_control() => {
                    let style = self.state.borrow().text_style.clone();
                    self.edit_mut(move |ed| ed.insert(ch, style));
                }
                _ => return glib::Propagation::Proceed,
            },
        }
        glib::Propagation::Stop
    }

    fn commit_editing(&self) {
        let editing = self.state.borrow_mut().editing.take();
        let Some(ed) = editing else {
            return;
        };
        {
            let mut st = self.state.borrow_mut();
            // Non-empty text is (re)added; clearing an existing box deletes it.
            if !ed.is_blank()
                && let Some(page) = st.doc.as_mut().and_then(|d| d.pages.get_mut(ed.page))
            {
                page.annotations.push(Annotation {
                    id: ed.id,
                    kind: AnnotationKind::Text(TextAnnotation {
                        x: ed.x,
                        y: ed.y,
                        size: ed.size,
                        runs: ed.to_runs(),
                    }),
                });
            }
            st.cache.remove(&ed.page);
            // Record the session as one undo entry, but only if it changed the pages.
            if let Some(baseline) = st.edit_baseline.take()
                && st.doc.as_ref().is_some_and(|d| d.pages != baseline)
            {
                st.history.record(baseline);
            }
        }
        self.area.queue_draw();
    }

    fn cancel_editing(&self) {
        self.state.borrow_mut().edit_baseline = None;
        let editing = self.state.borrow_mut().editing.take();
        if let Some(ed) = editing {
            if let Some(original) = ed.original {
                let mut st = self.state.borrow_mut();
                if let Some(page) = st.doc.as_mut().and_then(|d| d.pages.get_mut(ed.page)) {
                    page.annotations.push(original);
                }
                st.cache.remove(&ed.page);
            }
            self.area.queue_draw();
        }
    }

    fn on_drag_begin(&self, x: f64, y: f64) {
        self.state.borrow_mut().drag_start = None;

        // Drawing/erasing tools start their gesture here.
        let tool = self.state.borrow().tool;
        if matches!(tool, Tool::Pen | Tool::Shape | Tool::Eraser) {
            self.begin_draw(tool, x, y);
            return;
        }

        if tool == Tool::Lasso {
            self.begin_lasso(x, y);
            return;
        }

        // A drag inside the box being edited selects text. This applies whenever
        // an edit is active, not just with the Text tool - editing can also start
        // from a Select-mode double-click, which never sets `text_mode`.
        if self.state.borrow().editing.is_some() {
            if let Some((page, lx, ly)) = self.page_hit(x, y) {
                let pos = {
                    let st = self.state.borrow();
                    match st.editing.as_ref() {
                        Some(ed) if ed.page == page => {
                            let (w, h) = measure_glyphs(ed.size, &ed.glyphs);
                            if lx >= ed.x && lx <= ed.x + w && ly >= ed.y && ly <= ed.y + h {
                                Some(cursor_at(ed.size, &ed.glyphs, lx - ed.x, ly - ed.y))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                };
                if let Some(pos) = pos {
                    let mut st = self.state.borrow_mut();
                    if let Some(ed) = st.editing.as_mut() {
                        ed.cursor = pos;
                        ed.anchor = Some(pos);
                    }
                    st.text_drag = Some((x, y));
                    drop(st);
                    self.area.queue_draw();
                }
            }
            return;
        }

        if self.state.borrow().text_mode {
            // Text tool active but nothing is being edited yet (about to place a
            // new box on click) - there is no box to drag.
            return;
        }

        // Otherwise a drag on any annotation (text, stroke, or shape) grabs it for moving.
        if let Some((page, index)) = self.annotation_hit(x, y) {
            let id = {
                let st = self.state.borrow();
                st.doc
                    .as_ref()
                    .and_then(|d| d.pages.get(page))
                    .and_then(|p| p.annotations.get(index))
                    .map(|a| a.id)
            };
            if let Some(id) = id {
                let mut st = self.state.borrow_mut();
                st.drag_start = Some((page, index));
                st.selected = Some((page, id));
                drop(st);
                self.area.queue_draw();
            }
        }
    }

    fn on_drag_update(&self, offset_x: f64, offset_y: f64) {
        // Drawing/erasing: the gesture owns the pointer while draw_op is active.
        if self.state.borrow().draw_op.is_some() {
            self.update_draw(offset_x, offset_y);
            return;
        }

        // Lasso: marquee-select or group-move owns the pointer while active.
        if self.state.borrow().lasso_start.is_some() || self.state.borrow().lasso_op.is_some() {
            self.update_lasso(offset_x, offset_y);
            return;
        }

        // Text drag-select: extend the selection to the current point.
        let text_drag = self.state.borrow().text_drag;
        if let Some((sx, sy)) = text_drag {
            if let Some((page, lx, ly)) = self.page_hit(sx + offset_x, sy + offset_y) {
                let mut st = self.state.borrow_mut();
                if let Some(ed) = st.editing.as_mut()
                    && ed.page == page
                {
                    let pos = cursor_at(ed.size, &ed.glyphs, lx - ed.x, ly - ed.y);
                    ed.cursor = pos;
                }
                drop(st);
                self.area.queue_draw();
            }
            return;
        }

        let zoom = self.state.borrow().zoom;
        let should_lift = {
            let st = self.state.borrow();
            st.dragging.is_none()
                && st.drag_start.is_some()
                && offset_x * offset_x + offset_y * offset_y > DRAG_THRESHOLD_SQ
        };
        if should_lift {
            self.lift_for_drag();
        }

        let mut st = self.state.borrow_mut();
        let State { doc, dragging, .. } = &mut *st;
        if let Some(ds) = dragging {
            let (pw, ph) = doc
                .as_ref()
                .and_then(|d| d.pages.get(ds.page))
                .map(|p| (p.width, p.height))
                .unwrap_or(A4);
            // Keep the whole box on the page so it can't slip behind it. Always
            // translates from `orig` (the pre-drag snapshot) so repeated clamping
            // never accumulates drift.
            let (bx, by, bw, bh) = annotation_bounds(&ds.orig);
            let (dx, dy) = clamp_translate(bx, by, bw, bh, offset_x / zoom, offset_y / zoom, pw, ph);
            ds.annotation.kind = translate_annotation(&ds.orig, dx, dy);
            drop(st);
            self.area.queue_draw();
        }
    }

    fn lift_for_drag(&self) {
        let mut st = self.state.borrow_mut();
        let Some((page, index)) = st.drag_start else {
            return;
        };
        // Snapshot before lifting so the whole move is one undo entry.
        let snapshot = st.doc.as_ref().map(|d| d.pages.clone());
        let removed = match st.doc.as_mut() {
            Some(doc) if page < doc.pages.len() && index < doc.pages[page].annotations.len() => {
                Some(doc.pages[page].annotations.remove(index))
            }
            _ => None,
        };
        if let Some(annotation) = removed {
            if let Some(snapshot) = snapshot {
                st.history.record(snapshot);
            }
            st.cache.remove(&page);
            let orig = annotation.kind.clone();
            st.dragging = Some(DragState { page, annotation, orig });
        }
        st.drag_start = None;
    }

    /// Records where a Lasso-tool press landed; the actual marquee/group-move
    /// gesture is only decided once the drag exceeds the threshold (see
    /// `update_lasso`), so a plain click can still fall through to `on_click`.
    fn begin_lasso(&self, x: f64, y: f64) {
        let hit = self.page_hit(x, y);
        let mut st = self.state.borrow_mut();
        st.lasso_op = None;
        st.lasso_press = Some((x, y));
        st.lasso_start = hit;
    }

    /// Promotes a pending Lasso press into either a group move (press landed on
    /// an already-selected annotation) or a fresh marquee rectangle.
    fn start_lasso_op(&self) {
        let Some((page, lx, ly)) = self.state.borrow().lasso_start else {
            return;
        };
        let hit_id = {
            let st = self.state.borrow();
            st.doc
                .as_ref()
                .and_then(|d| d.pages.get(page))
                .and_then(|p| annotation_at(p, lx, ly).map(|idx| p.annotations[idx].id))
        };
        let in_selection = hit_id.is_some_and(|id| {
            self.state.borrow().lasso_selected.as_ref().is_some_and(|(p, ids)| *p == page && ids.contains(&id))
        });
        if in_selection {
            self.lift_lasso_group(page);
        } else {
            let mut st = self.state.borrow_mut();
            st.lasso_selected = None;
            st.lasso_op = Some(LassoOp::Marquee { page, start: (lx, ly), current: (lx, ly) });
        }
    }

    /// Lifts every currently lasso-selected annotation on `page` out of the
    /// model into a `LassoOp::Move`, so the group can be dragged as one unit.
    fn lift_lasso_group(&self, page: usize) {
        let mut st = self.state.borrow_mut();
        let Some((sel_page, ids)) = st.lasso_selected.clone() else {
            return;
        };
        if sel_page != page {
            return;
        }
        // Snapshot before lifting so the whole move is one undo entry.
        let snapshot = st.doc.as_ref().map(|d| d.pages.clone());
        let Some(p) = st.doc.as_mut().and_then(|d| d.pages.get_mut(page)) else {
            return;
        };
        let mut lifted = Vec::new();
        let mut orig = Vec::new();
        p.annotations.retain(|a| {
            if ids.contains(&a.id) {
                orig.push(a.kind.clone());
                lifted.push(a.clone());
                false
            } else {
                true
            }
        });
        if lifted.is_empty() {
            return;
        }
        if let Some(snapshot) = snapshot {
            st.history.record(snapshot);
        }
        st.cache.remove(&page);
        st.lasso_op = Some(LassoOp::Move { page, orig, lifted });
    }

    fn update_lasso(&self, offset_x: f64, offset_y: f64) {
        let should_start = {
            let st = self.state.borrow();
            st.lasso_op.is_none()
                && st.lasso_start.is_some()
                && offset_x * offset_x + offset_y * offset_y > DRAG_THRESHOLD_SQ
        };
        if should_start {
            self.start_lasso_op();
        }

        let Some((px, py)) = self.state.borrow().lasso_press else {
            return;
        };
        let current_hit = self.page_hit(px + offset_x, py + offset_y);
        let zoom = self.state.borrow().zoom;

        let mut st = self.state.borrow_mut();
        let State { doc, lasso_op, .. } = &mut *st;
        match lasso_op {
            Some(LassoOp::Marquee { page, current, .. }) => {
                if let Some((hit_page, lx, ly)) = current_hit
                    && hit_page == *page
                {
                    *current = (lx, ly);
                }
            }
            Some(LassoOp::Move { page, orig, lifted }) => {
                let (pw, ph) =
                    doc.as_ref().and_then(|d| d.pages.get(*page)).map(|p| (p.width, p.height)).unwrap_or(A4);
                let (bx, by, bw, bh) = union_bounds(orig);
                let (dx, dy) = clamp_translate(bx, by, bw, bh, offset_x / zoom, offset_y / zoom, pw, ph);
                for (annotation, orig_kind) in lifted.iter_mut().zip(orig.iter()) {
                    annotation.kind = translate_annotation(orig_kind, dx, dy);
                }
            }
            None => {}
        }
        drop(st);
        self.area.queue_draw();
    }

    fn end_lasso(&self) {
        let op = {
            let mut st = self.state.borrow_mut();
            st.lasso_start = None;
            st.lasso_press = None;
            st.lasso_op.take()
        };
        match op {
            Some(LassoOp::Move { page, lifted, .. }) => {
                let mut st = self.state.borrow_mut();
                let ids: Vec<Uuid> = lifted.iter().map(|a| a.id).collect();
                if let Some(p) = st.doc.as_mut().and_then(|d| d.pages.get_mut(page)) {
                    p.annotations.extend(lifted);
                }
                st.lasso_selected = Some((page, ids));
                st.cache.remove(&page);
                drop(st);
                self.area.queue_draw();
            }
            Some(LassoOp::Marquee { page, start, current }) => {
                let (x0, y0) = start;
                let (x1, y1) = current;
                let rect = (x0.min(x1), y0.min(y1), (x1 - x0).abs(), (y1 - y0).abs());
                let mut st = self.state.borrow_mut();
                let ids: Vec<Uuid> = st
                    .doc
                    .as_ref()
                    .and_then(|d| d.pages.get(page))
                    .map(|p| {
                        p.annotations
                            .iter()
                            .filter(|a| rects_intersect(annotation_bounds(&a.kind), rect))
                            .map(|a| a.id)
                            .collect()
                    })
                    .unwrap_or_default();
                st.lasso_selected = if ids.is_empty() { None } else { Some((page, ids)) };
                drop(st);
                self.area.queue_draw();
            }
            None => {}
        }
    }

    /// Removes every lasso-selected annotation (one undo entry).
    fn delete_lasso_selected(&self) {
        let Some((page, ids)) = self.state.borrow().lasso_selected.clone() else {
            return;
        };
        self.record_change();
        {
            let mut st = self.state.borrow_mut();
            if let Some(p) = st.doc.as_mut().and_then(|d| d.pages.get_mut(page)) {
                p.annotations.retain(|a| !ids.contains(&a.id));
            }
            st.lasso_selected = None;
            st.cache.remove(&page);
        }
        self.area.queue_draw();
    }

    /// Applies `f` to every lasso-selected annotation whose kind matches
    /// `applies_to` (one undo entry if anything actually applies).
    fn apply_to_lasso_if(&self, applies_to: fn(&AnnotationKind) -> bool, f: impl Fn(&mut AnnotationKind)) {
        let Some((page, ids)) = self.state.borrow().lasso_selected.clone() else {
            return;
        };
        let any_applies = self
            .state
            .borrow()
            .doc
            .as_ref()
            .and_then(|d| d.pages.get(page))
            .map(|p| p.annotations.iter().any(|a| ids.contains(&a.id) && applies_to(&a.kind)))
            .unwrap_or(false);
        if !any_applies {
            return;
        }
        self.record_change();
        let mut st = self.state.borrow_mut();
        if let Some(p) = st.doc.as_mut().and_then(|d| d.pages.get_mut(page)) {
            for annotation in p.annotations.iter_mut().filter(|a| ids.contains(&a.id)) {
                if applies_to(&annotation.kind) {
                    f(&mut annotation.kind);
                }
            }
        }
        st.cache.remove(&page);
        drop(st);
        self.area.queue_draw();
    }

    /// Bulk-recolors the Stroke/Shape members of the current Lasso selection.
    pub fn set_lasso_color(&self, color: Color) {
        self.apply_to_lasso_if(
            |k| matches!(k, AnnotationKind::Stroke(_) | AnnotationKind::Shape(_)),
            move |k| match k {
                AnnotationKind::Stroke(s) => s.color = color,
                AnnotationKind::Shape(s) => s.color = color,
                AnnotationKind::Text(_) => {}
            },
        );
    }

    /// Bulk-resizes the Stroke/Shape members of the current Lasso selection.
    pub fn set_lasso_width(&self, width: f64) {
        self.apply_to_lasso_if(
            |k| matches!(k, AnnotationKind::Stroke(_) | AnnotationKind::Shape(_)),
            move |k| match k {
                AnnotationKind::Stroke(s) => s.width = width,
                AnnotationKind::Shape(s) => s.width = width,
                AnnotationKind::Text(_) => {}
            },
        );
    }

    /// Bulk-changes the shape kind of the Shape members of the current Lasso selection.
    pub fn set_lasso_shape_kind(&self, kind: ShapeKind) {
        self.apply_to_lasso_if(
            |k| matches!(k, AnnotationKind::Shape(_)),
            move |k| {
                if let AnnotationKind::Shape(s) = k {
                    s.shape = kind;
                }
            },
        );
    }

    fn on_drag_end(&self) {
        if self.state.borrow().draw_op.is_some() {
            self.finish_draw();
            return;
        }

        if self.state.borrow().lasso_start.is_some() || self.state.borrow().lasso_op.is_some() {
            self.end_lasso();
            return;
        }

        let (dragging, was_text_drag) = {
            let mut st = self.state.borrow_mut();
            st.drag_start = None;
            let text_drag = st.text_drag.take().is_some();
            (st.dragging.take(), text_drag)
        };
        if was_text_drag {
            return;
        }
        if let Some(ds) = dragging {
            let page_index = ds.page;
            let id = ds.annotation.id;
            let mut st = self.state.borrow_mut();
            if let Some(page) = st.doc.as_mut().and_then(|d| d.pages.get_mut(page_index)) {
                page.annotations.push(ds.annotation);
            }
            st.selected = Some((page_index, id));
            st.cache.remove(&page_index);
            drop(st);
            self.area.queue_draw();
        }
    }

    /// Width/height in points of a page.
    fn page_size(&self, page: usize) -> Option<(f64, f64)> {
        let st = self.state.borrow();
        st.doc.as_ref()?.pages.get(page).map(|p| (p.width, p.height))
    }

    /// Maps a widget-space point to page-local points for a specific page (unclamped).
    fn page_local(&self, page: usize, wx: f64, wy: f64) -> Option<(f64, f64)> {
        let st = self.state.borrow();
        let doc = st.doc.as_ref()?;
        let z = st.zoom;
        let width = self.area.width() as f64;
        let mut top = PAGE_GAP;
        for (i, p) in doc.pages.iter().enumerate() {
            if i == page {
                let left = (width - p.width * z) / 2.0;
                return Some(((wx - left) / z, (wy - top) / z));
            }
            top += p.height * z + PAGE_GAP;
        }
        None
    }

    /// Starts a pen/shape/erase gesture at the pressed point.
    fn begin_draw(&self, tool: Tool, x: f64, y: f64) {
        let Some((page, lx, ly)) = self.page_hit(x, y) else {
            return;
        };
        {
            let mut st = self.state.borrow_mut();
            st.draw_origin = (x, y);
            st.selected = None;
            let op = match tool {
                Tool::Pen => {
                    let mut model = Box::new(PenModel::new((lx, ly)));
                    let points = model.feed(ModelerInputEventType::Down, (lx, ly), 0.0);
                    Draw::Stroke { page, points, color: st.pen_color, width: st.pen_width, model }
                }
                Tool::Shape => Draw::Shape {
                    page,
                    shape: st.shape_kind,
                    start: (lx, ly),
                    end: (lx, ly),
                    color: st.shape_color,
                    width: st.shape_width,
                },
                _ => Draw::Erase {
                    baseline: st.doc.as_ref().map(|d| d.pages.clone()).unwrap_or_default(),
                    changed: false,
                },
            };
            st.draw_op = Some(op);
        }
        if tool == Tool::Eraser {
            self.erase_at(page, lx, ly);
        }
        self.area.queue_draw();
    }

    /// Extends the active pen/shape/erase gesture to the current point.
    fn update_draw(&self, offset_x: f64, offset_y: f64) {
        let (wx, wy) = {
            let st = self.state.borrow();
            (st.draw_origin.0 + offset_x, st.draw_origin.1 + offset_y)
        };
        enum Act {
            Stroke(usize),
            Shape(usize),
            Erase,
        }
        let act = match &self.state.borrow().draw_op {
            Some(Draw::Stroke { page, .. }) => Act::Stroke(*page),
            Some(Draw::Shape { page, .. }) => Act::Shape(*page),
            Some(Draw::Erase { .. }) => Act::Erase,
            None => return,
        };
        match act {
            Act::Stroke(page) => {
                if let Some((lx, ly)) = self.clamped_local(page, wx, wy)
                    && let Some(Draw::Stroke { points, model, .. }) =
                        self.state.borrow_mut().draw_op.as_mut()
                {
                    let time = model.started.elapsed().as_secs_f64();
                    // The modeler requires strictly increasing timestamps.
                    if time > model.last_time {
                        points.extend(model.feed(ModelerInputEventType::Move, (lx, ly), time));
                        model.refresh_prediction();
                    }
                }
            }
            Act::Shape(page) => {
                if let Some((lx, ly)) = self.clamped_local(page, wx, wy)
                    && let Some(Draw::Shape { end, .. }) = self.state.borrow_mut().draw_op.as_mut()
                {
                    *end = (lx, ly);
                }
            }
            Act::Erase => {
                if let Some((page, lx, ly)) = self.page_hit(wx, wy) {
                    self.erase_at(page, lx, ly);
                }
            }
        }
        self.area.queue_draw();
    }

    /// Page-local point clamped to the page bounds (so drawing can't leave the page).
    fn clamped_local(&self, page: usize, wx: f64, wy: f64) -> Option<(f64, f64)> {
        let (lx, ly) = self.page_local(page, wx, wy)?;
        let (pw, ph) = self.page_size(page)?;
        Some((lx.clamp(0.0, pw), ly.clamp(0.0, ph)))
    }

    /// Removes strokes/shapes on `page` within the eraser radius of the point.
    fn erase_at(&self, page: usize, lx: f64, ly: f64) {
        let radius = self.state.borrow().eraser_width / 2.0;
        let mut st = self.state.borrow_mut();
        let State { doc, draw_op, cache, .. } = &mut *st;
        let Some(p) = doc.as_mut().and_then(|d| d.pages.get_mut(page)) else {
            return;
        };
        let before = p.annotations.len();
        p.annotations.retain(|a| !eraser_hits(&a.kind, lx, ly, radius));
        if p.annotations.len() != before {
            cache.remove(&page);
            if let Some(Draw::Erase { changed, .. }) = draw_op {
                *changed = true;
            }
        }
    }

    /// Commits the active pen/shape/erase gesture into the model (one undo entry).
    fn finish_draw(&self) {
        let op = self.state.borrow_mut().draw_op.take();
        self.state.borrow_mut().draw_origin = (0.0, 0.0);
        let Some(op) = op else {
            return;
        };
        match op {
            Draw::Stroke { page, mut points, color, width, mut model } => {
                let time = model.started.elapsed().as_secs_f64().max(model.last_time + 1e-4);
                let pos = model.last_pos;
                points.extend(model.feed(ModelerInputEventType::Up, pos, time));
                if points.is_empty() {
                    return;
                }
                self.record_change();
                self.push_annotation(page, AnnotationKind::Stroke(StrokeAnnotation { points, color, width }));
            }
            Draw::Shape { page, shape, start, end, color, width } => {
                // Ignore accidental zero-size shapes.
                if (start.0 - end.0).abs() < 1.0 && (start.1 - end.1).abs() < 1.0 {
                    return;
                }
                self.record_change();
                self.push_annotation(
                    page,
                    AnnotationKind::Shape(ShapeAnnotation {
                        shape,
                        x0: start.0,
                        y0: start.1,
                        x1: end.0,
                        y1: end.1,
                        color,
                        width,
                    }),
                );
            }
            Draw::Erase { baseline, changed } => {
                if changed {
                    self.state.borrow_mut().history.record(baseline);
                }
            }
        }
        self.area.queue_draw();
    }

    fn push_annotation(&self, page: usize, kind: AnnotationKind) {
        let mut st = self.state.borrow_mut();
        if let Some(p) = st.doc.as_mut().and_then(|d| d.pages.get_mut(page)) {
            p.annotations.push(Annotation { id: Uuid::new_v4(), kind });
        }
        st.cache.remove(&page);
    }

    /// Sets the pointer cursor to match the active tool (sized to the tool where it
    /// makes sense: text caret to font size, eraser ring to eraser size).
    fn update_cursor(&self) {
        let (tool, z, text_size, eraser_width) = {
            let st = self.state.borrow();
            (st.tool, st.zoom, st.text_size, st.eraser_width)
        };
        let cursor = match tool {
            Tool::Text => text_cursor((text_line_height(text_size) * z).round() as i32),
            Tool::Eraser => circle_cursor((eraser_width * z).round() as i32),
            Tool::Pen | Tool::Shape => plus_cursor(),
            Tool::Select | Tool::Lasso | Tool::Markdown | Tool::Pages => None,
        };
        self.area.set_cursor(cursor.as_ref());
    }

    fn update_layout(&self) {
        let (w, h) = {
            let st = self.state.borrow();
            content_size(&st)
        };
        self.area.set_content_width(w as i32);
        self.area.set_content_height(h as i32);
        self.area.queue_draw();
        self.recompute_current();
    }
}

fn content_size(st: &State) -> (f64, f64) {
    let Some(doc) = st.doc.as_ref() else {
        return (0.0, 0.0);
    };
    let z = st.zoom;
    let mut total_h = PAGE_GAP;
    let mut max_w = 0.0_f64;
    for page in &doc.pages {
        total_h += page.height * z + PAGE_GAP;
        max_w = max_w.max(page.width * z);
    }
    (max_w + 2.0 * PAGE_GAP, total_h)
}

/// Maps a widget-space click to a page index and page-local point (in PDF points).
fn hit_test(pages: &[Page], zoom: f64, width: f64, x: f64, y: f64) -> Option<(usize, f64, f64)> {
    let mut top = PAGE_GAP;
    for (i, page) in pages.iter().enumerate() {
        let pw = page.width * zoom;
        let ph = page.height * zoom;
        let left = (width - pw) / 2.0;
        if x >= left && x <= left + pw && y >= top && y <= top + ph {
            return Some((i, (x - left) / zoom, (y - top) / zoom));
        }
        top += ph + PAGE_GAP;
    }
    None
}

/// Topmost text annotation whose box contains the page-local point (in points).
/// Only text is box-selectable; strokes/shapes are edited only via the eraser.
fn annotation_at(page: &Page, lx: f64, ly: f64) -> Option<usize> {
    for (i, annotation) in page.annotations.iter().enumerate().rev() {
        if let AnnotationKind::Text(t) = &annotation.kind {
            let (w, h) = measure_glyphs(t.size, &ann_glyphs(t));
            if lx >= t.x && lx <= t.x + w && ly >= t.y && ly <= t.y + h {
                return Some(i);
            }
        }
    }
    None
}

fn ann_glyphs(t: &TextAnnotation) -> Vec<Glyph> {
    t.glyphs().into_iter().map(|(ch, style)| Glyph { ch, style }).collect()
}

/// Selects a glyph's font family, slant, and weight on the context.
fn apply_glyph_font(c: &cairo::Context, size: f64, style: &TextStyle) {
    let slant = if style.italic { cairo::FontSlant::Italic } else { cairo::FontSlant::Normal };
    let weight = if style.bold { cairo::FontWeight::Bold } else { cairo::FontWeight::Normal };
    c.select_font_face(&style.font, slant, weight);
    c.set_font_size(size);
}

/// Reference (ascent, line height) at `size` from the default font, so every glyph
/// on a line shares one baseline regardless of its family or weight.
fn line_metrics(c: &cairo::Context, size: f64) -> (f64, f64) {
    c.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
    c.set_font_size(size);
    match c.font_extents() {
        Ok(e) => (e.ascent(), e.height().max(1.0)),
        Err(_) => (size * 0.8, size.max(1.0)),
    }
}

/// Per-caret x positions and line indices for a glyph run (index 0..=len).
struct TextLayout {
    positions: Vec<(f64, usize)>,
    line_height: f64,
    max_width: f64,
    line_count: usize,
}

fn layout(c: &cairo::Context, size: f64, glyphs: &[Glyph]) -> TextLayout {
    let (_ascent, line_height) = line_metrics(c, size);
    let mut positions = Vec::with_capacity(glyphs.len() + 1);
    let mut x = 0.0_f64;
    let mut line = 0usize;
    let mut max_width = 0.0_f64;
    positions.push((0.0, 0));
    for g in glyphs {
        if g.ch == '\n' {
            max_width = max_width.max(x);
            line += 1;
            x = 0.0;
        } else {
            apply_glyph_font(c, size, &g.style);
            let adv = c.text_extents(&g.ch.to_string()).map(|e| e.x_advance()).unwrap_or(0.0);
            x += adv;
            max_width = max_width.max(x);
        }
        positions.push((x, line));
    }
    TextLayout { positions, line_height, max_width, line_count: line + 1 }
}

fn with_scratch<R>(f: impl FnOnce(&cairo::Context) -> R, fallback: R) -> R {
    let Ok(surface) = cairo::ImageSurface::create(cairo::Format::ARgb32, 1, 1) else {
        return fallback;
    };
    let Ok(c) = cairo::Context::new(&surface) else {
        return fallback;
    };
    f(&c)
}

/// Height of one text line (in points) for a given font size.
fn text_line_height(size: f64) -> f64 {
    with_scratch(|c| line_metrics(c, size).1, size.max(1.0))
}

/// Box (width, height) in points for a glyph run, honoring newlines.
fn measure_glyphs(size: f64, glyphs: &[Glyph]) -> (f64, f64) {
    with_scratch(
        |c| {
            let l = layout(c, size, glyphs);
            (l.max_width.max(MIN_BOX_WIDTH), l.line_height * l.line_count as f64)
        },
        (MIN_BOX_WIDTH, size),
    )
}

/// Glyph index nearest to a point (dx, dy) relative to the box top-left.
fn cursor_at(size: f64, glyphs: &[Glyph], dx: f64, dy: f64) -> usize {
    with_scratch(
        |c| {
            let l = layout(c, size, glyphs);
            let target = ((dy / l.line_height).floor().max(0.0) as usize).min(l.line_count.saturating_sub(1));
            let mut best = 0usize;
            let mut best_dist = f64::MAX;
            let mut seen = false;
            for (i, &(x, line)) in l.positions.iter().enumerate() {
                if line == target {
                    let dist = (x - dx).abs();
                    if dist < best_dist {
                        best_dist = dist;
                        best = i;
                    }
                    seen = true;
                } else if seen {
                    break;
                }
            }
            best
        },
        glyphs.len(),
    )
}

fn draw(state: &Rc<RefCell<State>>, ctx: &cairo::Context, width: i32) {
    // Theme-aware canvas backdrop: dark grey in dark mode, a light grey in light
    // mode that still reads as distinct from the white page.
    let (r, g, b) = if adw::StyleManager::default().is_dark() {
        CANVAS_BG_DARK
    } else {
        CANVAS_BG_LIGHT
    };
    ctx.set_source_rgb(r, g, b);
    let _ = ctx.paint();

    let mut st = state.borrow_mut();
    let State {
        doc,
        pdf,
        zoom,
        cache,
        text_mode,
        editing,
        current,
        dragging,
        selected,
        draw_op,
        lasso_selected,
        lasso_op,
        ..
    } = &mut *st;
    let Some(doc) = doc.as_ref() else {
        return;
    };
    let z = *zoom;
    let current = *current;
    let overlay = Overlay {
        text_mode: *text_mode,
        editing: editing.as_ref(),
        dragging: dragging.as_ref(),
        selected: *selected,
        drawing: draw_op.as_ref(),
        lasso_selected: lasso_selected.as_ref(),
        lasso_op: lasso_op.as_ref(),
    };

    let (_x0, cy0, _x1, cy1) = ctx.clip_extents().unwrap_or((0.0, 0.0, f64::MAX, f64::MAX));

    let mut y = PAGE_GAP;
    for (i, page) in doc.pages.iter().enumerate() {
        let pw = page.width * z;
        let ph = page.height * z;
        let x = ((width as f64) - pw) / 2.0;

        if y + ph >= cy0
            && y <= cy1
            && let Some(surface) =
                page_surface(pdf.as_ref(), cache, i, page, z, pw.ceil() as i32, ph.ceil() as i32)
        {
            let _ = ctx.set_source_surface(&surface, x, y);
            let _ = ctx.paint();

            ctx.rectangle(x, y, pw, ph);
            if i == current {
                let (r, g, b, a) = BOX_ACTIVE;
                ctx.set_source_rgba(r, g, b, a);
                ctx.set_line_width(1.5);
            } else {
                ctx.set_source_rgba(0.0, 0.0, 0.0, 0.35);
                ctx.set_line_width(1.0);
            }
            let _ = ctx.stroke();

            let _ = ctx.save();
            ctx.translate(x, y);
            ctx.scale(z, z);
            draw_overlay(ctx, page, i, z, &overlay);
            let _ = ctx.restore();
        }

        y += ph + PAGE_GAP;
    }
}

/// Draws a blank page's background ruling in page-point coordinates (the
/// context must already be scaled to zoom). `spacing` is the gap in points
/// between grid lines / dots / ruled lines, user-configurable per page.
pub(crate) fn draw_page_pattern(c: &cairo::Context, pattern: PagePattern, spacing: f64, width: f64, height: f64) {
    match pattern {
        PagePattern::Plain => {}
        PagePattern::Grid => {
            c.set_source_rgba(0.0, 0.0, 0.0, 0.12);
            c.set_line_width(0.5);
            let mut x = spacing;
            while x < width {
                c.move_to(x, 0.0);
                c.line_to(x, height);
                x += spacing;
            }
            let mut y = spacing;
            while y < height {
                c.move_to(0.0, y);
                c.line_to(width, y);
                y += spacing;
            }
            let _ = c.stroke();
        }
        PagePattern::Dotted => {
            c.set_source_rgba(0.0, 0.0, 0.0, 0.35);
            let mut y = spacing;
            while y < height {
                let mut x = spacing;
                while x < width {
                    c.arc(x, y, 0.6, 0.0, std::f64::consts::TAU);
                    let _ = c.fill();
                    x += spacing;
                }
                y += spacing;
            }
        }
        PagePattern::Lined => {
            c.set_source_rgba(0.0, 0.0, 0.0, 0.15);
            c.set_line_width(0.5);
            let line_spacing = spacing * 1.5;
            let mut y = line_spacing;
            while y < height {
                c.move_to(0.0, y);
                c.line_to(width, y);
                y += line_spacing;
            }
            let _ = c.stroke();
        }
    }
}

fn page_surface(
    pdf: Option<&PdfDocument>,
    cache: &mut HashMap<usize, cairo::ImageSurface>,
    index: usize,
    page: &Page,
    zoom: f64,
    pw: i32,
    ph: i32,
) -> Option<cairo::ImageSurface> {
    if let Some(surface) = cache.get(&index) {
        return Some(surface.clone());
    }
    if pw <= 0 || ph <= 0 {
        return None;
    }

    let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, pw, ph).ok()?;
    {
        let c = cairo::Context::new(&surface).ok()?;
        match &page.kind {
            PageKind::Pdf { page_index } => {
                // poppler draws no page background, so lay down white first.
                c.set_source_rgb(1.0, 1.0, 1.0);
                let _ = c.paint();
                c.scale(zoom, zoom);
                if let Some(pdf) = pdf {
                    pdf.render_page(*page_index, &c);
                }
            }
            PageKind::Blank { color, pattern, pattern_spacing } => {
                let Color { r, g, b, a } = *color;
                c.set_source_rgba(r, g, b, a);
                let _ = c.paint();
                c.scale(zoom, zoom);
                draw_page_pattern(&c, *pattern, *pattern_spacing, page.width, page.height);
            }
        }
        for annotation in &page.annotations {
            draw_annotation(&c, &annotation.kind);
        }
    }

    cache.insert(index, surface.clone());
    Some(surface)
}

/// Draws glyphs (each with its own style: color, font, highlight, weight, decoration),
/// honoring newlines. Context in page-point space.
fn draw_glyphs(c: &cairo::Context, x: f64, y: f64, size: f64, glyphs: &[Glyph]) {
    let (ascent, line_height) = line_metrics(c, size);
    let mut gx = x;
    let mut line_top = y;
    for g in glyphs {
        if g.ch == '\n' {
            gx = x;
            line_top += line_height;
            continue;
        }
        apply_glyph_font(c, size, &g.style);
        let adv = c.text_extents(&g.ch.to_string()).map(|e| e.x_advance()).unwrap_or(0.0);

        if let Some(h) = g.style.highlight {
            c.set_source_rgba(h.r, h.g, h.b, h.a);
            c.rectangle(gx, line_top, adv, line_height);
            let _ = c.fill();
        }

        let baseline = line_top + ascent;
        let Color { r, g: gg, b, a } = g.style.color;
        c.set_source_rgba(r, gg, b, a);
        c.move_to(gx, baseline);
        let _ = c.show_text(&g.ch.to_string());

        if g.style.underline || g.style.strikethrough {
            c.set_line_width((size * 0.06).max(0.5));
            if g.style.underline {
                let uy = baseline + size * 0.12;
                c.move_to(gx, uy);
                c.line_to(gx + adv, uy);
                let _ = c.stroke();
            }
            if g.style.strikethrough {
                let sy = baseline - ascent * 0.32;
                c.move_to(gx, sy);
                c.line_to(gx + adv, sy);
                let _ = c.stroke();
            }
        }
        gx += adv;
    }
}

/// Dispatches an annotation to its renderer. Context in page-point space.
fn draw_annotation(c: &cairo::Context, kind: &AnnotationKind) {
    match kind {
        AnnotationKind::Text(t) => draw_glyphs(c, t.x, t.y, t.size, &ann_glyphs(t)),
        AnnotationKind::Stroke(s) => draw_stroke(c, &s.points, s.color, s.width),
        AnnotationKind::Shape(s) => draw_shape(c, s),
    }
}

/// Bounding box (x, y, w, h) of an annotation in page-point space, regardless
/// of kind - used for selection frames and to clamp/anchor moves.
fn annotation_bounds(kind: &AnnotationKind) -> (f64, f64, f64, f64) {
    match kind {
        AnnotationKind::Text(t) => {
            let (w, h) = measure_glyphs(t.size, &ann_glyphs(t));
            (t.x, t.y, w, h)
        }
        AnnotationKind::Stroke(s) => bounds_of_points(&s.points),
        AnnotationKind::Shape(s) => {
            (s.x0.min(s.x1), s.y0.min(s.y1), (s.x1 - s.x0).abs(), (s.y1 - s.y0).abs())
        }
    }
}

fn bounds_of_points(points: &[(f64, f64)]) -> (f64, f64, f64, f64) {
    let Some(&(fx, fy)) = points.first() else {
        return (0.0, 0.0, 0.0, 0.0);
    };
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (fx, fy, fx, fy);
    for &(x, y) in &points[1..] {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    (min_x, min_y, max_x - min_x, max_y - min_y)
}

/// Returns a copy of `kind` translated by `(dx, dy)` in page points.
fn translate_annotation(kind: &AnnotationKind, dx: f64, dy: f64) -> AnnotationKind {
    match kind {
        AnnotationKind::Text(t) => {
            let mut t = t.clone();
            t.x += dx;
            t.y += dy;
            AnnotationKind::Text(t)
        }
        AnnotationKind::Stroke(s) => {
            let mut s = s.clone();
            for p in &mut s.points {
                p.0 += dx;
                p.1 += dy;
            }
            AnnotationKind::Stroke(s)
        }
        AnnotationKind::Shape(s) => {
            let mut s = s.clone();
            s.x0 += dx;
            s.x1 += dx;
            s.y0 += dy;
            s.y1 += dy;
            AnnotationKind::Shape(s)
        }
    }
}

/// Clamps a proposed translation `(dx, dy)` so a box `(bx, by, bw, bh)` stays
/// within `[0, pw] x [0, ph]`.
fn clamp_translate(bx: f64, by: f64, bw: f64, bh: f64, dx: f64, dy: f64, pw: f64, ph: f64) -> (f64, f64) {
    let min_dx = -bx;
    let max_dx = (pw - bw - bx).max(min_dx);
    let min_dy = -by;
    let max_dy = (ph - bh - by).max(min_dy);
    (dx.clamp(min_dx, max_dx), dy.clamp(min_dy, max_dy))
}

/// Combined bounding box (x, y, w, h) of several annotation kinds, e.g. for
/// clamping a Lasso group-move so the whole group stays on the page.
fn union_bounds(kinds: &[AnnotationKind]) -> (f64, f64, f64, f64) {
    let mut iter = kinds.iter().map(annotation_bounds);
    let Some((x0, y0, w0, h0)) = iter.next() else {
        return (0.0, 0.0, 0.0, 0.0);
    };
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (x0, y0, x0 + w0, y0 + h0);
    for (x, y, w, h) in iter {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x + w);
        max_y = max_y.max(y + h);
    }
    (min_x, min_y, max_x - min_x, max_y - min_y)
}

/// Whether two (x, y, w, h) rectangles overlap.
fn rects_intersect(a: (f64, f64, f64, f64), b: (f64, f64, f64, f64)) -> bool {
    let (ax, ay, aw, ah) = a;
    let (bx, by, bw, bh) = b;
    ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by
}

/// Draws a freehand stroke as a Catmull-Rom spline through the sampled points
/// (rendered as cubic Béziers), so it looks smooth instead of polygonal.
/// A single point renders as a dot.
fn draw_stroke(c: &cairo::Context, points: &[(f64, f64)], color: Color, width: f64) {
    if points.is_empty() {
        return;
    }
    c.set_source_rgba(color.r, color.g, color.b, color.a);
    c.set_line_width(width.max(0.1));
    c.set_line_cap(cairo::LineCap::Round);
    c.set_line_join(cairo::LineJoin::Round);
    if points.len() == 1 {
        let (x, y) = points[0];
        c.arc(x, y, (width / 2.0).max(0.1), 0.0, std::f64::consts::TAU);
        let _ = c.fill();
        return;
    }
    c.move_to(points[0].0, points[0].1);
    // Catmull-Rom segment p1→p2 as a cubic Bézier; endpoints are duplicated so
    // the curve passes through the first and last point.
    for i in 0..points.len() - 1 {
        let p0 = points[i.saturating_sub(1)];
        let p1 = points[i];
        let p2 = points[i + 1];
        let p3 = points[(i + 2).min(points.len() - 1)];
        let c1 = (p1.0 + (p2.0 - p0.0) / 6.0, p1.1 + (p2.1 - p0.1) / 6.0);
        let c2 = (p2.0 - (p3.0 - p1.0) / 6.0, p2.1 - (p3.1 - p1.1) / 6.0);
        c.curve_to(c1.0, c1.1, c2.0, c2.1, p2.0, p2.1);
    }
    let _ = c.stroke();
}

/// Draws a rectangle, ellipse, or line outline.
fn draw_shape(c: &cairo::Context, s: &ShapeAnnotation) {
    c.set_source_rgba(s.color.r, s.color.g, s.color.b, s.color.a);
    c.set_line_width(s.width.max(0.1));
    c.set_line_cap(cairo::LineCap::Round);
    c.set_line_join(cairo::LineJoin::Round);
    let (x0, y0, x1, y1) = (s.x0, s.y0, s.x1, s.y1);
    match s.shape {
        ShapeKind::Rectangle => {
            c.rectangle(x0.min(x1), y0.min(y1), (x1 - x0).abs(), (y1 - y0).abs());
        }
        ShapeKind::Ellipse => {
            let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
            let (rx, ry) = ((x1 - x0).abs() / 2.0, (y1 - y0).abs() / 2.0);
            if rx > 0.0 && ry > 0.0 {
                let _ = c.save();
                c.translate(cx, cy);
                c.scale(rx, ry);
                c.arc(0.0, 0.0, 1.0, 0.0, std::f64::consts::TAU);
                let _ = c.restore();
            }
        }
        ShapeKind::Line => {
            c.move_to(x0, y0);
            c.line_to(x1, y1);
        }
    }
    let _ = c.stroke();
}

/// Whether the eraser (a disc of `radius` at px,py) touches this annotation's
/// geometry. Text is ignored - it has its own selection/delete.
fn eraser_hits(kind: &AnnotationKind, px: f64, py: f64, radius: f64) -> bool {
    match kind {
        AnnotationKind::Text(_) => false,
        AnnotationKind::Stroke(s) => {
            point_near_polyline(&s.points, false, px, py, radius + s.width / 2.0)
        }
        AnnotationKind::Shape(s) => {
            let closed = !matches!(s.shape, ShapeKind::Line);
            point_near_polyline(&shape_polyline(s), closed, px, py, radius + s.width / 2.0)
        }
    }
}

/// A shape's outline sampled as a polyline (for hit-testing).
fn shape_polyline(s: &ShapeAnnotation) -> Vec<(f64, f64)> {
    match s.shape {
        ShapeKind::Rectangle => vec![(s.x0, s.y0), (s.x1, s.y0), (s.x1, s.y1), (s.x0, s.y1)],
        ShapeKind::Line => vec![(s.x0, s.y0), (s.x1, s.y1)],
        ShapeKind::Ellipse => {
            let (cx, cy) = ((s.x0 + s.x1) / 2.0, (s.y0 + s.y1) / 2.0);
            let (rx, ry) = ((s.x1 - s.x0).abs() / 2.0, (s.y1 - s.y0).abs() / 2.0);
            (0..24)
                .map(|i| {
                    let a = i as f64 / 24.0 * std::f64::consts::TAU;
                    (cx + rx * a.cos(), cy + ry * a.sin())
                })
                .collect()
        }
    }
}

/// Whether (px,py) is within `thr` of the polyline. `closed` adds the wrap segment.
fn point_near_polyline(points: &[(f64, f64)], closed: bool, px: f64, py: f64, thr: f64) -> bool {
    match points.len() {
        0 => false,
        1 => (points[0].0 - px).hypot(points[0].1 - py) <= thr,
        _ => {
            let n = points.len();
            let last = if closed { n } else { n - 1 };
            (0..last).any(|i| {
                let a = points[i];
                let b = points[(i + 1) % n];
                dist_point_segment(px, py, a, b) <= thr
            })
        }
    }
}

/// Distance from a point to a line segment a-b.
fn dist_point_segment(px: f64, py: f64, a: (f64, f64), b: (f64, f64)) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len_sq = dx * dx + dy * dy;
    if len_sq <= f64::EPSILON {
        return (px - a.0).hypot(py - a.1);
    }
    let t = (((px - a.0) * dx + (py - a.1) * dy) / len_sq).clamp(0.0, 1.0);
    (px - (a.0 + t * dx)).hypot(py - (a.1 + t * dy))
}

/// Builds a cursor from a cairo-drawn ARGB surface. `hotspot` is in pixels.
fn cursor_from_draw(
    w: i32,
    h: i32,
    hotspot: (i32, i32),
    draw: impl FnOnce(&cairo::Context),
) -> Option<gdk::Cursor> {
    let (w, h) = (w.clamp(1, 256), h.clamp(1, 256));
    let mut surface = cairo::ImageSurface::create(cairo::Format::ARgb32, w, h).ok()?;
    {
        let c = cairo::Context::new(&surface).ok()?;
        draw(&c);
    }
    surface.flush();
    let stride = surface.stride() as usize;
    let data = surface.data().ok()?;
    let bytes = glib::Bytes::from(&data[..]);
    let texture =
        gdk::MemoryTexture::new(w, h, gdk::MemoryFormat::B8g8r8a8Premultiplied, &bytes, stride);
    Some(gdk::Cursor::from_texture(&texture, hotspot.0, hotspot.1, None))
}

/// Strokes the current path as a dark core with a light halo, so a cursor stays
/// visible on both light and dark backgrounds.
fn stroke_halo(c: &cairo::Context) {
    c.set_line_cap(cairo::LineCap::Round);
    c.set_source_rgba(1.0, 1.0, 1.0, 0.9);
    c.set_line_width(3.0);
    let _ = c.stroke_preserve();
    c.set_source_rgba(0.0, 0.0, 0.0, 0.95);
    c.set_line_width(1.3);
    let _ = c.stroke();
}

/// Text caret cursor: a vertical I-beam whose height tracks the on-screen font size.
fn text_cursor(height: i32) -> Option<gdk::Cursor> {
    let h = height.clamp(8, 200);
    let w = 9;
    let cx = (w / 2) as f64;
    cursor_from_draw(w, h, (w / 2, h / 2), move |c| {
        let (top, bot) = (1.5, h as f64 - 1.5);
        c.move_to(cx, top);
        c.line_to(cx, bot);
        c.move_to(cx - 3.0, top);
        c.line_to(cx + 3.0, top);
        c.move_to(cx - 3.0, bot);
        c.line_to(cx + 3.0, bot);
        stroke_halo(c);
    })
}

/// Eraser cursor: a ring whose diameter tracks the on-screen eraser size.
fn circle_cursor(diameter: i32) -> Option<gdk::Cursor> {
    let d = diameter.clamp(6, 200);
    let size = d + 4;
    let center = size as f64 / 2.0;
    cursor_from_draw(size, size, (size / 2, size / 2), move |c| {
        c.arc(center, center, d as f64 / 2.0, 0.0, std::f64::consts::TAU);
        stroke_halo(c);
    })
}

/// Pen/shape cursor: a small, fixed-size "+".
fn plus_cursor() -> Option<gdk::Cursor> {
    let size = 17;
    let m = size as f64 / 2.0;
    cursor_from_draw(size, size, (size / 2, size / 2), move |c| {
        c.move_to(2.0, m);
        c.line_to(size as f64 - 2.0, m);
        c.move_to(m, 2.0);
        c.line_to(m, size as f64 - 2.0);
        stroke_halo(c);
    })
}

fn draw_overlay(c: &cairo::Context, page: &Page, index: usize, zoom: f64, overlay: &Overlay) {
    if overlay.text_mode {
        for annotation in &page.annotations {
            if let AnnotationKind::Text(t) = &annotation.kind {
                let (w, h) = measure_glyphs(t.size, &ann_glyphs(t));
                stroke_box(c, t.x, t.y, w, h, zoom, BOX_ANNOTATION);
            }
        }
    }

    if let Some((sel_page, id)) = overlay.selected
        && sel_page == index
        && let Some(annotation) = page.annotations.iter().find(|a| a.id == id)
    {
        let (x, y, w, h) = annotation_bounds(&annotation.kind);
        stroke_selection_handles(c, x, y, w, h, zoom);
    }

    if let Some(ed) = overlay.editing
        && ed.page == index
    {
        let l = layout(c, ed.size, &ed.glyphs);

        // Highlight the selected characters.
        if let Some((s, e)) = ed.selection() {
            let (r, g, b, a) = SELECTION_FILL;
            c.set_source_rgba(r, g, b, a);
            for i in s..e {
                if ed.glyphs[i].ch == '\n' {
                    continue;
                }
                let (x0, line) = l.positions[i];
                let (x1, _) = l.positions[i + 1];
                let top = line as f64 * l.line_height;
                c.rectangle(ed.x + x0, ed.y + top, x1 - x0, l.line_height);
            }
            let _ = c.fill();
        }

        draw_glyphs(c, ed.x, ed.y, ed.size, &ed.glyphs);

        let (bw, bh) = measure_glyphs(ed.size, &ed.glyphs);
        stroke_box(c, ed.x, ed.y, bw, bh, zoom, BOX_ACTIVE);
        draw_caret(c, ed, &l, zoom);
    }

    if let Some(ds) = overlay.dragging
        && ds.page == index
    {
        draw_annotation(c, &ds.annotation.kind);
        let (x, y, w, h) = annotation_bounds(&ds.annotation.kind);
        stroke_box(c, x, y, w, h, zoom, BOX_ACTIVE);
    }

    // Selection handles for every Lasso-selected annotation.
    if let Some((sel_page, ids)) = overlay.lasso_selected
        && *sel_page == index
    {
        for id in ids {
            if let Some(annotation) = page.annotations.iter().find(|a| a.id == *id) {
                let (x, y, w, h) = annotation_bounds(&annotation.kind);
                stroke_selection_handles(c, x, y, w, h, zoom);
            }
        }
    }

    // The in-progress Lasso gesture: a marquee rectangle, or the group being dragged.
    if let Some(op) = overlay.lasso_op {
        match op {
            LassoOp::Marquee { page: op_page, start, current } if *op_page == index => {
                let (x0, y0) = *start;
                let (x1, y1) = *current;
                stroke_box(c, x0.min(x1), y0.min(y1), (x1 - x0).abs(), (y1 - y0).abs(), zoom, BOX_ACTIVE);
            }
            LassoOp::Move { page: op_page, lifted, .. } if *op_page == index => {
                for annotation in lifted {
                    draw_annotation(c, &annotation.kind);
                    let (x, y, w, h) = annotation_bounds(&annotation.kind);
                    stroke_selection_handles(c, x, y, w, h, zoom);
                }
            }
            _ => {}
        }
    }

    // Live preview of the in-progress pen stroke or shape.
    if let Some(draw) = overlay.drawing {
        match draw {
            Draw::Stroke { page, points, color, width, model } if *page == index => {
                if model.prediction.is_empty() {
                    draw_stroke(c, points, *color, *width);
                } else {
                    let mut with_tail = points.clone();
                    with_tail.extend_from_slice(&model.prediction);
                    draw_stroke(c, &with_tail, *color, *width);
                }
            }
            Draw::Shape { page, shape, start, end, color, width } if *page == index => {
                draw_shape(
                    c,
                    &ShapeAnnotation {
                        shape: *shape,
                        x0: start.0,
                        y0: start.1,
                        x1: end.0,
                        y1: end.1,
                        color: *color,
                        width: *width,
                    },
                );
            }
            _ => {}
        }
    }
}

/// Strokes a bounding box (context in page-point space).
fn stroke_box(c: &cairo::Context, x: f64, y: f64, w: f64, h: f64, zoom: f64, rgba: (f64, f64, f64, f64)) {
    let (r, g, b, a) = rgba;
    c.set_source_rgba(r, g, b, a);
    c.set_line_width(1.0 / zoom);
    c.rectangle(x, y, w, h);
    let _ = c.stroke();
}

/// Selection outline with small corner handles (for move/select mode).
fn stroke_selection_handles(c: &cairo::Context, x: f64, y: f64, w: f64, h: f64, zoom: f64) {
    let (r, g, b, _) = BOX_ACTIVE;
    c.set_source_rgba(r, g, b, 1.0);
    c.set_line_width(1.5 / zoom);
    c.rectangle(x, y, w, h);
    let _ = c.stroke();

    let half = 3.0 / zoom;
    for (hx, hy) in [(x, y), (x + w, y), (x, y + h), (x + w, y + h)] {
        c.rectangle(hx - half, hy - half, 2.0 * half, 2.0 * half);
    }
    let _ = c.fill();
}

fn draw_caret(c: &cairo::Context, ed: &TextEdit, l: &TextLayout, zoom: f64) {
    let (x, line) = l.positions.get(ed.cursor).copied().unwrap_or((0.0, 0));
    let cx = ed.x + x;
    let top = ed.y + line as f64 * l.line_height;
    let (r, g, b, a) = BOX_ACTIVE;
    c.set_source_rgba(r, g, b, a);
    c.set_line_width(1.0 / zoom);
    c.move_to(cx, top);
    c.line_to(cx, top + l.line_height);
    let _ = c.stroke();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Zig-zag input along a horizontal line must come out flatter than it went in.
    #[test]
    fn pen_model_smooths_jittery_input() {
        let mut model = PenModel::new((0.0, 0.0));
        let mut points = model.feed(ModelerInputEventType::Down, (0.0, 0.0), 0.0);
        for i in 1..=20 {
            let jitter = if i % 2 == 0 { 0.8 } else { -0.8 };
            points.extend(model.feed(
                ModelerInputEventType::Move,
                (i as f64 * 2.0, jitter),
                i as f64 * 0.01,
            ));
        }
        points.extend(model.feed(ModelerInputEventType::Up, (42.0, 0.0), 0.21));
        assert!(points.len() > 21, "modeler should upsample the input");
        let max_dev = points.iter().map(|p| p.1.abs()).fold(0.0, f64::max);
        assert!(max_dev < 0.5, "deviation {max_dev} should stay well below the 0.8 input jitter");
    }

    fn a4_page() -> Page {
        Page {
            kind: PageKind::Blank {
                color: Color::WHITE,
                pattern: PagePattern::Plain,
                pattern_spacing: DEFAULT_PATTERN_SPACING,
            },
            width: A4.0,
            height: A4.1,
            annotations: vec![],
        }
    }

    fn text_page(x: f64, y: f64, content: &str) -> Page {
        let mut page = a4_page();
        page.annotations.push(Annotation {
            id: Uuid::new_v4(),
            kind: AnnotationKind::Text(TextAnnotation {
                x,
                y,
                size: TEXT_SIZE,
                runs: vec![TextRun { text: content.into(), style: TextStyle::default() }],
            }),
        });
        page
    }

    fn glyphs_of(s: &str) -> Vec<Glyph> {
        s.chars().map(|ch| Glyph { ch, style: TextStyle::default() }).collect()
    }

    fn empty_edit() -> TextEdit {
        TextEdit {
            page: 0,
            x: 0.0,
            y: 0.0,
            size: TEXT_SIZE,
            glyphs: Vec::new(),
            cursor: 0,
            anchor: None,
            id: Uuid::new_v4(),
            original: None,
        }
    }

    fn text_of(ed: &TextEdit) -> String {
        ed.glyphs.iter().map(|g| g.ch).collect()
    }

    #[test]
    fn hit_test_maps_click_to_page_local_point() {
        let pages = vec![a4_page(), a4_page()];
        let width = A4.0 + 2.0 * PAGE_GAP;
        let left = PAGE_GAP;

        let hit = hit_test(&pages, 1.0, width, left + 10.0, PAGE_GAP + 20.0);
        assert_eq!(hit, Some((0, 10.0, 20.0)));

        let second_top = PAGE_GAP + A4.1 + PAGE_GAP;
        let hit2 = hit_test(&pages, 1.0, width, left + 5.0, second_top + 1.0);
        assert_eq!(hit2, Some((1, 5.0, 1.0)));

        assert_eq!(hit_test(&pages, 1.0, width, 2.0, 2.0), None);
    }

    #[test]
    fn annotation_at_hits_inside_and_misses_outside() {
        let page = text_page(100.0, 200.0, "Hello");
        assert_eq!(annotation_at(&page, 101.0, 201.0), Some(0));
        assert_eq!(annotation_at(&page, 10.0, 10.0), None);
    }

    #[test]
    fn text_edit_insert_delete_and_navigate() {
        let mut ed = empty_edit();
        for ch in "abc".chars() {
            ed.insert(ch, TextStyle::default());
        }
        assert_eq!((text_of(&ed), ed.cursor), ("abc".to_string(), 3));

        ed.move_left(false);
        ed.insert('X', TextStyle::default());
        assert_eq!((text_of(&ed), ed.cursor), ("abXc".to_string(), 3));

        ed.backspace();
        assert_eq!((text_of(&ed), ed.cursor), ("abc".to_string(), 2));

        ed.move_home(false);
        assert_eq!(ed.cursor, 0);
        ed.move_end(false);
        assert_eq!(ed.cursor, 3);
        ed.delete_forward(); // at end -> no-op
        assert_eq!(text_of(&ed), "abc");
    }

    #[test]
    fn text_edit_vertical_keeps_column() {
        let mut ed = empty_edit();
        ed.glyphs = glyphs_of("ab\ncd");
        ed.cursor = 5; // end of "cd"
        ed.move_up(false);
        assert_eq!(ed.cursor, 2);
        ed.move_down(false);
        assert_eq!(ed.cursor, 5);
    }

    #[test]
    fn shift_selects_and_color_applies_to_selection_only() {
        let mut ed = empty_edit();
        ed.glyphs = glyphs_of("abcd");
        ed.cursor = 1;
        ed.move_right(true); // select "b"
        ed.move_right(true); // select "bc"
        assert_eq!(ed.selection(), Some((1, 3)));

        let red = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
        assert!(ed.style_selection(|s| s.color = red));
        assert_eq!(ed.glyphs[0].style.color, Color::BLACK);
        assert_eq!(ed.glyphs[1].style.color, red);
        assert_eq!(ed.glyphs[2].style.color, red);
        assert_eq!(ed.glyphs[3].style.color, Color::BLACK);

        // Runs merge same-style neighbors.
        let runs = ed.to_runs();
        assert_eq!(runs.len(), 3);
        assert_eq!(runs[1].text, "bc");
    }

    #[test]
    fn ctrl_word_navigation_jumps_by_word() {
        let mut ed = empty_edit();
        ed.glyphs = glyphs_of("foo bar baz");
        ed.cursor = ed.glyphs.len();
        ed.move_word_left(false);
        assert_eq!(ed.cursor, 8); // start of "baz"
        ed.move_word_left(false);
        assert_eq!(ed.cursor, 4); // start of "bar"
        ed.move_word_right(false);
        assert_eq!(ed.cursor, 7); // end of "bar"
    }

    #[test]
    fn marker_and_bold_apply_to_selection() {
        let mut ed = empty_edit();
        ed.glyphs = glyphs_of("abcd");
        ed.anchor = Some(1);
        ed.cursor = 3; // "bc" selected

        let yellow = Color { r: 1.0, g: 0.9, b: 0.2, a: 0.4 };
        assert!(ed.style_selection(|s| s.highlight = Some(yellow)));
        assert!(ed.style_selection(|s| s.bold = true));
        assert_eq!(ed.glyphs[0].style.highlight, None);
        assert_eq!(ed.glyphs[1].style.highlight, Some(yellow));
        assert!(ed.glyphs[2].style.bold);
        assert!(!ed.glyphs[3].style.bold);
    }

    #[test]
    fn typing_replaces_selection() {
        let mut ed = empty_edit();
        ed.glyphs = glyphs_of("abcd");
        ed.anchor = Some(1);
        ed.cursor = 3; // "bc" selected
        ed.insert('X', TextStyle::default());
        assert_eq!(text_of(&ed), "aXd");
        assert_eq!(ed.cursor, 2);
    }

    #[test]
    fn text_annotation_renders_pixels() {
        let page = text_page(50.0, 50.0, "Hello");
        let mut cache = HashMap::new();
        let (pw, ph) = (A4.0.ceil() as i32, A4.1.ceil() as i32);
        let mut surface = page_surface(None, &mut cache, 0, &page, 1.0, pw, ph).unwrap();
        drop(cache);
        surface.flush();

        let data = surface.data().unwrap();
        let non_white = data.iter().filter(|&&b| b != 0xFF).count();
        assert!(non_white > 0, "text should render as dark pixels on the white page");
    }

    #[test]
    fn measure_grows_with_newlines() {
        let (_, one) = measure_glyphs(TEXT_SIZE, &glyphs_of("single line"));
        let (_, three) = measure_glyphs(TEXT_SIZE, &glyphs_of("line\ntwo\nthree"));
        assert!(three > one);
    }

    #[test]
    fn history_record_clears_redo_and_caps() {
        let mut h = History::default();
        h.redo.push(vec![a4_page()]); // stale redo
        h.record(vec![a4_page()]);
        assert_eq!(h.undo.len(), 1);
        assert!(h.redo.is_empty(), "recording a new change must invalidate redo");

        for _ in 0..(HISTORY_LIMIT + 10) {
            h.record(vec![a4_page()]);
        }
        assert_eq!(h.undo.len(), HISTORY_LIMIT, "undo stack is capped");
    }

    #[test]
    fn history_step_round_trips() {
        let mut h = History::default();
        // One recorded change: pages went from empty -> [page].
        h.record(vec![]);
        let current = vec![a4_page()];

        // Undo: restore empty, current pushed to redo.
        let undone = h.step(current.clone(), false).unwrap();
        assert!(undone.is_empty());
        assert!(h.undo.is_empty());
        assert_eq!(h.redo.len(), 1);

        // Redo: restore [page].
        let redone = h.step(undone, true).unwrap();
        assert_eq!(redone.len(), 1);
        assert_eq!(h.undo.len(), 1);
        assert!(h.redo.is_empty());

        // Nothing left to redo.
        assert!(h.step(redone, true).is_none());
    }

    #[test]
    fn highlighted_text_renders_marker_pixels() {
        let mut page = a4_page();
        page.annotations.push(Annotation {
            id: Uuid::new_v4(),
            kind: AnnotationKind::Text(TextAnnotation {
                x: 50.0,
                y: 50.0,
                size: 32.0,
                runs: vec![TextRun {
                    text: "Hi".into(),
                    style: TextStyle {
                        highlight: Some(Color { r: 1.0, g: 0.9, b: 0.2, a: 1.0 }),
                        ..Default::default()
                    },
                }],
            }),
        });
        let mut cache = HashMap::new();
        let (pw, ph) = (A4.0.ceil() as i32, A4.1.ceil() as i32);
        let mut surface = page_surface(None, &mut cache, 0, &page, 1.0, pw, ph).unwrap();
        drop(cache);
        surface.flush();

        // ARgb32 is BGRA in memory; look for a pixel that is strongly yellow
        // (high red + green, low blue) — the marker fill behind the glyphs.
        let stride = surface.stride() as usize;
        let data = surface.data().unwrap();
        let mut found = false;
        for row in data.chunks_exact(stride) {
            for px in row.chunks_exact(4) {
                let (b, g, r) = (px[0], px[1], px[2]);
                if r > 200 && g > 180 && b < 120 {
                    found = true;
                }
            }
        }
        assert!(found, "marker highlight should paint yellow pixels");
    }

    #[test]
    fn dist_point_segment_cases() {
        let (a, b) = ((0.0, 0.0), (2.0, 0.0));
        assert!(dist_point_segment(1.0, 0.0, a, b) < 1e-9); // on segment
        assert!((dist_point_segment(1.0, 3.0, a, b) - 3.0).abs() < 1e-9); // perpendicular
        assert!((dist_point_segment(5.0, 0.0, a, b) - 3.0).abs() < 1e-9); // past end -> endpoint
    }

    #[test]
    fn annotation_bounds_covers_all_kinds() {
        let stroke = AnnotationKind::Stroke(StrokeAnnotation {
            points: vec![(2.0, 5.0), (8.0, 1.0), (4.0, 9.0)],
            color: Color::BLACK,
            width: 2.0,
        });
        assert_eq!(annotation_bounds(&stroke), (2.0, 1.0, 6.0, 8.0));

        // Reversed corners (x1 < x0, y1 < y0) still normalize to a positive box.
        let shape = AnnotationKind::Shape(ShapeAnnotation {
            shape: ShapeKind::Rectangle,
            x0: 10.0,
            y0: 10.0,
            x1: 3.0,
            y1: 4.0,
            color: Color::BLACK,
            width: 1.0,
        });
        assert_eq!(annotation_bounds(&shape), (3.0, 4.0, 7.0, 6.0));

        let text = AnnotationKind::Text(TextAnnotation { x: 5.0, y: 7.0, size: 16.0, runs: vec![] });
        let (x, y, _, _) = annotation_bounds(&text);
        assert_eq!((x, y), (5.0, 7.0));
    }

    #[test]
    fn translate_annotation_shifts_every_kind() {
        let stroke = AnnotationKind::Stroke(StrokeAnnotation {
            points: vec![(1.0, 1.0), (2.0, 2.0)],
            color: Color::BLACK,
            width: 2.0,
        });
        let AnnotationKind::Stroke(s) = translate_annotation(&stroke, 10.0, -1.0) else {
            unreachable!()
        };
        assert_eq!(s.points, vec![(11.0, 0.0), (12.0, 1.0)]);

        let shape = AnnotationKind::Shape(ShapeAnnotation {
            shape: ShapeKind::Line,
            x0: 0.0,
            y0: 0.0,
            x1: 5.0,
            y1: 5.0,
            color: Color::BLACK,
            width: 1.0,
        });
        let AnnotationKind::Shape(s) = translate_annotation(&shape, 2.0, 3.0) else {
            unreachable!()
        };
        assert_eq!((s.x0, s.y0, s.x1, s.y1), (2.0, 3.0, 7.0, 8.0));
    }

    #[test]
    fn clamp_translate_keeps_box_on_page() {
        // Box at (70, 70) sized 20x20 on a 100x100 page: 10pt of headroom right/down.
        let (dx, dy) = clamp_translate(70.0, 70.0, 20.0, 20.0, 50.0, 50.0, 100.0, 100.0);
        assert_eq!((dx, dy), (10.0, 10.0));
        // Moving far left/up clamps to exactly reach the page's left/top edge.
        let (dx, dy) = clamp_translate(70.0, 70.0, 20.0, 20.0, -200.0, -200.0, 100.0, 100.0);
        assert_eq!((dx, dy), (-70.0, -70.0));
    }

    #[test]
    fn union_bounds_covers_the_whole_group() {
        let a = AnnotationKind::Shape(ShapeAnnotation {
            shape: ShapeKind::Rectangle,
            x0: 0.0,
            y0: 0.0,
            x1: 5.0,
            y1: 5.0,
            color: Color::BLACK,
            width: 1.0,
        });
        let b = AnnotationKind::Stroke(StrokeAnnotation {
            points: vec![(10.0, 10.0), (20.0, 15.0)],
            color: Color::BLACK,
            width: 1.0,
        });
        assert_eq!(union_bounds(&[a, b]), (0.0, 0.0, 20.0, 15.0));
        assert_eq!(union_bounds(&[]), (0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn rects_intersect_cases() {
        assert!(rects_intersect((0.0, 0.0, 10.0, 10.0), (5.0, 5.0, 10.0, 10.0)), "overlapping");
        assert!(!rects_intersect((0.0, 0.0, 10.0, 10.0), (10.0, 10.0, 5.0, 5.0)), "touching edges only");
        assert!(!rects_intersect((0.0, 0.0, 10.0, 10.0), (20.0, 20.0, 5.0, 5.0)), "far apart");
    }

    #[test]
    fn eraser_hits_strokes_and_shapes_not_text() {
        let stroke = AnnotationKind::Stroke(StrokeAnnotation {
            points: vec![(0.0, 0.0), (10.0, 0.0)],
            color: Color::BLACK,
            width: 2.0,
        });
        assert!(eraser_hits(&stroke, 5.0, 1.0, 1.0), "within radius+halfwidth");
        assert!(!eraser_hits(&stroke, 5.0, 10.0, 1.0), "too far");

        let rect = AnnotationKind::Shape(ShapeAnnotation {
            shape: ShapeKind::Rectangle,
            x0: 0.0,
            y0: 0.0,
            x1: 10.0,
            y1: 10.0,
            color: Color::BLACK,
            width: 1.0,
        });
        assert!(eraser_hits(&rect, 0.0, 5.0, 1.0), "on the left edge");
        assert!(!eraser_hits(&rect, 5.0, 5.0, 1.0), "interior, not on the outline");

        let text = AnnotationKind::Text(TextAnnotation {
            x: 0.0,
            y: 0.0,
            size: 16.0,
            runs: vec![],
        });
        assert!(!eraser_hits(&text, 0.0, 0.0, 100.0), "eraser never removes text");
    }

    #[test]
    fn stroke_renders_dark_pixels() {
        let mut page = a4_page();
        page.annotations.push(Annotation {
            id: Uuid::new_v4(),
            kind: AnnotationKind::Stroke(StrokeAnnotation {
                points: vec![(10.0, 10.0), (200.0, 200.0)],
                color: Color::BLACK,
                width: 4.0,
            }),
        });
        let mut cache = HashMap::new();
        let (pw, ph) = (A4.0.ceil() as i32, A4.1.ceil() as i32);
        let mut surface = page_surface(None, &mut cache, 0, &page, 1.0, pw, ph).unwrap();
        drop(cache);
        surface.flush();

        let data = surface.data().unwrap();
        let dark = data.iter().filter(|&&b| b < 0x40).count();
        assert!(dark > 0, "the stroke should paint dark pixels");
    }
}

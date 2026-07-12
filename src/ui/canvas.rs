use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gtk::cairo;
use gtk::gdk;
use gtk::glib;
use gtk::prelude::*;
use uuid::Uuid;

use crate::engine::OpenDocument;
use crate::engine::document::{
    A4, Annotation, AnnotationKind, Color, Document, Page, PageKind, TextAnnotation, TextRun,
};
use crate::engine::pdf::PdfDocument;

const PAGE_GAP: f64 = 16.0;
const MIN_ZOOM: f64 = 0.1;
const MAX_ZOOM: f64 = 6.0;
const ZOOM_STEP: f64 = 1.25;
const TEXT_SIZE: f64 = 16.0;
const TEXT_COLOR: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
const MIN_BOX_WIDTH: f64 = 4.0;
/// Squared pixel distance a press must move before it counts as a drag (not a click).
const DRAG_THRESHOLD_SQ: f64 = 9.0;
// Bounding-box colors (r, g, b, a).
const BOX_ACTIVE: (f64, f64, f64, f64) = (0.20, 0.51, 0.92, 1.0);
const BOX_ANNOTATION: (f64, f64, f64, f64) = (0.55, 0.55, 0.60, 0.9);
const SELECTION_FILL: (f64, f64, f64, f64) = (0.20, 0.51, 0.92, 0.30);

/// A single character with its own color.
#[derive(Clone, Copy)]
struct Glyph {
    ch: char,
    color: Color,
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

    fn insert(&mut self, ch: char, color: Color) {
        self.delete_selection();
        self.glyphs.insert(self.cursor, Glyph { ch, color });
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

    /// Applies a color to the selected glyphs; returns whether anything was colored.
    fn color_selection(&mut self, color: Color) -> bool {
        if let Some((s, e)) = self.selection() {
            for g in &mut self.glyphs[s..e] {
                g.color = color;
            }
            true
        } else {
            false
        }
    }

    /// Merges consecutive same-color glyphs into runs for storage.
    fn to_runs(&self) -> Vec<TextRun> {
        let mut runs: Vec<TextRun> = Vec::new();
        for g in &self.glyphs {
            match runs.last_mut() {
                Some(last) if last.color == g.color => last.text.push(g.ch),
                _ => runs.push(TextRun { text: g.ch.to_string(), color: g.color }),
            }
        }
        runs
    }

    fn is_blank(&self) -> bool {
        self.glyphs.iter().all(|g| g.ch.is_whitespace())
    }
}

/// A text annotation lifted out of the model while being dragged.
struct DragState {
    page: usize,
    annotation: Annotation,
    orig_x: f64,
    orig_y: f64,
}

/// The transient bits drawn on top of the cached page surfaces.
struct Overlay<'a> {
    text_mode: bool,
    editing: Option<&'a TextEdit>,
    dragging: Option<&'a DragState>,
    selected: Option<(usize, Uuid)>,
}

struct State {
    doc: Option<Document>,
    pdf: Option<PdfDocument>,
    zoom: f64,
    /// Rendered pages keyed by index; cleared on zoom or structure change.
    cache: HashMap<usize, cairo::ImageSurface>,
    text_mode: bool,
    editing: Option<TextEdit>,
    /// Page nearest the viewport center; gets the red frame and anchors insert/delete.
    current: usize,
    /// Candidate for a box drag: (page, annotation index, orig x, orig y).
    drag_start: Option<(usize, usize, f64, f64)>,
    dragging: Option<DragState>,
    /// Widget-space start point while drag-selecting text inside the edited box.
    text_drag: Option<(f64, f64)>,
    /// Currently selected text box (page, annotation id) in move/select mode.
    selected: Option<(usize, Uuid)>,
    /// Font size for new text boxes (and the one being edited).
    text_size: f64,
    /// Font color for new text / new typing (and for coloring a selection).
    text_color: Color,
}

/// Where an insert/delete acts relative to the current page.
#[derive(Clone, Copy)]
pub enum Relative {
    Before,
    After,
}

/// The selectable tools. Only `Text` is interactive so far; the rest are placeholders.
#[derive(Clone, Copy, PartialEq)]
pub enum Tool {
    Select,
    Text,
    Pen,
    Eraser,
    Shape,
    Markdown,
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
            text_color: TEXT_COLOR,
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

        // Track which page is in view so the red frame follows scrolling.
        let this = self.clone();
        self.root.vadjustment().connect_value_changed(move |_| this.recompute_current());
    }

    pub fn set_open_document(&self, open: OpenDocument) {
        {
            let mut st = self.state.borrow_mut();
            st.editing = None;
            st.drag_start = None;
            st.dragging = None;
            st.text_drag = None;
            st.selected = None;
            st.doc = Some(open.model);
            st.pdf = open.pdf;
            st.zoom = 1.0;
            st.cache.clear();
        }
        self.update_layout();
    }

    /// Snapshot of the current document model (for saving).
    pub fn document(&self) -> Option<Document> {
        self.commit_editing();
        self.state.borrow().doc.clone()
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

    /// Selects the active tool. Only `Text` has behavior for now.
    pub fn set_tool(&self, tool: Tool) {
        self.set_text_mode(tool == Tool::Text);
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
        self.area.queue_draw();
    }

    /// Sets the font color. While editing: colors the selection if there is one,
    /// otherwise this becomes the color for newly typed characters.
    pub fn set_text_color(&self, color: Color) {
        {
            let mut st = self.state.borrow_mut();
            st.text_color = color;
            if let Some(ed) = st.editing.as_mut() {
                ed.color_selection(color);
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
        let current = self.current_index();
        {
            let mut st = self.state.borrow_mut();
            let (w, h) = st
                .doc
                .as_ref()
                .and_then(|d| d.pages.get(current).or_else(|| d.pages.last()))
                .map(|p| (p.width, p.height))
                .unwrap_or(A4);
            let doc = st.doc.get_or_insert_with(Document::new);
            let at = match rel {
                _ if doc.pages.is_empty() => 0,
                Relative::Before => current,
                Relative::After => current + 1,
            };
            doc.insert_blank_page(at, w, h, Color::WHITE);
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
        {
            let mut st = self.state.borrow_mut();
            match st.doc.as_mut() {
                Some(doc) if index < doc.pages.len() => {
                    doc.pages.remove(index);
                }
                _ => return,
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
        self.update_layout();
    }

    pub fn zoom_in(&self) {
        self.set_zoom(self.zoom() * ZOOM_STEP);
    }

    pub fn zoom_out(&self) {
        self.set_zoom(self.zoom() / ZOOM_STEP);
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

    /// If a box is being edited and (page, lx, ly) lands inside it, moves the caret there.
    fn click_in_editing(&self, page: usize, lx: f64, ly: f64) -> bool {
        let cursor = {
            let st = self.state.borrow();
            let Some(ed) = st.editing.as_ref() else {
                return false;
            };
            if ed.page != page {
                return false;
            }
            let (w, h) = measure_glyphs(ed.size, &ed.glyphs);
            if lx < ed.x || lx > ed.x + w || ly < ed.y || ly > ed.y + h {
                return false;
            }
            cursor_at(ed.size, &ed.glyphs, lx - ed.x, ly - ed.y)
        };
        if let Some(ed) = self.state.borrow_mut().editing.as_mut() {
            ed.set_cursor(cursor, false);
        }
        self.area.queue_draw();
        true
    }

    fn start_new_text(&self, page: usize, lx: f64, ly: f64) {
        {
            let mut st = self.state.borrow_mut();
            st.selected = None;
            let size = st.text_size;
            st.editing = Some(TextEdit {
                page,
                x: lx,
                y: ly,
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
        if self.state.borrow().editing.is_none() {
            // Not editing: Delete/Backspace removes the selected box.
            if self.state.borrow().selected.is_some()
                && matches!(keyval, gdk::Key::Delete | gdk::Key::KP_Delete | gdk::Key::BackSpace)
            {
                self.delete_selected();
                return glib::Propagation::Stop;
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
                    let color = self.state.borrow().text_color;
                    self.edit_mut(move |ed| ed.insert('\n', color));
                }
            }
            gdk::Key::BackSpace => self.edit_mut(TextEdit::backspace),
            gdk::Key::Delete | gdk::Key::KP_Delete => self.edit_mut(TextEdit::delete_forward),
            gdk::Key::Left | gdk::Key::KP_Left => self.edit_mut(move |ed| ed.move_left(extend)),
            gdk::Key::Right | gdk::Key::KP_Right => self.edit_mut(move |ed| ed.move_right(extend)),
            gdk::Key::Up | gdk::Key::KP_Up => self.edit_mut(move |ed| ed.move_up(extend)),
            gdk::Key::Down | gdk::Key::KP_Down => self.edit_mut(move |ed| ed.move_down(extend)),
            gdk::Key::Home | gdk::Key::KP_Home => self.edit_mut(move |ed| ed.move_home(extend)),
            gdk::Key::End | gdk::Key::KP_End => self.edit_mut(move |ed| ed.move_end(extend)),
            gdk::Key::a | gdk::Key::A if ctrl => self.edit_mut(TextEdit::select_all),
            _ => match keyval.to_unicode() {
                Some(ch) if !ch.is_control() => {
                    let color = self.state.borrow().text_color;
                    self.edit_mut(move |ed| ed.insert(ch, color));
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
        }
        self.area.queue_draw();
    }

    fn cancel_editing(&self) {
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

        if self.state.borrow().text_mode {
            // In text mode a drag inside the edited box selects text.
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

        // Otherwise a drag on a box grabs it for moving.
        if let Some((page, index)) = self.annotation_hit(x, y) {
            let info = {
                let st = self.state.borrow();
                st.doc
                    .as_ref()
                    .and_then(|d| d.pages.get(page))
                    .and_then(|p| p.annotations.get(index))
                    .map(|a| match &a.kind {
                        AnnotationKind::Text(t) => (t.x, t.y, a.id),
                    })
            };
            if let Some((ox, oy, id)) = info {
                let mut st = self.state.borrow_mut();
                st.drag_start = Some((page, index, ox, oy));
                st.selected = Some((page, id));
                drop(st);
                self.area.queue_draw();
            }
        }
    }

    fn on_drag_update(&self, offset_x: f64, offset_y: f64) {
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
            match &mut ds.annotation.kind {
                AnnotationKind::Text(t) => {
                    // Keep the whole box on the page so it can't slip behind it.
                    let (bw, bh) = measure_glyphs(t.size, &ann_glyphs(t));
                    t.x = (ds.orig_x + offset_x / zoom).clamp(0.0, (pw - bw).max(0.0));
                    t.y = (ds.orig_y + offset_y / zoom).clamp(0.0, (ph - bh).max(0.0));
                }
            }
            drop(st);
            self.area.queue_draw();
        }
    }

    fn lift_for_drag(&self) {
        let mut st = self.state.borrow_mut();
        let Some((page, index, ox, oy)) = st.drag_start else {
            return;
        };
        let removed = match st.doc.as_mut() {
            Some(doc) if page < doc.pages.len() && index < doc.pages[page].annotations.len() => {
                Some(doc.pages[page].annotations.remove(index))
            }
            _ => None,
        };
        if let Some(annotation) = removed {
            st.cache.remove(&page);
            st.dragging = Some(DragState { page, annotation, orig_x: ox, orig_y: oy });
        }
        st.drag_start = None;
    }

    fn on_drag_end(&self) {
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
fn annotation_at(page: &Page, lx: f64, ly: f64) -> Option<usize> {
    for (i, annotation) in page.annotations.iter().enumerate().rev() {
        match &annotation.kind {
            AnnotationKind::Text(t) => {
                let (w, h) = measure_glyphs(t.size, &ann_glyphs(t));
                if lx >= t.x && lx <= t.x + w && ly >= t.y && ly <= t.y + h {
                    return Some(i);
                }
            }
        }
    }
    None
}

fn ann_glyphs(t: &TextAnnotation) -> Vec<Glyph> {
    t.glyphs().into_iter().map(|(ch, color)| Glyph { ch, color }).collect()
}

/// Per-caret x positions and line indices for a glyph run (index 0..=len).
struct TextLayout {
    positions: Vec<(f64, usize)>,
    line_height: f64,
    max_width: f64,
    line_count: usize,
}

fn layout(c: &cairo::Context, size: f64, glyphs: &[Glyph]) -> TextLayout {
    c.set_font_size(size);
    let line_height = c.font_extents().map(|e| e.height()).unwrap_or(size).max(1.0);
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
    ctx.set_source_rgb(0.18, 0.18, 0.20);
    let _ = ctx.paint();

    let mut st = state.borrow_mut();
    let State { doc, pdf, zoom, cache, text_mode, editing, current, dragging, selected, .. } =
        &mut *st;
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
                ctx.set_source_rgb(0.85, 0.15, 0.15);
                ctx.set_line_width(3.0);
            } else {
                ctx.set_source_rgb(0.0, 0.0, 0.0);
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
            PageKind::Blank { color } => {
                let Color { r, g, b, a } = *color;
                c.set_source_rgba(r, g, b, a);
                let _ = c.paint();
                c.scale(zoom, zoom);
            }
        }
        for annotation in &page.annotations {
            match &annotation.kind {
                AnnotationKind::Text(t) => draw_glyphs(&c, t.x, t.y, t.size, &ann_glyphs(t)),
            }
        }
    }

    cache.insert(index, surface.clone());
    Some(surface)
}

/// Draws glyphs (each with its own color), honoring newlines. Context in page-point space.
fn draw_glyphs(c: &cairo::Context, x: f64, y: f64, size: f64, glyphs: &[Glyph]) {
    c.set_font_size(size);
    let Ok(fe) = c.font_extents() else {
        return;
    };
    let mut baseline = y + fe.ascent();
    c.move_to(x, baseline);
    for g in glyphs {
        if g.ch == '\n' {
            baseline += fe.height();
            c.move_to(x, baseline);
            continue;
        }
        let Color { r, g: gg, b, a } = g.color;
        c.set_source_rgba(r, gg, b, a);
        let _ = c.show_text(&g.ch.to_string());
    }
}

fn draw_overlay(c: &cairo::Context, page: &Page, index: usize, zoom: f64, overlay: &Overlay) {
    if overlay.text_mode {
        for annotation in &page.annotations {
            match &annotation.kind {
                AnnotationKind::Text(t) => {
                    let (w, h) = measure_glyphs(t.size, &ann_glyphs(t));
                    stroke_box(c, t.x, t.y, w, h, zoom, BOX_ANNOTATION);
                }
            }
        }
    }

    if let Some((sel_page, id)) = overlay.selected
        && sel_page == index
        && let Some(annotation) = page.annotations.iter().find(|a| a.id == id)
    {
        match &annotation.kind {
            AnnotationKind::Text(t) => {
                let (w, h) = measure_glyphs(t.size, &ann_glyphs(t));
                stroke_selection_handles(c, t.x, t.y, w, h, zoom);
            }
        }
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
        match &ds.annotation.kind {
            AnnotationKind::Text(t) => {
                draw_glyphs(c, t.x, t.y, t.size, &ann_glyphs(t));
                let (w, h) = measure_glyphs(t.size, &ann_glyphs(t));
                stroke_box(c, t.x, t.y, w, h, zoom, BOX_ACTIVE);
            }
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

    fn a4_page() -> Page {
        Page {
            kind: PageKind::Blank { color: Color::WHITE },
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
                runs: vec![TextRun { text: content.into(), color: TEXT_COLOR }],
            }),
        });
        page
    }

    fn glyphs_of(s: &str) -> Vec<Glyph> {
        s.chars().map(|ch| Glyph { ch, color: TEXT_COLOR }).collect()
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
            ed.insert(ch, TEXT_COLOR);
        }
        assert_eq!((text_of(&ed), ed.cursor), ("abc".to_string(), 3));

        ed.move_left(false);
        ed.insert('X', TEXT_COLOR);
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
        assert!(ed.color_selection(red));
        assert_eq!(ed.glyphs[0].color, TEXT_COLOR);
        assert_eq!(ed.glyphs[1].color, red);
        assert_eq!(ed.glyphs[2].color, red);
        assert_eq!(ed.glyphs[3].color, TEXT_COLOR);

        // Runs merge same-color neighbors.
        let runs = ed.to_runs();
        assert_eq!(runs.len(), 3);
        assert_eq!(runs[1].text, "bc");
    }

    #[test]
    fn typing_replaces_selection() {
        let mut ed = empty_edit();
        ed.glyphs = glyphs_of("abcd");
        ed.anchor = Some(1);
        ed.cursor = 3; // "bc" selected
        ed.insert('X', TEXT_COLOR);
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
}

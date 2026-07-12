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
    A4, Annotation, AnnotationKind, Color, Document, Page, PageKind, TextAnnotation,
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

/// A text box currently being typed. `x`/`y` are the top-left anchor in page points.
struct TextEdit {
    page: usize,
    x: f64,
    y: f64,
    size: f64,
    buffer: String,
    id: Uuid,
    /// The annotation this edit replaces, kept so Escape can restore it.
    original: Option<Annotation>,
}

/// A text annotation lifted out of the model while being dragged.
struct DragState {
    page: usize,
    annotation: Annotation,
    orig_x: f64,
    orig_y: f64,
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
    /// Candidate for a drag: (page, annotation index, orig x, orig y) recorded on press.
    drag_start: Option<(usize, usize, f64, f64)>,
    dragging: Option<DragState>,
}

/// Where an insert/delete acts relative to the current page.
#[derive(Clone, Copy)]
pub enum Relative {
    Before,
    After,
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
        if on {
            self.area.grab_focus();
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
        let text_mode = self.state.borrow().text_mode;

        if text_mode && n_press == 1 {
            self.commit_editing();
            if let Some((page, index)) = self.annotation_hit(x, y) {
                self.start_edit_existing(page, index);
            } else if let Some((page, lx, ly)) = self.page_hit(x, y) {
                self.start_new_text(page, lx, ly);
            }
        } else if !text_mode
            && n_press == 2
            && let Some((page, index)) = self.annotation_hit(x, y)
        {
            self.start_edit_existing(page, index);
        }
    }

    fn start_new_text(&self, page: usize, lx: f64, ly: f64) {
        self.state.borrow_mut().editing = Some(TextEdit {
            page,
            x: lx,
            y: ly,
            size: TEXT_SIZE,
            buffer: String::new(),
            id: Uuid::new_v4(),
            original: None,
        });
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
        let (x, y, size, content, id) = match &annotation.kind {
            AnnotationKind::Text(t) => (t.x, t.y, t.size, t.content.clone(), annotation.id),
        };
        {
            let mut st = self.state.borrow_mut();
            st.editing = Some(TextEdit { page, x, y, size, buffer: content, id, original: Some(annotation) });
            st.cache.remove(&page);
        }
        self.area.grab_focus();
        self.area.queue_draw();
    }

    fn on_key(&self, keyval: gdk::Key, state: gdk::ModifierType) -> glib::Propagation {
        if self.state.borrow().editing.is_none() {
            return glib::Propagation::Proceed;
        }
        match keyval {
            gdk::Key::Escape => {
                self.cancel_editing();
                glib::Propagation::Stop
            }
            // Ctrl+Enter finishes; plain Enter inserts a newline.
            gdk::Key::Return | gdk::Key::KP_Enter => {
                if state.contains(gdk::ModifierType::CONTROL_MASK) {
                    self.commit_editing();
                } else {
                    if let Some(ed) = self.state.borrow_mut().editing.as_mut() {
                        ed.buffer.push('\n');
                    }
                    self.area.queue_draw();
                }
                glib::Propagation::Stop
            }
            gdk::Key::BackSpace => {
                if let Some(ed) = self.state.borrow_mut().editing.as_mut() {
                    ed.buffer.pop();
                }
                self.area.queue_draw();
                glib::Propagation::Stop
            }
            _ => match keyval.to_unicode() {
                Some(ch) if !ch.is_control() => {
                    if let Some(ed) = self.state.borrow_mut().editing.as_mut() {
                        ed.buffer.push(ch);
                    }
                    self.area.queue_draw();
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            },
        }
    }

    fn commit_editing(&self) {
        let editing = self.state.borrow_mut().editing.take();
        let Some(ed) = editing else {
            return;
        };
        {
            let mut st = self.state.borrow_mut();
            // Non-empty text is (re)added; clearing an existing box deletes it.
            if !ed.buffer.trim().is_empty()
                && let Some(page) = st.doc.as_mut().and_then(|d| d.pages.get_mut(ed.page))
            {
                page.annotations.push(Annotation {
                    id: ed.id,
                    kind: AnnotationKind::Text(TextAnnotation {
                        x: ed.x,
                        y: ed.y,
                        content: ed.buffer,
                        size: ed.size,
                        color: TEXT_COLOR,
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
            return; // moving is only possible without a tool
        }
        if let Some((page, index)) = self.annotation_hit(x, y) {
            let orig = {
                let st = self.state.borrow();
                st.doc
                    .as_ref()
                    .and_then(|d| d.pages.get(page))
                    .and_then(|p| p.annotations.get(index))
                    .map(|a| match &a.kind {
                        AnnotationKind::Text(t) => (t.x, t.y),
                    })
            };
            if let Some((ox, oy)) = orig {
                self.state.borrow_mut().drag_start = Some((page, index, ox, oy));
            }
        }
    }

    fn on_drag_update(&self, offset_x: f64, offset_y: f64) {
        if self.state.borrow().editing.is_some() {
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
        if let Some(ds) = st.dragging.as_mut() {
            match &mut ds.annotation.kind {
                AnnotationKind::Text(t) => {
                    t.x = ds.orig_x + offset_x / zoom;
                    t.y = ds.orig_y + offset_y / zoom;
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
        let dragging = {
            let mut st = self.state.borrow_mut();
            st.drag_start = None;
            st.dragging.take()
        };
        if let Some(ds) = dragging {
            let mut st = self.state.borrow_mut();
            if let Some(page) = st.doc.as_mut().and_then(|d| d.pages.get_mut(ds.page)) {
                page.annotations.push(ds.annotation);
            }
            st.cache.remove(&ds.page);
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
                let (w, h) = measure_text(t.size, &t.content);
                if lx >= t.x && lx <= t.x + w && ly >= t.y && ly <= t.y + h {
                    return Some(i);
                }
            }
        }
    }
    None
}

/// Box (width, height) and line height in points, honoring newlines.
fn text_metrics(c: &cairo::Context, size: f64, content: &str) -> (f64, f64, f64) {
    c.set_font_size(size);
    let line_height = c.font_extents().map(|e| e.height()).unwrap_or(size);
    let mut width = 0.0_f64;
    let mut lines = 0usize;
    for line in content.split('\n') {
        let w = c.text_extents(line).map(|e| e.x_advance()).unwrap_or(0.0);
        width = width.max(w);
        lines += 1;
    }
    (width.max(MIN_BOX_WIDTH), line_height * lines.max(1) as f64, line_height)
}

/// Measures a text box (width, height) in points using a scratch context.
fn measure_text(size: f64, content: &str) -> (f64, f64) {
    let Ok(surface) = cairo::ImageSurface::create(cairo::Format::ARgb32, 1, 1) else {
        return (MIN_BOX_WIDTH, size);
    };
    let Ok(c) = cairo::Context::new(&surface) else {
        return (MIN_BOX_WIDTH, size);
    };
    let (w, h, _) = text_metrics(&c, size, content);
    (w, h)
}

fn draw(state: &Rc<RefCell<State>>, ctx: &cairo::Context, width: i32) {
    ctx.set_source_rgb(0.18, 0.18, 0.20);
    let _ = ctx.paint();

    let mut st = state.borrow_mut();
    let State { doc, pdf, zoom, cache, text_mode, editing, current, dragging, .. } = &mut *st;
    let Some(doc) = doc.as_ref() else {
        return;
    };
    let z = *zoom;
    let current = *current;

    let (_x0, cy0, _x1, cy1) = ctx.clip_extents().unwrap_or((0.0, 0.0, f64::MAX, f64::MAX));

    let mut y = PAGE_GAP;
    for (i, page) in doc.pages.iter().enumerate() {
        let pw = page.width * z;
        let ph = page.height * z;
        let x = ((width as f64) - pw) / 2.0;

        // Cull pages outside the visible band.
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

            // Overlay (boxes, in-progress text, dragged box) in page-point space.
            let _ = ctx.save();
            ctx.translate(x, y);
            ctx.scale(z, z);
            draw_overlay(ctx, page, i, z, *text_mode, editing.as_ref(), dragging.as_ref());
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
                AnnotationKind::Text(text) => draw_text(&c, text),
            }
        }
    }

    cache.insert(index, surface.clone());
    Some(surface)
}

/// Draws a text annotation; the context must be in page-point space.
fn draw_text(c: &cairo::Context, text: &TextAnnotation) {
    let Color { r, g, b, a } = text.color;
    c.set_source_rgba(r, g, b, a);
    c.set_font_size(text.size);
    let Ok(fe) = c.font_extents() else {
        return;
    };
    let mut baseline = text.y + fe.ascent();
    for line in text.content.split('\n') {
        c.move_to(text.x, baseline);
        let _ = c.show_text(line);
        baseline += fe.height();
    }
}

fn draw_overlay(
    c: &cairo::Context,
    page: &Page,
    index: usize,
    zoom: f64,
    text_mode: bool,
    editing: Option<&TextEdit>,
    dragging: Option<&DragState>,
) {
    if text_mode {
        for annotation in &page.annotations {
            match &annotation.kind {
                AnnotationKind::Text(text) => {
                    stroke_text_box(c, text.x, text.y, text.size, &text.content, zoom, BOX_ANNOTATION);
                }
            }
        }
    }

    if let Some(ed) = editing
        && ed.page == index
    {
        let preview = TextAnnotation {
            x: ed.x,
            y: ed.y,
            content: ed.buffer.clone(),
            size: ed.size,
            color: TEXT_COLOR,
        };
        draw_text(c, &preview);
        stroke_text_box(c, ed.x, ed.y, ed.size, &ed.buffer, zoom, BOX_ACTIVE);
        draw_caret(c, ed, zoom);
    }

    if let Some(ds) = dragging
        && ds.page == index
    {
        match &ds.annotation.kind {
            AnnotationKind::Text(text) => {
                draw_text(c, text);
                stroke_text_box(c, text.x, text.y, text.size, &text.content, zoom, BOX_ACTIVE);
            }
        }
    }
}

/// Strokes the bounding box of a piece of text (context in page-point space).
fn stroke_text_box(
    c: &cairo::Context,
    x: f64,
    y: f64,
    size: f64,
    content: &str,
    zoom: f64,
    rgba: (f64, f64, f64, f64),
) {
    let (width, height, _) = text_metrics(c, size, content);
    let (r, g, b, a) = rgba;
    c.set_source_rgba(r, g, b, a);
    c.set_line_width(1.0 / zoom);
    c.rectangle(x, y, width, height);
    let _ = c.stroke();
}

fn draw_caret(c: &cairo::Context, ed: &TextEdit, zoom: f64) {
    let (_, _, line_height) = text_metrics(c, ed.size, &ed.buffer);
    let last_line = ed.buffer.rsplit('\n').next().unwrap_or("");
    let lines = ed.buffer.split('\n').count().max(1);
    let last_width = c.text_extents(last_line).map(|e| e.x_advance()).unwrap_or(0.0);

    let cx = ed.x + last_width;
    let top = ed.y + (lines as f64 - 1.0) * line_height;
    let (r, g, b, a) = BOX_ACTIVE;
    c.set_source_rgba(r, g, b, a);
    c.set_line_width(1.0 / zoom);
    c.move_to(cx, top);
    c.line_to(cx, top + line_height);
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
                content: content.into(),
                size: TEXT_SIZE,
                color: TEXT_COLOR,
            }),
        });
        page
    }

    #[test]
    fn hit_test_maps_click_to_page_local_point() {
        let pages = vec![a4_page(), a4_page()];
        let width = A4.0 + 2.0 * PAGE_GAP; // matches content_size at zoom 1
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
        // A point just inside the text box's top-left.
        assert_eq!(annotation_at(&page, 101.0, 201.0), Some(0));
        // Far from the box.
        assert_eq!(annotation_at(&page, 10.0, 10.0), None);
    }

    #[test]
    fn measure_text_grows_with_newlines() {
        let (_, one) = measure_text(TEXT_SIZE, "single line");
        let (_, three) = measure_text(TEXT_SIZE, "line\ntwo\nthree");
        assert!(three > one, "more lines should be taller");
    }

    #[test]
    fn text_annotation_renders_pixels() {
        let page = text_page(50.0, 50.0, "Hello");
        let mut cache = HashMap::new();
        let (pw, ph) = (A4.0.ceil() as i32, A4.1.ceil() as i32);
        let mut surface = page_surface(None, &mut cache, 0, &page, 1.0, pw, ph).unwrap();
        // Drop the cache so we hold the only reference (data() needs exclusive access).
        drop(cache);
        surface.flush();

        let data = surface.data().unwrap();
        let non_white = data.iter().filter(|&&b| b != 0xFF).count();
        assert!(non_white > 0, "text should render as dark pixels on the white page");
    }
}

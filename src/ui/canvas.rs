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
// Bounding-box colors (r, g, b, a).
const BOX_ACTIVE: (f64, f64, f64, f64) = (0.20, 0.51, 0.92, 1.0);
const BOX_ANNOTATION: (f64, f64, f64, f64) = (0.55, 0.55, 0.60, 0.9);

/// A text box currently being typed. `x`/`y` are the top-left anchor in page points.
struct TextEdit {
    page: usize,
    x: f64,
    y: f64,
    buffer: String,
}

struct State {
    doc: Option<Document>,
    pdf: Option<PdfDocument>,
    zoom: f64,
    /// Rendered pages keyed by index; cleared on zoom or structure change.
    cache: HashMap<usize, cairo::ImageSurface>,
    text_mode: bool,
    editing: Option<TextEdit>,
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
            click.connect_pressed(move |_, _, x, y| this.on_click(x, y));
        }
        self.area.add_controller(click);

        let keys = gtk::EventControllerKey::new();
        {
            let this = self.clone();
            keys.connect_key_pressed(move |_, keyval, _, _| this.on_key(keyval));
        }
        self.area.add_controller(keys);
    }

    pub fn set_open_document(&self, open: OpenDocument) {
        {
            let mut st = self.state.borrow_mut();
            st.editing = None;
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

    /// Inserts a blank page directly after the page currently in view.
    pub fn insert_blank_page(&self) {
        self.cancel_editing();
        let current = self.current_page();
        {
            let mut st = self.state.borrow_mut();
            let (w, h) = st
                .doc
                .as_ref()
                .and_then(|d| d.pages.get(current).or_else(|| d.pages.last()))
                .map(|p| (p.width, p.height))
                .unwrap_or(A4);
            let doc = st.doc.get_or_insert_with(Document::new);
            let at = if doc.pages.is_empty() { 0 } else { current + 1 };
            doc.insert_blank_page(at, w, h, Color::WHITE);
            st.cache.clear();
        }
        self.update_layout();
    }

    /// Removes the page currently in view.
    pub fn delete_current_page(&self) {
        self.cancel_editing();
        let current = self.current_page();
        {
            let mut st = self.state.borrow_mut();
            match st.doc.as_mut() {
                Some(doc) if !doc.pages.is_empty() => {
                    doc.pages.remove(current.min(doc.pages.len() - 1));
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
    fn current_page(&self) -> usize {
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

    fn on_click(&self, x: f64, y: f64) {
        self.area.grab_focus();
        if !self.state.borrow().text_mode {
            return;
        }
        self.commit_editing();

        let hit = {
            let st = self.state.borrow();
            match st.doc.as_ref() {
                Some(doc) => hit_test(&doc.pages, st.zoom, self.area.width() as f64, x, y),
                None => None,
            }
        };
        if let Some((page, lx, ly)) = hit {
            self.state.borrow_mut().editing = Some(TextEdit { page, x: lx, y: ly, buffer: String::new() });
            self.area.queue_draw();
        }
    }

    fn on_key(&self, keyval: gdk::Key) -> glib::Propagation {
        if self.state.borrow().editing.is_none() {
            return glib::Propagation::Proceed;
        }
        match keyval {
            gdk::Key::Escape => {
                self.cancel_editing();
                glib::Propagation::Stop
            }
            gdk::Key::Return | gdk::Key::KP_Enter => {
                self.commit_editing();
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
        if ed.buffer.trim().is_empty() {
            self.area.queue_draw();
            return;
        }
        {
            let mut st = self.state.borrow_mut();
            if let Some(page) = st.doc.as_mut().and_then(|d| d.pages.get_mut(ed.page)) {
                page.annotations.push(Annotation {
                    id: Uuid::new_v4(),
                    kind: AnnotationKind::Text(TextAnnotation {
                        x: ed.x,
                        y: ed.y,
                        content: ed.buffer,
                        size: TEXT_SIZE,
                        color: TEXT_COLOR,
                    }),
                });
            }
            st.cache.remove(&ed.page);
        }
        self.area.queue_draw();
    }

    fn cancel_editing(&self) {
        if self.state.borrow_mut().editing.take().is_some() {
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

fn draw(state: &Rc<RefCell<State>>, ctx: &cairo::Context, width: i32) {
    ctx.set_source_rgb(0.18, 0.18, 0.20);
    let _ = ctx.paint();

    let mut st = state.borrow_mut();
    let State { doc, pdf, zoom, cache, text_mode, editing } = &mut *st;
    let Some(doc) = doc.as_ref() else {
        return;
    };
    let z = *zoom;

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
            ctx.set_source_rgb(0.0, 0.0, 0.0);
            ctx.set_line_width(1.0);
            let _ = ctx.stroke();

            // Overlay (boxes, in-progress text, caret) in page-point space.
            let _ = ctx.save();
            ctx.translate(x, y);
            ctx.scale(z, z);
            draw_overlay(ctx, page, i, z, *text_mode, editing.as_ref());
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
    c.move_to(text.x, text.y + fe.ascent());
    let _ = c.show_text(&text.content);
}

fn draw_overlay(
    c: &cairo::Context,
    page: &Page,
    index: usize,
    zoom: f64,
    text_mode: bool,
    editing: Option<&TextEdit>,
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
            size: TEXT_SIZE,
            color: TEXT_COLOR,
        };
        draw_text(c, &preview);
        stroke_text_box(c, ed.x, ed.y, TEXT_SIZE, &ed.buffer, zoom, BOX_ACTIVE);
        draw_caret(c, ed, zoom);
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
    c.set_font_size(size);
    let Ok(fe) = c.font_extents() else {
        return;
    };
    let width = c.text_extents(content).map(|e| e.x_advance()).unwrap_or(0.0).max(MIN_BOX_WIDTH);
    let (r, g, b, a) = rgba;
    c.set_source_rgba(r, g, b, a);
    c.set_line_width(1.0 / zoom);
    c.rectangle(x, y, width, fe.height());
    let _ = c.stroke();
}

fn draw_caret(c: &cairo::Context, ed: &TextEdit, zoom: f64) {
    c.set_font_size(TEXT_SIZE);
    let Ok(fe) = c.font_extents() else {
        return;
    };
    let width = c.text_extents(&ed.buffer).map(|e| e.x_advance()).unwrap_or(0.0);
    let cx = ed.x + width;
    let (r, g, b, a) = BOX_ACTIVE;
    c.set_source_rgba(r, g, b, a);
    c.set_line_width(1.0 / zoom);
    c.move_to(cx, ed.y);
    c.line_to(cx, ed.y + fe.height());
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
    fn text_annotation_renders_pixels() {
        let mut page = a4_page();
        page.annotations.push(Annotation {
            id: Uuid::new_v4(),
            kind: AnnotationKind::Text(TextAnnotation {
                x: 50.0,
                y: 50.0,
                content: "Hello".into(),
                size: 16.0,
                color: TEXT_COLOR,
            }),
        });

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

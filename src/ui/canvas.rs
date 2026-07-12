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

/// An in-progress text edit: the entry widget plus the page-local baseline anchor.
struct Editing {
    entry: gtk::Text,
    page: usize,
    x: f64,
    y: f64,
}

struct State {
    doc: Option<Document>,
    pdf: Option<PdfDocument>,
    zoom: f64,
    /// Rendered pages keyed by index; cleared on zoom or structure change.
    cache: HashMap<usize, cairo::ImageSurface>,
    editing: Option<Editing>,
}

#[derive(Clone)]
pub struct Canvas {
    pub root: gtk::ScrolledWindow,
    area: gtk::DrawingArea,
    layer: gtk::Fixed,
    state: Rc<RefCell<State>>,
}

impl Canvas {
    pub fn new() -> Self {
        let area = gtk::DrawingArea::new();
        let layer = gtk::Fixed::new();
        layer.put(&area, 0.0, 0.0);

        let root = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .child(&layer)
            .build();

        let state = Rc::new(RefCell::new(State {
            doc: None,
            pdf: None,
            zoom: 1.0,
            cache: HashMap::new(),
            editing: None,
        }));

        {
            let state = state.clone();
            area.set_draw_func(move |_area, ctx, width, _height| {
                draw(&state, ctx, width);
            });
        }

        let canvas = Self { root, area, layer, state };
        canvas.attach_input();
        canvas
    }

    fn attach_input(&self) {
        let click = gtk::GestureClick::new();
        let this = self.clone();
        click.connect_pressed(move |_, _, x, y| this.on_click(x, y));
        self.area.add_controller(click);
    }

    pub fn set_open_document(&self, open: OpenDocument) {
        {
            let mut st = self.state.borrow_mut();
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

    /// Appends a blank page, matching the last page's size or falling back to A4.
    pub fn insert_blank_page(&self) {
        {
            let mut st = self.state.borrow_mut();
            let (w, h) = st
                .doc
                .as_ref()
                .and_then(|d| d.pages.last())
                .map(|p| (p.width, p.height))
                .unwrap_or(A4);
            let doc = st.doc.get_or_insert_with(Document::new);
            let at = doc.pages.len();
            doc.insert_blank_page(at, w, h, Color::WHITE);
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

    fn on_click(&self, x: f64, y: f64) {
        self.commit_editing();

        let hit = {
            let st = self.state.borrow();
            let Some(doc) = st.doc.as_ref() else {
                return;
            };
            hit_test(&doc.pages, st.zoom, self.area.width() as f64, x, y)
        };
        let Some((page, lx, ly)) = hit else {
            return;
        };

        self.start_editing(page, lx, ly, x, y);
    }

    fn start_editing(&self, page: usize, lx: f64, ly: f64, wx: f64, wy: f64) {
        let entry = gtk::Text::builder().width_request(220).build();
        self.layer.put(&entry, wx, wy);
        entry.grab_focus();

        {
            let this = self.clone();
            entry.connect_activate(move |_| this.commit_editing());
        }
        {
            let focus = gtk::EventControllerFocus::new();
            let this = self.clone();
            focus.connect_leave(move |_| this.commit_editing());
            entry.add_controller(focus);
        }
        {
            let key = gtk::EventControllerKey::new();
            let this = self.clone();
            key.connect_key_pressed(move |_, keyval, _, _| {
                if keyval == gdk::Key::Escape {
                    this.cancel_editing();
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            });
            entry.add_controller(key);
        }

        self.state.borrow_mut().editing = Some(Editing { entry, page, x: lx, y: ly });
    }

    fn commit_editing(&self) {
        // Take the edit out first so re-entrant focus-leave signals become no-ops.
        let editing = self.state.borrow_mut().editing.take();
        let Some(ed) = editing else {
            return;
        };
        let text = ed.entry.text().to_string();
        self.layer.remove(&ed.entry);

        if text.trim().is_empty() {
            return;
        }

        {
            let mut st = self.state.borrow_mut();
            if let Some(page) = st.doc.as_mut().and_then(|d| d.pages.get_mut(ed.page)) {
                page.annotations.push(Annotation {
                    id: Uuid::new_v4(),
                    kind: AnnotationKind::Text(TextAnnotation {
                        x: ed.x,
                        // Store the baseline; the click anchors the text's top.
                        y: ed.y + TEXT_SIZE,
                        content: text,
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
        let editing = self.state.borrow_mut().editing.take();
        if let Some(ed) = editing {
            self.layer.remove(&ed.entry);
        }
    }

    fn update_layout(&self) {
        let (w, h) = {
            let st = self.state.borrow();
            content_size(&st)
        };
        self.area.set_content_width(w as i32);
        self.area.set_content_height(h as i32);
        self.area.set_size_request(w as i32, h as i32);
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
    let State { doc, pdf, zoom, cache, .. } = &mut *st;
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
        // Annotations are drawn in page-point space (context is already scaled).
        draw_annotations(&c, page);
    }

    cache.insert(index, surface.clone());
    Some(surface)
}

fn draw_annotations(c: &cairo::Context, page: &Page) {
    for annotation in &page.annotations {
        match &annotation.kind {
            AnnotationKind::Text(text) => {
                let Color { r, g, b, a } = text.color;
                c.set_source_rgba(r, g, b, a);
                c.set_font_size(text.size);
                c.move_to(text.x, text.y);
                let _ = c.show_text(&text.content);
            }
        }
    }
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

        // Inside first page, 10/20 from its top-left.
        let hit = hit_test(&pages, 1.0, width, left + 10.0, PAGE_GAP + 20.0);
        assert_eq!(hit, Some((0, 10.0, 20.0)));

        // Inside second page (after first page + gap).
        let second_top = PAGE_GAP + A4.1 + PAGE_GAP;
        let hit2 = hit_test(&pages, 1.0, width, left + 5.0, second_top + 1.0);
        assert_eq!(hit2, Some((1, 5.0, 1.0)));

        // In the margin/gap -> no hit.
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

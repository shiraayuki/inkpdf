use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gtk::cairo;
use gtk::prelude::*;

use crate::engine::pdf::PdfDocument;

const PAGE_GAP: f64 = 16.0;
const MIN_ZOOM: f64 = 0.1;
const MAX_ZOOM: f64 = 6.0;
const ZOOM_STEP: f64 = 1.25;

struct State {
    pdf: Option<PdfDocument>,
    zoom: f64,
    /// Rendered pages keyed by index; cleared on zoom change.
    cache: HashMap<usize, cairo::ImageSurface>,
}

#[derive(Clone)]
pub struct Canvas {
    pub root: gtk::ScrolledWindow,
    area: gtk::DrawingArea,
    state: Rc<RefCell<State>>,
}

impl Canvas {
    pub fn new() -> Self {
        let area = gtk::DrawingArea::new();
        let root = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .child(&area)
            .build();

        let state = Rc::new(RefCell::new(State {
            pdf: None,
            zoom: 1.0,
            cache: HashMap::new(),
        }));

        {
            let state = state.clone();
            area.set_draw_func(move |_area, ctx, width, _height| {
                draw(&state, ctx, width);
            });
        }

        Self { root, area, state }
    }

    pub fn set_document(&self, pdf: PdfDocument) {
        {
            let mut st = self.state.borrow_mut();
            st.pdf = Some(pdf);
            st.zoom = 1.0;
            st.cache.clear();
        }
        self.update_layout();
    }

    pub fn zoom(&self) -> f64 {
        self.state.borrow().zoom
    }

    pub fn set_zoom(&self, zoom: f64) {
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
    let Some(pdf) = st.pdf.as_ref() else {
        return (0.0, 0.0);
    };
    let z = st.zoom;
    let mut total_h = PAGE_GAP;
    let mut max_w = 0.0_f64;
    for &(w, h) in pdf.page_sizes() {
        total_h += h * z + PAGE_GAP;
        max_w = max_w.max(w * z);
    }
    (max_w + 2.0 * PAGE_GAP, total_h)
}

fn draw(state: &Rc<RefCell<State>>, ctx: &cairo::Context, width: i32) {
    ctx.set_source_rgb(0.18, 0.18, 0.20);
    let _ = ctx.paint();

    let mut st = state.borrow_mut();
    let State { pdf, zoom, cache } = &mut *st;
    let Some(pdf) = pdf.as_ref() else {
        return;
    };
    let z = *zoom;

    let (_x0, cy0, _x1, cy1) = ctx.clip_extents().unwrap_or((0.0, 0.0, f64::MAX, f64::MAX));

    let mut y = PAGE_GAP;
    for i in 0..pdf.n_pages() {
        let (pw_pt, ph_pt) = pdf.page_size(i);
        let pw = pw_pt * z;
        let ph = ph_pt * z;
        let x = ((width as f64) - pw) / 2.0;

        // Cull pages outside the visible band.
        if y + ph >= cy0
            && y <= cy1
            && let Some(surface) =
                page_surface(pdf, cache, i, z, pw.ceil() as i32, ph.ceil() as i32)
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
    pdf: &PdfDocument,
    cache: &mut HashMap<usize, cairo::ImageSurface>,
    index: usize,
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
        // poppler draws no page background, so lay down white first.
        c.set_source_rgb(1.0, 1.0, 1.0);
        let _ = c.paint();
        c.scale(zoom, zoom);
        pdf.render_page(index, &c);
    }

    cache.insert(index, surface.clone());
    Some(surface)
}

# Graph Report - inkpdf  (2026-07-13)

## Corpus Check
- 9 files · ~24,089 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 426 nodes · 1245 edges · 16 communities (11 shown, 5 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS · INFERRED: 1 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `c1503c37`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Canvas Rendering & Hit-Testing
- .record_change
- Window & Tool UI
- Document Model
- Text Editing & Styling
- Engine / PDF Loading
- Canvas
- App Entry Point
- .record_change
- settings.rs
- Option
- .attach_input
- .on_click
- .commit_editing
- .page_hit

## God Nodes (most connected - your core abstractions)
1. `Canvas` - 119 edges
2. `TextEdit` - 37 edges
3. `WindowUi` - 31 edges
4. `build()` - 28 edges
5. `Document` - 22 edges
6. `State` - 22 edges
7. `Color` - 21 edges
8. `draw_overlay()` - 17 edges
9. `AnnotationKind` - 16 edges
10. `Page` - 16 edges

## Surprising Connections (you probably didn't know these)
- `pattern_thumbnail()` --calls--> `draw_page_pattern()`  [INFERRED]
  src/ui/window.rs → src/ui/canvas.rs
- `draw_stroke()` --references--> `Color`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs
- `AppSettings` --references--> `Color`  [EXTRACTED]
  src/ui/settings.rs → src/engine/document.rs
- `color_from_rgba()` --references--> `Color`  [EXTRACTED]
  src/ui/window.rs → src/engine/document.rs
- `apply_glyph_font()` --references--> `TextStyle`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs

## Import Cycles
- None detected.

## Communities (16 total, 5 thin omitted)

### Community 0 - "Canvas Rendering & Hit-Testing"
Cohesion: 0.07
Nodes (69): HashMap, HeadingLevel, ImageSurface, a4_page(), ann_glyphs(), annotation_at(), annotation_at_also_hits_strokes_and_shapes(), annotation_at_hits_inside_and_misses_outside() (+61 more)

### Community 1 - ".record_change"
Cohesion: 0.10
Nodes (5): ScrolledWindow, Canvas, DrawingArea, Fn, FnOnce

### Community 2 - "Window & Tool UI"
Cohesion: 0.08
Nodes (63): Application, ApplicationWindow, Button, Cell, ColorDialogButton, IsA, Label, MenuItem (+55 more)

### Community 3 - "Document Model"
Cohesion: 0.07
Nodes (44): Annotation, AnnotationKind, Color, default_font(), Document, insert_blank_page_adds_page_at_index(), MarkdownAnnotation, Page (+36 more)

### Community 4 - "Text Editing & Styling"
Cohesion: 0.13
Nodes (12): Key, ModifierType, Propagation, ctrl_word_navigation_jumps_by_word(), empty_edit(), glyphs_of(), marker_and_bold_apply_to_selection(), shift_selects_and_color_applies_to_selection_only() (+4 more)

### Community 5 - "Engine / PDF Loading"
Cohesion: 0.13
Nodes (15): file_name(), OpenDocument, pdf_to_inkpdf_roundtrip_rebuilds_renderer(), Option, Path, Result, Self, String (+7 more)

### Community 6 - "Canvas"
Cohesion: 0.29
Nodes (12): AppSettings, load(), load_from(), path(), roundtrip_preserves_settings(), Default, PathBuf, Result (+4 more)

### Community 8 - ".record_change"
Cohesion: 0.14
Nodes (4): PageKind, PagePattern, content_size(), Relative

### Community 9 - "settings.rs"
Cohesion: 0.13
Nodes (10): Instant, ModelerInputEventType, clamp_translate(), clamp_translate_keeps_box_on_page(), pen_model_smooths_jittery_input(), PenModel, translate_annotation(), translate_annotation_shifts_every_kind() (+2 more)

### Community 11 - "Option"
Cohesion: 0.23
Nodes (9): Cursor, circle_cursor(), cursor_from_draw(), hit_test(), hit_test_maps_click_to_page_local_point(), plus_cursor(), Option, stroke_halo() (+1 more)

## Knowledge Gaps
- **5 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Canvas` connect `.record_change` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Document Model`, `Text Editing & Styling`, `Engine / PDF Loading`, `.record_change`, `settings.rs`, `Option`, `.attach_input`, `.on_click`, `.commit_editing`, `.page_hit`?**
  _High betweenness centrality (0.457) - this node is a cross-community bridge._
- **Why does `Document` connect `Document Model` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Engine / PDF Loading`, `.commit_editing`?**
  _High betweenness centrality (0.101) - this node is a cross-community bridge._
- **Why does `WindowUi` connect `Window & Tool UI` to `.record_change`?**
  _High betweenness centrality (0.087) - this node is a cross-community bridge._
- **Should `Canvas Rendering & Hit-Testing` be split into smaller, more focused modules?**
  _Cohesion score 0.06582278481012659 - nodes in this community are weakly interconnected._
- **Should `.record_change` be split into smaller, more focused modules?**
  _Cohesion score 0.1032258064516129 - nodes in this community are weakly interconnected._
- **Should `Window & Tool UI` be split into smaller, more focused modules?**
  _Cohesion score 0.07530022719896137 - nodes in this community are weakly interconnected._
- **Should `Document Model` be split into smaller, more focused modules?**
  _Cohesion score 0.073224043715847 - nodes in this community are weakly interconnected._
# Graph Report - inkpdf  (2026-07-13)

## Corpus Check
- 10 files · ~29,676 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 492 nodes · 1450 edges · 24 communities (20 shown, 4 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS · INFERRED: 4 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `43f96f50`
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
- .update_layout
- Option
- Option
- .on_click
- .attach_input
- .is_empty
- Context
- parse_markdown_lines
- canvas.rs
- .update_layout
- Option
- .lift_annotation
- PenModel
- render_document_to_pdf

## God Nodes (most connected - your core abstractions)
1. `Canvas` - 128 edges
2. `TextEdit` - 37 edges
3. `WindowUi` - 37 edges
4. `build()` - 30 edges
5. `Document` - 25 edges
6. `State` - 23 edges
7. `Color` - 20 edges
8. `page_surface()` - 17 edges
9. `draw_overlay()` - 17 edges
10. `AnnotationKind` - 16 edges

## Surprising Connections (you probably didn't know these)
- `refresh()` --calls--> `add_secondary_click()`  [INFERRED]
  src/ui/file_browser.rs → src/ui/window.rs
- `refresh()` --calls--> `show_menu()`  [INFERRED]
  src/ui/file_browser.rs → src/ui/window.rs
- `build()` --calls--> `refresh()`  [INFERRED]
  src/ui/window.rs → src/ui/file_browser.rs
- `Draw` --references--> `Color`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs
- `draw_stroke()` --references--> `Color`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs

## Import Cycles
- None detected.

## Communities (24 total, 4 thin omitted)

### Community 0 - "Canvas Rendering & Hit-Testing"
Cohesion: 0.10
Nodes (25): annotation_at(), annotation_bounds(), annotation_bounds_covers_all_kinds(), bounds_of_points(), dist_point_segment(), eraser_hits(), fraction_needs_more_height_than_plain_text(), history_record_clears_redo_and_caps() (+17 more)

### Community 1 - ".record_change"
Cohesion: 0.10
Nodes (4): ScrolledWindow, Canvas, DrawingArea, Fn

### Community 2 - "Window & Tool UI"
Cohesion: 0.07
Nodes (63): Application, ApplicationWindow, Cell, ColorDialogButton, IsA, MenuItem, RGBA, Separator (+55 more)

### Community 3 - "Document Model"
Cohesion: 0.11
Nodes (24): Annotation, AnnotationKind, Color, default_font(), Document, insert_blank_page_adds_page_at_index(), LatexAnnotation, MarkdownAnnotation (+16 more)

### Community 4 - "Text Editing & Styling"
Cohesion: 0.13
Nodes (10): ctrl_word_navigation_jumps_by_word(), empty_edit(), glyphs_of(), marker_and_bold_apply_to_selection(), read_math_span_rejects_unterminated_or_cross_line(), shift_selects_and_color_applies_to_selection_only(), text_edit_insert_delete_and_navigate(), text_edit_vertical_keeps_column() (+2 more)

### Community 5 - "Engine / PDF Loading"
Cohesion: 0.12
Nodes (17): file_name(), OpenDocument, pdf_to_inkpdf_roundtrip_rebuilds_renderer(), Option, Path, Result, Self, String (+9 more)

### Community 6 - "Canvas"
Cohesion: 0.29
Nodes (12): AppSettings, load(), load_from(), path(), roundtrip_preserves_settings(), Default, PathBuf, Result (+4 more)

### Community 8 - ".record_change"
Cohesion: 0.18
Nodes (16): ListBox, FileBrowser, icon_button(), initial_dir(), list_dir_entries(), list_dir_entries_shows_dirs_and_pdf_inkpdf_only(), refresh(), Box (+8 more)

### Community 11 - "Option"
Cohesion: 0.23
Nodes (14): a4_page(), annotation_at_also_hits_strokes_and_shapes(), annotation_at_hits_inside_and_misses_outside(), ellipse_shape_does_not_touch_its_bounding_box_corner(), export_pdf_writes_a_readable_multi_page_pdf(), highlighted_text_renders_marker_pixels(), latex_annotation_renders_pixels(), markdown_annotation_renders_pixels() (+6 more)

### Community 12 - "Option"
Cohesion: 0.26
Nodes (4): clamp_translate(), clamp_translate_keeps_box_on_page(), translate_annotation(), translate_annotation_shifts_every_kind()

### Community 14 - ".attach_input"
Cohesion: 0.14
Nodes (15): HashMap, ImageSurface, content_size(), DragKind, DragState, Draw, LassoOp, LassoShape (+7 more)

### Community 16 - "Context"
Cohesion: 0.23
Nodes (24): apply_glyph_font(), cursor_at(), draw_annotation(), draw_caret(), draw_glyphs(), draw_glyphs_with_math(), draw_one_glyph(), draw_overlay() (+16 more)

### Community 17 - "parse_markdown_lines"
Cohesion: 0.15
Nodes (16): HeadingLevel, heading_scale(), MathSplit, MdLine, MdPiece, MdRun, parse_markdown_lines(), parse_markdown_lines_list_items_get_prefixes() (+8 more)

### Community 18 - "canvas.rs"
Cohesion: 0.18
Nodes (13): ann_glyphs(), glyphs_from_plain(), History, history_step_round_trips(), math_symbol(), MathToken, parse_math(), parse_math_depth() (+5 more)

### Community 19 - ".update_layout"
Cohesion: 0.16
Nodes (4): Key, ModifierType, Propagation, FnOnce

### Community 20 - "Option"
Cohesion: 0.28
Nodes (10): Cursor, circle_cursor(), cursor_from_draw(), hit_test(), hit_test_maps_click_to_page_local_point(), plus_cursor(), Option, stroke_halo() (+2 more)

### Community 21 - ".lift_annotation"
Cohesion: 0.38
Nodes (12): load(), load_capped(), load_capped_accepts_documents_within_the_cap(), load_capped_rejects_a_gzip_bomb(), load_rejects_garbage(), roundtrip_preserves_document(), Path, PathBuf (+4 more)

### Community 22 - "PenModel"
Cohesion: 0.29
Nodes (5): Instant, ModelerInputEventType, pen_model_smooths_jittery_input(), PenModel, StrokeModeler

### Community 23 - "render_document_to_pdf"
Cohesion: 0.38
Nodes (6): draw_page_pattern(), render_document_to_pdf(), Path, Result, pattern_thumbnail(), DrawingArea

## Knowledge Gaps
- **4 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Canvas` connect `.record_change` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Document Model`, `Engine / PDF Loading`, `.update_layout`, `Option`, `Option`, `.on_click`, `.attach_input`, `.is_empty`, `parse_markdown_lines`, `canvas.rs`, `.update_layout`, `Option`, `render_document_to_pdf`?**
  _High betweenness centrality (0.449) - this node is a cross-community bridge._
- **Why does `WindowUi` connect `Window & Tool UI` to `.record_change`, `.record_change`?**
  _High betweenness centrality (0.115) - this node is a cross-community bridge._
- **Why does `Document` connect `Document Model` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Engine / PDF Loading`, `.attach_input`, `canvas.rs`, `.update_layout`, `.lift_annotation`, `render_document_to_pdf`?**
  _High betweenness centrality (0.112) - this node is a cross-community bridge._
- **Should `Canvas Rendering & Hit-Testing` be split into smaller, more focused modules?**
  _Cohesion score 0.10483870967741936 - nodes in this community are weakly interconnected._
- **Should `.record_change` be split into smaller, more focused modules?**
  _Cohesion score 0.09659090909090909 - nodes in this community are weakly interconnected._
- **Should `Window & Tool UI` be split into smaller, more focused modules?**
  _Cohesion score 0.07437518819632641 - nodes in this community are weakly interconnected._
- **Should `Document Model` be split into smaller, more focused modules?**
  _Cohesion score 0.11411411411411411 - nodes in this community are weakly interconnected._
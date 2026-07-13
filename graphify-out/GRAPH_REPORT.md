# Graph Report - inkpdf  (2026-07-13)

## Corpus Check
- 10 files · ~28,475 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 476 nodes · 1395 edges · 16 communities (13 shown, 3 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS · INFERRED: 4 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `cfb84acb`
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
- .style_selection

## God Nodes (most connected - your core abstractions)
1. `Canvas` - 127 edges
2. `TextEdit` - 37 edges
3. `WindowUi` - 36 edges
4. `build()` - 29 edges
5. `State` - 23 edges
6. `Document` - 22 edges
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
- `draw_stroke()` --references--> `Color`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs
- `AppSettings` --references--> `Color`  [EXTRACTED]
  src/ui/settings.rs → src/engine/document.rs

## Import Cycles
- None detected.

## Communities (16 total, 3 thin omitted)

### Community 0 - "Canvas Rendering & Hit-Testing"
Cohesion: 0.05
Nodes (89): HashMap, HeadingLevel, Instant, a4_page(), ann_glyphs(), annotation_at(), annotation_at_also_hits_strokes_and_shapes(), annotation_at_hits_inside_and_misses_outside() (+81 more)

### Community 1 - ".record_change"
Cohesion: 0.12
Nodes (4): ScrolledWindow, Canvas, LassoShape, DrawingArea

### Community 2 - "Window & Tool UI"
Cohesion: 0.07
Nodes (62): Application, ApplicationWindow, Cell, ColorDialogButton, IsA, MenuItem, RGBA, Separator (+54 more)

### Community 3 - "Document Model"
Cohesion: 0.06
Nodes (52): ImageSurface, Annotation, AnnotationKind, Color, default_font(), Document, insert_blank_page_adds_page_at_index(), LatexAnnotation (+44 more)

### Community 4 - "Text Editing & Styling"
Cohesion: 0.11
Nodes (13): Key, ModifierType, Propagation, ctrl_word_navigation_jumps_by_word(), empty_edit(), glyphs_of(), marker_and_bold_apply_to_selection(), read_math_span_rejects_unterminated_or_cross_line() (+5 more)

### Community 5 - "Engine / PDF Loading"
Cohesion: 0.13
Nodes (15): file_name(), OpenDocument, pdf_to_inkpdf_roundtrip_rebuilds_renderer(), Option, Path, Result, Self, String (+7 more)

### Community 6 - "Canvas"
Cohesion: 0.29
Nodes (12): AppSettings, load(), load_from(), path(), roundtrip_preserves_settings(), Default, PathBuf, Result (+4 more)

### Community 8 - ".record_change"
Cohesion: 0.18
Nodes (16): ListBox, FileBrowser, icon_button(), initial_dir(), list_dir_entries(), list_dir_entries_shows_dirs_and_pdf_inkpdf_only(), refresh(), Box (+8 more)

### Community 9 - ".update_layout"
Cohesion: 0.15
Nodes (3): content_size(), Relative, FnOnce

### Community 11 - "Option"
Cohesion: 0.17
Nodes (10): Cursor, circle_cursor(), cursor_from_draw(), plus_cursor(), Option, stroke_halo(), text_cursor(), text_line_height() (+2 more)

### Community 12 - "Option"
Cohesion: 0.18
Nodes (6): clamp_translate(), clamp_translate_keeps_box_on_page(), DragKind, translate_annotation(), translate_annotation_shifts_every_kind(), union_bounds()

## Knowledge Gaps
- **3 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Canvas` connect `.record_change` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Document Model`, `Text Editing & Styling`, `Engine / PDF Loading`, `.update_layout`, `Option`, `Option`, `.on_click`, `.attach_input`, `.style_selection`?**
  _High betweenness centrality (0.462) - this node is a cross-community bridge._
- **Why does `WindowUi` connect `Window & Tool UI` to `.record_change`, `.record_change`?**
  _High betweenness centrality (0.119) - this node is a cross-community bridge._
- **Why does `Document` connect `Document Model` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Text Editing & Styling`, `Engine / PDF Loading`?**
  _High betweenness centrality (0.096) - this node is a cross-community bridge._
- **Should `Canvas Rendering & Hit-Testing` be split into smaller, more focused modules?**
  _Cohesion score 0.0510989010989011 - nodes in this community are weakly interconnected._
- **Should `.record_change` be split into smaller, more focused modules?**
  _Cohesion score 0.11666666666666667 - nodes in this community are weakly interconnected._
- **Should `Window & Tool UI` be split into smaller, more focused modules?**
  _Cohesion score 0.07469135802469136 - nodes in this community are weakly interconnected._
- **Should `Document Model` be split into smaller, more focused modules?**
  _Cohesion score 0.05765765765765766 - nodes in this community are weakly interconnected._
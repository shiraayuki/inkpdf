# Graph Report - inkpdf  (2026-07-13)

## Corpus Check
- 10 files · ~25,849 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 450 nodes · 1292 edges · 11 communities (10 shown, 1 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS · INFERRED: 4 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `f9bafd0e`
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
- .page_hit

## God Nodes (most connected - your core abstractions)
1. `Canvas` - 118 edges
2. `TextEdit` - 37 edges
3. `WindowUi` - 36 edges
4. `build()` - 29 edges
5. `State` - 23 edges
6. `Document` - 22 edges
7. `Color` - 20 edges
8. `draw_overlay()` - 17 edges
9. `Page` - 16 edges
10. `TextStyle` - 15 edges

## Surprising Connections (you probably didn't know these)
- `pattern_thumbnail()` --calls--> `draw_page_pattern()`  [INFERRED]
  src/ui/window.rs → src/ui/canvas.rs
- `refresh()` --calls--> `add_secondary_click()`  [INFERRED]
  src/ui/file_browser.rs → src/ui/window.rs
- `refresh()` --calls--> `show_menu()`  [INFERRED]
  src/ui/file_browser.rs → src/ui/window.rs
- `build()` --calls--> `refresh()`  [INFERRED]
  src/ui/window.rs → src/ui/file_browser.rs
- `draw_stroke()` --references--> `Color`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs

## Import Cycles
- None detected.

## Communities (11 total, 1 thin omitted)

### Community 0 - "Canvas Rendering & Hit-Testing"
Cohesion: 0.06
Nodes (70): HashMap, HeadingLevel, a4_page(), ann_glyphs(), annotation_at(), annotation_at_also_hits_strokes_and_shapes(), annotation_at_hits_inside_and_misses_outside(), annotation_bounds() (+62 more)

### Community 1 - ".record_change"
Cohesion: 0.05
Nodes (19): Cursor, Key, ModifierType, Propagation, ScrolledWindow, Canvas, circle_cursor(), cursor_from_draw() (+11 more)

### Community 2 - "Window & Tool UI"
Cohesion: 0.07
Nodes (63): Application, ApplicationWindow, Cell, ColorDialogButton, IsA, MenuItem, RGBA, Separator (+55 more)

### Community 3 - "Document Model"
Cohesion: 0.06
Nodes (47): ImageSurface, Annotation, AnnotationKind, Color, default_font(), Document, insert_blank_page_adds_page_at_index(), MarkdownAnnotation (+39 more)

### Community 4 - "Text Editing & Styling"
Cohesion: 0.14
Nodes (9): ctrl_word_navigation_jumps_by_word(), empty_edit(), glyphs_of(), marker_and_bold_apply_to_selection(), shift_selects_and_color_applies_to_selection_only(), text_edit_insert_delete_and_navigate(), text_edit_vertical_keeps_column(), TextEdit (+1 more)

### Community 5 - "Engine / PDF Loading"
Cohesion: 0.16
Nodes (15): file_name(), OpenDocument, pdf_to_inkpdf_roundtrip_rebuilds_renderer(), Option, Path, Result, Self, String (+7 more)

### Community 6 - "Canvas"
Cohesion: 0.29
Nodes (12): AppSettings, load(), load_from(), path(), roundtrip_preserves_settings(), Default, PathBuf, Result (+4 more)

### Community 8 - ".record_change"
Cohesion: 0.18
Nodes (16): ListBox, FileBrowser, icon_button(), initial_dir(), list_dir_entries(), list_dir_entries_shows_dirs_and_pdf_inkpdf_only(), refresh(), Box (+8 more)

### Community 9 - ".page_hit"
Cohesion: 0.09
Nodes (15): Instant, ModelerInputEventType, clamp_translate(), clamp_translate_keeps_box_on_page(), eraser_hits(), hit_test(), hit_test_maps_click_to_page_local_point(), pen_model_smooths_jittery_input() (+7 more)

## Knowledge Gaps
- **1 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Canvas` connect `.record_change` to `Canvas Rendering & Hit-Testing`, `.page_hit`, `Window & Tool UI`, `Document Model`?**
  _High betweenness centrality (0.458) - this node is a cross-community bridge._
- **Why does `WindowUi` connect `Window & Tool UI` to `.record_change`, `.record_change`?**
  _High betweenness centrality (0.125) - this node is a cross-community bridge._
- **Why does `Document` connect `Document Model` to `Canvas Rendering & Hit-Testing`, `.record_change`, `Window & Tool UI`, `Engine / PDF Loading`?**
  _High betweenness centrality (0.100) - this node is a cross-community bridge._
- **Should `Canvas Rendering & Hit-Testing` be split into smaller, more focused modules?**
  _Cohesion score 0.059370725034199726 - nodes in this community are weakly interconnected._
- **Should `.record_change` be split into smaller, more focused modules?**
  _Cohesion score 0.05128205128205128 - nodes in this community are weakly interconnected._
- **Should `Window & Tool UI` be split into smaller, more focused modules?**
  _Cohesion score 0.0725685034628124 - nodes in this community are weakly interconnected._
- **Should `Document Model` be split into smaller, more focused modules?**
  _Cohesion score 0.06351236146632566 - nodes in this community are weakly interconnected._
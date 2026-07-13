# Graph Report - inkpdf  (2026-07-13)

## Corpus Check
- 10 files · ~26,763 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 462 nodes · 1343 edges · 11 communities (10 shown, 1 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS · INFERRED: 4 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `c2035c71`
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
- Option

## God Nodes (most connected - your core abstractions)
1. `Canvas` - 122 edges
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

## Communities (11 total, 1 thin omitted)

### Community 0 - "Canvas Rendering & Hit-Testing"
Cohesion: 0.06
Nodes (80): HashMap, HeadingLevel, ImageSurface, a4_page(), ann_glyphs(), annotation_at(), annotation_at_also_hits_strokes_and_shapes(), annotation_at_hits_inside_and_misses_outside() (+72 more)

### Community 1 - ".record_change"
Cohesion: 0.05
Nodes (9): ScrolledWindow, Canvas, content_size(), LassoShape, Relative, DrawingArea, FnOnce, Uuid (+1 more)

### Community 2 - "Window & Tool UI"
Cohesion: 0.07
Nodes (64): Application, ApplicationWindow, Cell, ColorDialogButton, IsA, MenuItem, RGBA, Separator (+56 more)

### Community 3 - "Document Model"
Cohesion: 0.06
Nodes (49): Annotation, AnnotationKind, Color, default_font(), Document, insert_blank_page_adds_page_at_index(), MarkdownAnnotation, Page (+41 more)

### Community 4 - "Text Editing & Styling"
Cohesion: 0.12
Nodes (9): Key, ModifierType, Propagation, MdPiece, MdRun, String, text_edit_insert_delete_and_navigate(), text_of() (+1 more)

### Community 5 - "Engine / PDF Loading"
Cohesion: 0.16
Nodes (15): file_name(), OpenDocument, pdf_to_inkpdf_roundtrip_rebuilds_renderer(), Option, Path, Result, Self, String (+7 more)

### Community 6 - "Canvas"
Cohesion: 0.29
Nodes (12): AppSettings, load(), load_from(), path(), roundtrip_preserves_settings(), Default, PathBuf, Result (+4 more)

### Community 8 - ".record_change"
Cohesion: 0.18
Nodes (16): ListBox, FileBrowser, icon_button(), initial_dir(), list_dir_entries(), list_dir_entries_shows_dirs_and_pdf_inkpdf_only(), refresh(), Box (+8 more)

### Community 12 - "Option"
Cohesion: 0.08
Nodes (20): Cursor, Instant, ModelerInputEventType, circle_cursor(), clamp_translate(), clamp_translate_keeps_box_on_page(), cursor_from_draw(), hit_test() (+12 more)

## Knowledge Gaps
- **1 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Canvas` connect `.record_change` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Document Model`, `Text Editing & Styling`, `Option`?**
  _High betweenness centrality (0.460) - this node is a cross-community bridge._
- **Why does `WindowUi` connect `Window & Tool UI` to `.record_change`, `.record_change`?**
  _High betweenness centrality (0.122) - this node is a cross-community bridge._
- **Why does `Document` connect `Document Model` to `Canvas Rendering & Hit-Testing`, `.record_change`, `Window & Tool UI`, `Engine / PDF Loading`?**
  _High betweenness centrality (0.098) - this node is a cross-community bridge._
- **Should `Canvas Rendering & Hit-Testing` be split into smaller, more focused modules?**
  _Cohesion score 0.06020066889632107 - nodes in this community are weakly interconnected._
- **Should `.record_change` be split into smaller, more focused modules?**
  _Cohesion score 0.053613053613053616 - nodes in this community are weakly interconnected._
- **Should `Window & Tool UI` be split into smaller, more focused modules?**
  _Cohesion score 0.07226738934056007 - nodes in this community are weakly interconnected._
- **Should `Document Model` be split into smaller, more focused modules?**
  _Cohesion score 0.060641627543035995 - nodes in this community are weakly interconnected._
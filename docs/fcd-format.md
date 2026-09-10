# The FidoCAD `.fcd` format (FidoRust)

This guide explains how a FidoCAD drawing is stored as text, and how FidoRust stores layer names, colours, visibility, and project settings in the same file.

You do not need to edit these files by hand. The text form matters because it is what you copy into a forum post, an email, or a chat—and what you get back when someone shares a schematic with you.

---

## What a `.fcd` file is

A `.fcd` file is a **plain text** description of a drawing. Each line is a short command: a line, a rectangle, a library symbol, a label, and so on.

That is why FidoCAD drawings travel so well: you can select the whole drawing, copy it, and paste the text into a message. The recipient pastes it back into FidoCAD (or FidoRust) and sees the same schematic or board.

A typical file starts like this:

```text
[FIDOCAD]
```

or with a title:

```text
[FIDOCAD My amplifier]
```

Everything after that header is the drawing itself.

---

## How the drawing is written

### Coordinates and size

Positions use whole numbers only. One unit is 127 micrometres (about 5 mils). That resolution is fine for schematics and for most hobby PCBs.

### Layers

A drawing is a stack of transparent sheets. Each object sits on one layer. Lower indices are drawn first; higher indices sit on top.

FidoRust does **not** keep a fixed list of layers in the program. Each file carries its own layer table: name, colour, and whether the layer is shown. That table is written as `LD` lines (Layer Definition), described below.

Every object sits on one of these layers. For most opcodes the layer is a number at the end of the line; if it is missing, the object is on layer **0**.

A **component instance** (`MC`) is special. If the line **ends with a layer number** (including `0`), the instance is a single object on that layer: inner artwork is painted onto it. If the layer number is **omitted**, the instance uses the layers stored in the symbol definition (classic FidoCAD): parts can sit on different project layers, so colours and overlap follow those layers.

### Drawing commands you will see

You do not need to memorise these. They are listed so that a pasted block looks familiar when you open it in a text editor.

| Code | Meaning |
|------|---------|
| `LD` | Layer definition (name, colour, visibility) |
| `PS` | Project settings (grid, snap, defaults) |
| `LI` | Straight line |
| `RV` / `RP` | Empty / filled rectangle |
| `EV` / `EP` | Empty / filled ellipse |
| `PV` / `PP` | Empty / filled polygon |
| `BE` | Curve (Bézier) |
| `SA` | Electrical junction (connection dot) |
| `TE` | Simple text (old style) |
| `TY` | Normal text (size, angle, font, layer) |
| `PL` | PCB track (with width) |
| `PA` | PCB pad |
| `MC` | Library component (FidoCAD opcode; historically called a “macro”). Optional trailing layer: present → instance belongs to that layer; absent → use the definition’s layers. |

A small complete example:

```text
[FIDOCAD Simple divider]
LD 0 0 0 1 Schema
PS 5 5 5 5 1 1 1 25 0
MC 40 30 0 0 080
LI 40 30 80 30
SA 40 30
SA 80 30
TY 50 20 4 3 0 0 0 * R1
```

This is a resistor from the standard library, a wire, two junctions, and the label `R1`.

### Libraries (`.fcl`)

Component collections are separate `.fcl` files (`[FIDOLIB]` header, `[key Display name]` entries). They use the same drawing commands inside each component definition. When you place a component, the `.fcd` stores only a short reference (`MC x y rot mir name`), not the whole artwork.

FidoRust keeps a **project library** and any number of **user libraries**:

- **Project library** — stored in the `.fcd` itself as a trailing `[FIDOLIB project]` block. Always present in the UI; clearing it splits instances and empties the slot.
- **User libraries** — created in the app or imported from `.fcl` files, stored on this device (not in the drawing file). There is no default local library.

A project-library block looks like this:

```text
[FIDOCAD]
LI 20 20 40 20
MC 20 20 0 0 project.C01
[FIDOLIB project]
[C01 New component]
DS optional description
LI 100 100 120 100
```

`DS` is a FidoRust extension for the component description. Classic FidoCAD ignores unknown lines.

The drawing parser **stops** at a following `[FIDOLIB` / `[FIDOCAD` header, so the library primitives are not ingested into the sheet. The `[FIDOLIB project]` block is written so FidoRust keeps the definitions; older tools skip the unknown header.

User-library instances stay as `MC` references (`stem.key`) and are not stored in the file. If the drawing uses those components when you save, FidoRust asks whether to keep the local references, copy the definitions into the project library for that save only, or write the expanded primitives.

---

## Layer definitions (`LD`)

FidoRust writes one `LD` line per layer, immediately under the `[FIDOCAD]` header, before any drawing commands.

```text
LD <r> <g> <b> <visible> [a] <name…>
```

- `r`, `g`, `b`: colour components, 0–255
- `visible`: `1` shown on screen, `0` hidden
- `a`: optional alpha, 0–255. Omitted means fully opaque (`255`). Written only when the layer is translucent.
- `name`: the rest of the line (spaces allowed)
- the layer **index** is the order of the `LD` lines (the first is 0, drawn underneath)

FidoRust always writes the full table when you save.

Example:

```text
[FIDOCAD Dual rail]
LD 0 0 0 1 Schema
LD 0 80 200 1 Bottom copper
LD 200 40 40 0 Top copper
LI 20 40 180 40 1
```

Here layer 1 is named “Bottom copper” and is blue; layer 2 is hidden.

You can add, rename, recolour, hide, reorder, and delete layers in the Layers panel. Reordering changes both the `LD` order and the layer numbers on objects, so the file stays consistent.

There is no per-layer print flag.

### Files that have no `LD` lines

Older drawings (forum pastes, files saved before this format) have no layer table. When FidoRust opens them it starts from the four familiar sheets:

| Layer | Name | Colour |
|------:|------|--------|
| 0 | Schema | Black |
| 1 | PCB lato rame | Blue |
| 2 | PCB lato componenti | Green |
| 3 | Serigrafie | Teal |

If an object uses a higher index, extra generic layers (`Layer 5`, `Layer 6`, …) are added to cover it. The next save writes `LD` lines for whatever the document actually has.

If the file **does** contain `LD` lines, those lines replace this fallback entirely.

A new empty document starts with the same four layers and writes them as `LD` on the first save.

### Opening the file elsewhere

Programs that do not know `LD` typically skip the unknown line and still load the geometry. Objects keep their layer numbers; names, colours, and visibility apply only where `LD` is understood.

---

## Project settings (`PS`)

FidoRust writes one `PS` line under the layer table, before any drawing commands.

```text
PS <gridX> <gridY> <snapX> <snapY> <showGrid> <snapEnable> <hideOrigin> <strokeHundredths> <filled>
```

- `gridX`, `gridY`: grid pitch in drawing units, 1–40 (default `5`)
- `snapX`, `snapY`: snap pitch in drawing units, 1–20 (default `5`)
- `showGrid`: `1` draw the grid, `0` hide it
- `snapEnable`: `1` snap coordinates while drawing, `0` free placement
- `hideOrigin`: `1` hide the red origin handle on components, `0` show it
- `strokeHundredths`: schematic line thickness in hundredths of a drawing unit (default `25` = 0.25). Applies to lines, curves, and empty shapes in this file; PCB tracks keep their own width.
- `filled`: `1` new rectangles, ellipses, and polygons are filled, `0` they are outlines

Trailing fields may be omitted; missing values keep the defaults above. If the file has more than one `PS` line, the last one wins.

Example:

```text
[FIDOCAD Dual rail]
LD 0 0 0 1 Schema
LD 0 80 200 1 Bottom copper
PS 10 10 5 5 1 1 1 25 0
LI 20 40 180 40 1
```

### Files that have no `PS` line

Older drawings have no project-settings line. FidoRust opens them with the defaults above (grid and snap 5, grid and snap on, origin hidden, hairline stroke, empty shapes). The next save writes a `PS` line.

Programs that do not know `PS` skip the unknown line and still load the geometry.

FidoRust always writes `PS` when you save. Clipboard fragments and library definitions do not include it.

---

## Practical takeaways

1. **Sharing schematics** still works as copy-and-paste of FidoCAD text.
2. **Objects keep their layer numbers** in every `.fcd` file.
3. **Names, colours (including RGBA), and visibility** travel with the drawing as `LD` lines under the header.
4. **Grid, snap, and drawing defaults** travel as a `PS` line under the layer table.
5. **A file without `LD`** opens with the four classic layers (plus extras if needed).
6. **A file without `PS`** opens with the usual grid/snap defaults.
7. FidoRust always writes `LD` and `PS` when you save, so the next person sees the same layer table and project settings.

# The FidoCAD `.fcd` format (FidoRust)

This guide explains how a FidoCAD drawing is stored as text, and how FidoRust stores layer names, colours, and visibility in the same file.

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

Every object can sit on one of these layers. In the text, that is often a number at the end of the line. If the number is missing, the object is on layer **0**.

### Drawing commands you will see

You do not need to memorise these. They are listed so that a pasted block looks familiar when you open it in a text editor.

| Code | Meaning |
|------|---------|
| `LD` | Layer definition (name, colour, visibility) |
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
| `MC` | Library symbol (macro) |

A small complete example:

```text
[FIDOCAD Simple divider]
LD 0 0 0 1 Schema
MC 40 30 0 0 080
LI 40 30 80 30
SA 40 30
SA 80 30
TY 50 20 4 3 0 0 0 * R1
```

This is a resistor from the standard library, a wire, two junctions, and the label `R1`.

### Libraries (`.fcl`)

Symbol collections are separate `.fcl` files. They also use the same drawing commands inside each symbol definition. When you place a symbol, the `.fcd` usually stores only a short reference (`MC …`), not the whole symbol artwork—unless you save with “split non-standard macros”, which expands custom symbols into ordinary lines and shapes so others can open the drawing without your private libraries.

---

## Layer definitions (`LD`)

FidoRust writes one `LD` line per layer, immediately under the `[FIDOCAD]` header, before any drawing commands.

```text
LD <r> <g> <b> <visible> <name…>
```

- `r`, `g`, `b`: colour components, 0–255
- `visible`: `1` shown on screen, `0` hidden
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

## Practical takeaways

1. **Sharing schematics** still works as copy-and-paste of FidoCAD text.
2. **Objects keep their layer numbers** in every `.fcd` file.
3. **Names, colours, and visibility** travel with the drawing as `LD` lines under the header.
4. **A file without `LD`** opens with the four classic layers (plus extras if needed).
5. FidoRust always writes `LD` when you save, so the next person sees the same layer table.

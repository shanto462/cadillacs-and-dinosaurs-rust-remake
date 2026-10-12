"""Builds the local gallery re/out/assets/index.html (plain HTML, relative links, opens from disk) and its pages.
Everything it links to is in re/out/assets (git-ignored). Do not publish it."""
import html
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from romio import OUT, outdir

CSS = """<style>
:root{--bg:#14161e;--fg:#e6e8f0;--mut:#9aa0b8;--card:#1d2030;--acc:#7fb4ff;--line:#2c3048}
@media (prefers-color-scheme: light){:root{--bg:#f4f5f9;--fg:#1c1f2b;--mut:#5a6075;--card:#fff;--acc:#1a56c4;--line:#d3d7e4}}
*{box-sizing:border-box}body{margin:0;background:var(--bg);color:var(--fg);font:15px/1.45 system-ui,sans-serif}
header,main{max-width:1500px;margin:0 auto;padding:12px 16px}
a{color:var(--acc)}h1{font-size:24px;margin:8px 0}h2{font-size:19px;margin:26px 0 8px;border-bottom:1px solid var(--line);padding-bottom:4px}h3{font-size:15px;margin:16px 0 6px}
.mut{color:var(--mut)}.grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(300px,1fr));gap:12px}
.card{background:var(--card);border:1px solid var(--line);border-radius:8px;padding:10px 12px}
.card b{display:block;font-size:16px;margin-bottom:2px}
table{border-collapse:collapse;width:100%;font-size:13px}td,th{border-bottom:1px solid var(--line);padding:4px 6px;text-align:left;vertical-align:top}
img{max-width:100%;height:auto;image-rendering:pixelated;background:#2a2d40}.anims{display:flex;flex-wrap:wrap;gap:10px}
figure{margin:0;background:var(--card);border:1px solid var(--line);border-radius:6px;padding:6px;font-size:12px}figcaption{color:var(--mut)}
.big img{image-rendering:auto}.sheets img{display:block;margin:6px 0;border:1px solid var(--line)}code{font:12px ui-monospace,monospace}
pre{background:var(--card);border:1px solid var(--line);padding:8px;overflow:auto;font-size:12px}
nav a{margin-right:12px}
</style>"""


def page(title, body, depth=1, back=True):
    up = "../" * depth
    nav = f'<nav><a href="{up}index.html">Gallery home</a></nav>' if back else ""
    return f"<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{html.escape(title)}</title>{CSS}</head><body><header>{nav}<h1>{html.escape(title)}</h1></header><main>{body}</main></body></html>"


def load(name, default=None):
    p = os.path.join(OUT, "data", name)
    if os.path.exists(p):
        return json.load(open(p))
    return default


def write(rel, text):
    p = os.path.join(OUT, rel)
    os.makedirs(os.path.dirname(p), exist_ok=True)
    with open(p, "w") as f:
        f.write(text)


def rel_files(sub, ext=None):
    d = os.path.join(OUT, sub)
    if not os.path.isdir(d):
        return []
    return sorted(f for f in os.listdir(d) if not ext or f.endswith(ext))


E = html.escape


def build():
    counts = {}
    sections = []
    pages_dir = outdir("pages")
    # ------------------------------------------------------------------ graphics
    b = ["<p class=mut>Graphics ROM: 4 MB of 4 bit planar tiles. Sheets show every tile of the ROM. Codes are tile numbers. Grey sheets use a grey ramp, colour sheets use the palette of the first known use.</p>"]
    for title, sub, note in (("16x16 tiles, colour (sprites and scroll2)", "gfx/tiles16_colour", "32 sheets of 1024 tiles"),
                             ("16x16 tiles, grey", "gfx/tiles16_grey", "same tiles, no palette"),
                             ("16x16 tiles that nothing uses (candidates for unused art)", "gfx/tiles16_unused", "not drawn by any composition, stage map or scene screen. Code may still use them (big fonts, text)"),
                             ("8x8 scroll1 tiles (font, HUD, pictures)", "gfx/tiles8", "codes 000 to FFF, grey and two palettes")):
        files = rel_files(sub, ".png")
        b.append(f"<h2>{E(title)} ({len(files)} sheets)</h2><p class=mut>{E(note)}</p><div class=sheets>" +
                 "".join(f'<a href="../{sub}/{f}"><img loading=lazy src="../{sub}/{f}" alt="{f}"></a>' for f in files) + "</div>")
    f32 = [f for f in rel_files("gfx", ".png") if f.startswith("tiles32_")]
    b.append(f"<h2>32x32 tiles (scroll3 far background) ({len(f32)} sheets)</h2><p class=mut>Codes 1C00 to 1FFF are the real scroll3 range, the rest of the ROM is shown for completeness.</p><div class=sheets>" +
             "".join(f'<a href="../gfx/{f}"><img loading=lazy src="../gfx/{f}" alt="{f}"></a>' for f in f32) + "</div>")
    write("pages/gfx.html", page("Graphics: tile sheets", "".join(b)))
    counts["gfx"] = sum(len(rel_files(s, ".png")) for s in ("gfx/tiles16_colour", "gfx/tiles16_grey", "gfx/tiles16_unused", "gfx/tiles8")) + len(f32)
    # ------------------------------------------------------------------ palettes
    b = ["<p class=mut>All colours are 12 bit (4 bits per channel), shown at full brightness like the game after its fade in. Pen 15 is transparent (red cross).</p>"]
    for title, pat in (("OBJ palette sets per episode and area (slot to ROM palette)", "obj_set_"), ("All OBJ ROM palettes", "obj_rom_palettes_"),
                       ("Scroll1 palette bank (HUD and text)", "scroll1_bank"), ("Scroll2 palette banks (playfield, 22 banks)", "scroll2_bank_"),
                       ("Scroll3 palette banks (far background)", "scroll3_bank_")):
        files = [f for f in rel_files("palettes", ".png") if f.startswith(pat)]
        b.append(f"<h2>{E(title)} ({len(files)})</h2><div class=sheets>" + "".join(f'<a href="../palettes/{f}"><img loading=lazy src="../palettes/{f}" alt="{f}"></a>' for f in files) + "</div>")
    write("pages/palettes.html", page("Palettes", "".join(b)))
    counts["palettes"] = len(rel_files("palettes", ".png"))
    # ------------------------------------------------------------------ compositions
    cm = load("comps.json")
    b = []
    n_sheets = 0
    if cm:
        groups = cm["groups"]
        kinds = {}
        for m in cm["comps"]:
            kinds.setdefault(m["group"], m)
        b.append(f"<p class=mut>{len(cm['comps'])} sprite compositions (16x16 hardware blocks put together). Cell label: number, byte offset in the pointer table, ROM address of the data, stage and area of the palette set. "
                 f"{sum(1 for m in cm['comps'] if not m['referenced'])} compositions are not drawn by any animation table (shown under 'unreferenced', with the neighbours that hint at their owner).</p>")
        order = sorted(groups, key=lambda g: (g.split('_')[0] != 'hero', g.split('_')[0] != 'boss', g.split('_')[0] != 'dinosaur', g.split('_')[0] != 'enemy', g))
        b.append("<ul>" + "".join(f'<li><a href="#{g}">{E(g)}</a> <span class=mut>{groups[g]["n"]} comps: {E(groups[g]["label"])}</span></li>' for g in order) + "</ul>")
        for g in order:
            gi = groups[g]
            b.append(f'<h2 id="{g}">{E(g)} <span class=mut>({gi["n"]} compositions)</span></h2><div class=sheets>')
            for s in gi["sheets"]:
                n_sheets += 1
                b.append(f'<a href="../comps/{s["file"]}"><img loading=lazy src="../comps/{s["file"]}" alt="comps {s["first"]} to {s["last"]}"></a>')
            b.append("</div>")
    write("pages/comps.html", page("Sprite compositions", "".join(b)))
    counts["comps"] = cm and len(cm["comps"])
    # ------------------------------------------------------------------ animations
    am = load("anims.json", [])
    groups = {}
    for t in am:
        groups.setdefault(t["group"], []).append(t)
    idx = ['<p class=mut>Every animation of every animation table found in the program. GIF: real frame durations (1 tick = 1/60 s, GIF steps of 1/100 s). Strip: every frame with hit boxes (green body, red attack when frame type is 4, orange probe or other), '
           'labels: frame number, ROM address, composition, duration in ticks, frame type (T), body box (b), attack box (a).</p>']
    n_anims = 0
    for g in sorted(groups, key=lambda g: (g.split('_')[0] != 'hero', g.split('_')[0] != 'boss', g.split('_')[0] != 'dinosaur', g.split('_')[0] != 'enemy', g)):
        tabs = groups[g]
        na = sum(len(t["anims"]) for t in tabs)
        n_anims += na
        body = []
        for t in tabs:
            eps = ", ".join(str(e + 1) for e in t.get("episodes", [])) or "none found in the stage scripts"
            body.append(f'<h2>{E(t["label"])} <span class=mut>table {t["table"]:#x}, {t["stride"]} byte records, {len(t["anims"])} animations, episodes: {E(eps)}, box table {hex(t["box"]) if t.get("box") else "n/a"}, palette set {t["pal_set"] if t.get("pal_set") else "(0, 0) default"}</span></h2><div class=anims>')
            for a in t["anims"]:
                tag = a["tag"]
                loop = "loops to frame %d" % a["loop_to"] if a.get("loop") else ("continues at %#x" % a["link"] if a.get("link") else "")
                body.append(f'<figure><a href="../anims/{g}/{tag}_strip.png"><img loading=lazy src="../anims/{g}/{tag}.gif" alt="{tag}"></a>'
                            f'<figcaption>id {a["id"]} &middot; {a["frames"]} frames, {a["ticks"]} ticks<br>{E(loop)}</figcaption></figure>')
            body.append("</div>")
        write(f"pages/anim_{g}.html", page(f"Animations: {g}", "".join(body)))
        idx.append(f'<div class=card><b><a href="anim_{g}.html">{E(g)}</a></b><span class=mut>{len(tabs)} tables, {na} animations</span></div>')
    write("pages/anims.html", page("Animations by owner", idx[0] + '<div class=grid>' + "".join(idx[1:]) + "</div>"))
    counts["anims"] = n_anims
    counts["anim_tables"] = len(am)
    # ------------------------------------------------------------------ stages
    sm = load("stages.json")
    b = []
    if sm:
        b.append("<p class=mut>Scroll2 is the playfield (16x16 tiles), scroll3 the far background (32x32 tiles, moves slower). Each area has the full map, a copy with the camera events marked "
                 "(coloured lines: event number, camera mode, area), and 384x224 views. Palettes come from the stage tables.</p>")
        for st in sm["stages"]:
            b.append(f'<h2>Episode {st["episode"]} (stage {st["stage"]})</h2>')
            for am_ in st["areas"]:
                tag = f'ep{st["episode"]}_a{am_["area"]}'
                s2, s3 = am_["s2"], am_["s3"]
                if "note" in am_:
                    b.append(f'<h3>Area {am_["area"]}</h3><p class=mut>{E(am_["note"])}</p>')
                    continue
                b.append(f'<h3>Area {am_["area"]} <span class=mut>scroll2 {s2["w"]}x{s2["h"]} screens, scroll3 {s3["w"]}x{s3["h"]}, parallax shift {am_["shift"]}, events {am_.get("events", [])}</span></h3><div class="big sheets">')
                b.append(f'<a href="../stages/{tag}_views.png"><img loading=lazy src="../stages/{tag}_views.png"></a>')
                b.append(f'<p class=mut>scroll2 with camera events</p><a href="../stages/{tag}_scroll2_events.png"><img loading=lazy src="../stages/{tag}_scroll2_events.png" style="image-rendering:auto"></a>')
                b.append(f'<p class=mut>scroll2 (plain)</p><a href="../stages/{tag}_scroll2.png"><img loading=lazy src="../stages/{tag}_scroll2.png" style="image-rendering:auto"></a>')
                if os.path.exists(os.path.join(OUT, "stages", f"{tag}_scroll3.png")):
                    b.append(f'<p class=mut>scroll3 (far background)</p><a href="../stages/{tag}_scroll3.png"><img loading=lazy src="../stages/{tag}_scroll3.png" style="image-rendering:auto"></a>')
                b.append("</div>")
            if st["stage"] == 6:
                b.append('<h3>Scroll1 elevator cage (used as a background layer)</h3><img loading=lazy src="../stages/ep7_a1_scroll1_elevator_cage.png">')
            b.append("<table><tr><th>#</th><th>x</th><th>y</th><th>area</th><th>mode</th><th>vfollow</th><th>next</th><th>by y</th></tr>" +
                     "".join(f'<tr><td>{e["index"]}</td><td>{e["x"]:#x}</td><td>{e["y"]:#x}</td><td>{e["area"]}</td><td>{e["mode"]:#x}</td><td>{e["vfollow"]}</td><td>{e["nxt"]:#x}</td><td>{e["by_y"]}</td></tr>' for e in st["events"]) + "</table>")
        b.append("<h2>Scene screens outside the stages</h2><p class=mut>Scroll3 screens 0 to 127 (256x256, shown at half size) in the five special scene palette banks; scroll2 scene screens; scroll1 picture screens. "
                 "The right palette of each picture is the one that looks right (see the front end captures for the real screens).</p><div class=sheets>")
        for f in rel_files("stages/scenes", ".png"):
            b.append(f'<a href="../stages/scenes/{f}"><img loading=lazy src="../stages/scenes/{f}" alt="{f}"></a>')
        b.append("</div>")
    write("pages/stages.html", page("Stage maps and scene screens", "".join(b)))
    counts["stage_images"] = len(rel_files("stages", ".png")) + len(rel_files("stages/scenes", ".png"))
    # ------------------------------------------------------------------ front end
    fm = load("frontend.json", [])
    b = ["<p class=mut>Screenshots taken from MAME at fixed frames (60 Hz frames from power on) with the recipes of plan/game/NEW-PLAN.md. Click a contact sheet for the full size. Every frame is also stored as its own PNG.</p>"]
    n_shots = 0
    for sc in fm:
        n_shots += len(sc["frames"])
        b.append(f'<h2>{E(sc["name"])} <span class=mut>{len(sc["frames"])} screenshots</span></h2><p class=mut>{E(sc["title"])}</p><details><summary>mode changes</summary><pre>' +
                 E("\n".join(f"frame {f}: {m}" for f, m in sc["modes"])) + "</pre></details><div class=sheets>")
        for s in sc["sheets"]:
            b.append(f'<a href="../frontend/{s}"><img loading=lazy src="../frontend/{s}" alt="{s}"></a>')
        b.append("</div>")
    write("pages/frontend.html", page("Front end and link scenes (MAME screenshots)", "".join(b)))
    counts["frontend_shots"] = n_shots
    # ------------------------------------------------------------------ text
    tm = load("text.json")
    b = []
    if tm:
        tall = json.load(open(os.path.join(OUT, "text", "tall_font_strings.json")))
        b.append("<h2>Story, titles and captions (tall font)</h2><p class=mut>Decoded text and the rendered picture of the same string. Address = ROM address of the string data.</p><table><tr><th>address</th><th>text</th><th>picture</th></tr>")
        for e in tall:
            b.append(f'<tr><td><code>{e["addr"]:06X}</code></td><td>{"<br>".join(E(t) for t in e["text"])}</td><td>{"<img loading=lazy src=../text/images/%s>" % e["image"] if os.path.exists(os.path.join(OUT, "text", "images", e["image"])) else "(blank placeholder)"}</td></tr>')
        b.append("</table>")
        b.append("<h2>ASCII strings</h2><p class=mut>The full list is in <a href=\"../text/ascii_strings.txt\">text/ascii_strings.txt</a>; names table in <a href=\"../text/names_table.txt\">text/names_table.txt</a> (it contains names that no id points to); staff roll in <a href=\"../text/staff_roll.txt\">text/staff_roll.txt</a>.</p>")
        b.append("<pre>" + E(open(os.path.join(OUT, "text", "ascii_strings.txt")).read()) + "</pre>")
        b.append("<h2>Name table</h2><pre>" + E(open(os.path.join(OUT, "text", "names_table.txt")).read()) + "</pre>")
    write("pages/text.html", page("Text", "".join(b)))
    counts["text"] = tm and (tm["meta"]["ascii_strings"], tm["meta"]["tall_strings"])
    # ------------------------------------------------------------------ sound
    rows = json.load(open(os.path.join(OUT, "sound", "sound_codes.json"))) if os.path.exists(os.path.join(OUT, "sound", "sound_codes.json")) else []
    sm_ = json.load(open(os.path.join(OUT, "sound", "samples.json"))) if os.path.exists(os.path.join(OUT, "sound", "samples.json")) else []
    b = ["<p class=mut>No audio of music or composed effects here: it comes later from our own sound driver (milestone M4). What is here: the raw QSound samples as WAV (you can play them), "
         "the decoded command table, and a text listing of every music track and effect (voices, instruments, notes, length). "
         "Z80 program: <a href=\"../sound/z80_listing.txt\">listing</a>, <a href=\"../sound/z80_opcode_image.bin\">opcode image</a>, <a href=\"../sound/z80_data_image.bin\">data image</a>. "
         "Tables: <a href=\"../sound/sound_codes.txt\">sound_codes.txt</a>, <a href=\"../sound/sound_sequences.txt\">sound_sequences.txt</a>, <a href=\"../sound/sample_table.txt\">sample_table.txt</a>, <a href=\"../sound/instrument_table.txt\">instrument_table.txt</a>.</p>",
         "<h2>Sound codes</h2><table><tr><th>code</th><th>type</th><th>voices</th><th>length</th><th>samples used</th><th>sent by</th></tr>"]
    for r in rows:
        if not r.get("ptr"):
            continue
        used = r.get("used_by", []) + r.get("hit_ids", []) + r.get("callers", [])
        ln = r.get("length_s")
        b.append(f'<tr><td><code>{r["code"]:#05x}</code></td><td>{r["kind"]}{" prio " + str(r["priority"]) if r.get("priority") is not None else ""}</td><td>{r["n_voices"]}</td>'
                 f'<td>{"%.2f s" % ln if ln is not None else "-"}{" loops" if r.get("loops") else ""}</td><td>{E(str(r.get("samples", [])))}</td><td>{E(", ".join(used[:6]))}</td></tr>')
    b.append("</table>")
    b.append(f"<h2>QSound samples ({len(sm_)})</h2><p class=mut>8 bit PCM at 24038 Hz (natural speed at the root key). Loop point not applied.</p><table><tr><th>#</th><th>bank</th><th>bytes</th><th>seconds</th><th>loop start</th><th>root key</th><th>instruments</th><th>codes</th><th>play</th></tr>")
    for s in sm_:
        b.append(f'<tr><td>{s["index"]}</td><td>{s["bank"]}</td><td>{s["length"]}</td><td>{s["seconds"]:.2f}</td><td>{s["loop"]}</td><td>{s["key"]}</td><td>{E(str(s["instruments"][:6]))}</td><td>{E(" ".join(hex(c) for c in s["codes"][:8]))}</td>'
                 f'<td><audio controls preload=none src="../sound/samples/sample_{s["index"]:03d}.wav"></audio></td></tr>')
    b.append("</table>")
    write("pages/sound.html", page("Sound", "".join(b)))
    counts["sound_codes"] = (len([r for r in rows if r.get("ptr")]), 0)
    counts["samples"] = len(sm_)
    # ------------------------------------------------------------------ index
    gm = load("gfx.json", {})
    home = ["<p class=mut>Everything extracted from <code>dino.zip</code> by <code>re/tools/dump_assets.sh</code>. Local files only: never publish or commit this folder.</p><div class=grid>"]
    cards = [
        ("pages/anims.html", "Animations", f"{counts.get('anims')} animations of {counts.get('anim_tables')} tables, each as GIF and strip with hit boxes"),
        ("pages/comps.html", "Sprite compositions", f"{counts.get('comps')} compositions on contact sheets grouped by owner, plus the unreferenced ones"),
        ("pages/stages.html", "Stage maps", f"all 8 episodes, every area: scroll2, scroll3, camera events, views; scene screens"),
        ("pages/frontend.html", "Front end and link scenes", f"{counts.get('frontend_shots')} screenshots (taken once from MAME): boot, title, profiles, demo, story, select, tally, ending, continue"),
        ("pages/gfx.html", "Tile sheets", f"{counts.get('gfx')} sheets: 8x8, 16x16, 32x32 tiles, unused tiles"),
        ("pages/palettes.html", "Palettes", f"{counts.get('palettes')} swatch images: OBJ sets, ROM palettes, scroll banks"),
        ("pages/text.html", "Text", f"{counts['text'][0] if counts.get('text') else 0} ASCII strings and {counts['text'][1] if counts.get('text') else 0} tall font strings with pictures"),
        ("pages/sound.html", "Sound", f"{counts.get('sound_codes', (0, 0))[0]} sound codes decoded to text, {counts.get('samples')} samples as WAV"),
    ]
    for href, t, d in cards:
        home.append(f'<div class=card><b><a href="{href}">{E(t)}</a></b><span class=mut>{E(d)}</span></div>')
    home.append("</div>")
    home.append("<h2>Data files</h2><ul>" + "".join(f'<li><a href="data/{f}">data/{f}</a></li>' for f in rel_files("data")) + "</ul>")
    write("index.html", page("Cadillacs and Dinosaurs asset gallery (local)", "".join(home), depth=0, back=False))
    print("index:", counts)
    return counts


if __name__ == "__main__":
    build()

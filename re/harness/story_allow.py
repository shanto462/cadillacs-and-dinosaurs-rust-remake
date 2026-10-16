#!/usr/bin/env python3
"""The allow rules for a run through a story panel (mode 8) and the level start after it (M2-26 to M2-28; re/harness/link_check.sh, tally_check.sh).

Usage: story_allow.py MAME.trc > allow.txt

Reads the (cut) MAME trace, finds the first record with the mode G.$4D0 = 8 (F), the first record after it with another mode (L: the mode 4 of the stage start, or
the mode 0xC itself after Episode 5's panel) and the stage G.$4D9, and writes the approved differences of re/harness/allow/m228.txt with the frame ranges that
depend on the recording (the derived pointers of the stage that is poked in, the first level frame of 2.5 video frames, the partial draws at the video frame
boundaries, the display shadows that lag the cameras). Each of them is a CPU time effect of the 68000 or a derived field of the port, explained in allow/m228.txt.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import trace as T  # noqa: E402


def main():
    t = T.Trace(sys.argv[1])
    first = level = stage = None
    last = 0
    heroes = 0
    shadow = {"display_y1": [], "display_y2": []}
    for rec in t.records():
        if rec.frame < 0:
            continue
        ram = rec.images["ram"]
        last = rec.frame
        if first is None and ram[0x84D0] == 8:
            first, stage = rec.frame, int(ram[0x84D9])
            heroes = sum(1 for a in (0xB274, 0xB3F4, 0xB574) if ram[a] != 0)
        elif first is not None and level is None and ram[0x84D0] != 8:
            level = rec.frame
        if first is not None:
            for key, cam, sh in (("display_y1", 0x86C8, 0x8860), ("display_y2", 0x8748, 0x8868)):
                if int(ram[cam]) != int(ram[sh]) or int(ram[cam + 1]) != int(ram[sh + 1]):
                    shadow[key].append(rec.frame)
    if first is None:
        sys.exit("no story panel in the recording")
    if level is None:
        level = last + 1  # the recording ends in the panel
    out = [open(os.path.join(HERE, "allow", "m228.txt")).read()]
    out_extra = []
    ep5 = stage == 4  # Episode 5: the phases 3 and 4 of the stage start are one frame, the first level frame is the record of the mode 0xC
    fl = level if ep5 else level + 1
    fe = fl + (2 if stage in (2, 4) else 1)  # the first level frame of Episodes 3 and 5 takes 3 video frames
    if heroes == 3:
        # with three heroes the spawn tick of the first game frame (0xFBF4, the last job of the frame) is cut one record later (objects, pool counts)
        for k in ("enemy.*", "item.*", "prop.*", "global.free_*", "global.spawn_*", "global.floor_ptr", "global.next_event", "global.hud_*", "global.splash_*"):
            out_extra.append(f"{k} {fe + 1}-{fe + 1}")
    rng = f"{fl}-{fe}"
    for k in ("cam3_state", "cam3_shift5f", "cam3_sx46", "spawn_init", "splash_timer", "splash_phase", "hud_timer", "cam3_*", "video_ctrl", "cam2_*",
              "event_x", "event_y", "push50", "up52", "down54", "event_idx"):
        out.append(f"global.{k} {rng}")
    for k in ("scroll2", "scroll3", "sprites"):
        out.append(f"{k} {rng}")
    # the pool A init (0x4C48) is cut by the video frame boundary in the record of the mode 4
    for k in ("enemy.live", "global.free_count", "global.free_ptr", "global.free_slot"):
        out.append(f"{k} {level}-{level}")
    # the derived pointers of the stage that is poked in (the lists belong to the stage before until the level starts)
    for k in ("spawn_list1", "spawn_list2", "floor_ptr", "next_event", "map_ptr"):
        out.append(f"global.{k} {first}-{fe}")
    out.append(f"global.display_y1 {level - 2}-{level - 1}")
    out.append(f"global.display_y2 {fl}-{fl}")
    if stage == 2:
        # the first game frame of Episode 3: the third frame of the story panel's body is cut (scroll1, scroll2, palette)
        out += [f"scroll1 {first + 2}-{first + 2}", f"scroll2 {first + 2}-{first + 2}", f"palette {first + 2}-{first + 2}"]
    if ep5:
        out.append(f"scroll2 {first + 302}-{first + 302}")  # the second picture of the panel is cut at the boundary
        out.append(f"palette {level - 2}-{level - 2}")  # 0x8A44 is cut
        out += [f"global.{k} {rng}" for k in ("blink_phase", "blink_timer", "cam_y", "shake_scroll1_y")]
        out += [f"player.{k} {rng}" for k in ("visible", "mode", "timer", "bd0", "sc_act", "sc_ptr", "sc_timer")]
    for key, frames in shadow.items():
        out += [f"global.{key} {fr}" for fr in frames]
    out += out_extra
    print("\n".join(out))


main()

#!/bin/sh
# M10-27 (task 71): every natural and flow whole-run recording of the final regression, through re/harness/m10_check.sh.
# Usage: [MODES="step lagsync"] [PORT_OUT=DIR] sh re/harness/m10_all.sh   (MODES default: step lagsync run, as m10_check.sh)
# The list: every NAME that has a section in allow/m10_runs.txt and every trace of ep[1-8]_*, m517_ep1_*, m523_ep1_*, m1019_*, m1020_*, m328_end-*
# that has an input the harness can replay (the *_probe traces and the traces without input are left out). NAME:FRAMES, FRAMES = the trace record count.
set -e
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
MODES=${MODES:-"step lagsync"}
export MODES
LIST=""
# Episode 1: the three-player and hero runs (m517_ep1_*; the .full variants have no input under their own name: left out)
LIST="$LIST m517_ep1_3p:24336 m517_ep1_hannah:15456 m517_ep1_jack:29857 m517_ep1_mess:16145 m517_ep1_mustapha:30544 m523_ep1_2p:30700 m523_ep1_2p_full:40000 m523_ep1_3pjhm:33800 m523_ep1_3pjhm_full:40000"
# Episodes 1 to 8 natural whole runs (ep*_ heroes and 3-player runs; the *_probe traces are left out)
LIST="$LIST ep1_3p:24336 ep2_3p:42000 ep2_jack:42000 ep3_3p:10500 ep3_hannah:10500 ep3_jack:10500 ep3_mess:10500 ep3_mustapha:10500 ep4_3p:28000 ep4_jack:28000 ep5_3p:65000 ep5_hannah:65000 ep5_jack:65000 ep5_mess:65000 ep5_mustapha:65000 ep6_3p:70000 ep6_jack:70000 ep7_3p:60000 ep7_jack:60000 ep8_3p:70000 ep8_hannah:70000 ep8_jack:70000 ep8_mess:70000 ep8_mustapha:70000"
# (ep8_row and ep8_watch have no input under their own name: left out)
LIST="$LIST "
# Boss and scene scripts from m1019 (holds and walks, short)
LIST="$LIST m1019_butcher_hold_hannah:450 m1019_butcher_hold_jack:450 m1019_butcher_hold_mess:450 m1019_butcher_hold_mustapha:450 m1019_hold_hannah:450 m1019_hold_jack:450 m1019_hold_mess:450 m1019_hold_mustapha:450 m1019_wjk30_hannah:130 m1019_wjk30_mustapha:130 m1019_wjk38_hannah:130 m1019_wjk38_mustapha:130"
# Continue, game over and join scenes from m1020 (short)
LIST="$LIST m1020_e1a1_cont:900 m1020_e1a1_gameover:2300 m1020_e1a1_gameover_name:2700 m1020_e1a1_join:400 m1020_e3a0_cont:900 m1020_e3a0_gameover:2300 m1020_e3a0_gameover_name:2700 m1020_e3a0_join:400 m1020_e7a0_cont:900 m1020_e7a0_gameover:2300 m1020_e7a0_gameover_name:2700 m1020_e7a0_join:400 m1020_e7lift_cont:900 m1020_e7lift_join:400"
# Episode 1 to 8 end scenes from m328_end (story panel and endings)
LIST="$LIST m328_end-3p:5700 m328_end-fine:5700 m328_end-hannah:5700 m328_end-hiscore:6900 m328_end-jack:5700 m328_end-mess-jack:5700 m328_end-mustapha:5700 m328_end-objs:5700"
exec sh "$ROOT/re/harness/m10_check.sh" $LIST

#!/bin/sh
# M9-01 / M9-02 (the tag team move): the tag team recordings against the port (step mode, MODE=run for the free run); the same as m603_check.sh with the tag team list.
# The recordings are made with `DINO_BOOT=re/harness/scripts/BOOT.txt DINO_POKE=re/harness/pokes/POKE.lua DINO_SYNC=control+86 re/harness/record.sh re/harness/scripts/NAME.txt NAME FRAMES`
# (boot_2p: P1 Jack, P2 Hannah; boot_2p_mm: Mustapha and Mess; boot_2p_hj: Hannah and Jack; boot_2p_mu: Mess and Mustapha; boot_3p: Jack, Mustapha, Mess).
# Usage: [MODE=run] re/harness/m901_check.sh [NAME[:FRAMES] ...]
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
exec "$ROOT/re/harness/m603_check.sh" ${*:-m901_tie_jack_hannah:260 m901_tie_mustapha_mess:260 m901_tie_hannah_jack:260 m901_tie_mess_mustapha:260 m901_tie_dx31_dz0:260 m901_tie_dx32_dz0:260 m901_tie_dx0_dz15:260 m901_tie_dx0_dz16:260 m901_tie_weapon:260 m901_tie_inputbreak:330 m901_tie3_all:420 m901_tie3_z:420 m901_tierun_hannah_hit:240 m901_tierun_hannah_nodir:240 m901_tierun_hannah_jump:260 m901_tierun_hannah_left:260 m901_tierun_hannah_stop:260 m901_tierun_knife:240 m901_tierun_uzi:240 m901_tierun_club:240 m901_tierun_gun:240 m901_tie_air_atk_mm:260 m901_tie_air_atk_hj:260 m901_tie_air_atk_mu:260}

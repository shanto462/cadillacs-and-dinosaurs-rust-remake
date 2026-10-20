#!/bin/sh
# M8-01 (the brontosaurus foot, classes 22 and 23): the foot recordings against the port (step mode, MODE=run for the free run); the same as m603_check.sh with the foot list.
# Usage: [MODE=run] re/harness/m801_check.sh [NAME[:FRAMES] ...]
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
exec "$ROOT/re/harness/m603_check.sh" ${*:-m800_d3b_a20_foot:3000 m600_a20_bronto_0:3000 m800_a09_e6a1_foot_god:3000 m308_foot1p:3000 m308_foot3p:3000 m600_bs_e1_h4_foot:3000 m600_bs_h1_h1_foot:3000 m600_bs_h1_h1_foot_replay:3000 m600_bs_h1_h4_foot:3000 m600_h14_h_foot_poacher:3000}

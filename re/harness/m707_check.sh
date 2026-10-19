#!/bin/sh
# M7-08 / M7-09 (Bludge parts 2 and 3): the Bludge recordings of M6-00 / M7-00 against the port (step mode, MODE=run for the free run); the same as m603_check.sh with the Bludge list.
# Usage: [MODE=run] re/harness/m707_check.sh [NAME[:FRAMES] ...]
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
exec "$ROOT/re/harness/m603_check.sh" ${*:-m600_h17_bl_vomit:2200 m600_h18_bl_death:2400 m600_h26_pk_bludge:1500 m700_h17_bl_vomit:2200 m700_h17_bl_vomit_replay:2191 m700_h18_bl_death:2400 m700_h26_pk_bludge:1500 m700_h26_pk_bludge_replay:1491}

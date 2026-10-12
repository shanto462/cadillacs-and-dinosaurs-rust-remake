# Copy of the user's Claude Doc "Cadillacs and Dinosaurs (1993 Arcade) - Recreation Blueprint"

Source: https://claude.ai/code/artifact/cfb9ed5d-5956-4fa5-9396-4a168996dde2 (rev 19, read 2026-10-12). Web research, not verified against the ROM. Tables are flattened to text.


# Cadillacs and Dinosaurs (1993 Arcade) — Recreation Blueprint
 · 

## Research method and verification limits
This blueprint is built from text sources (wikis, strategy guides, databases, Wikipedia) that I opened and read, not from watching video. I cannot play longplay videos, step through frames, or read timestamps off them, so every video-timestamp and frame-data field in your brief is marked Unverified unless a source states it.
Labels used throughout:

Verified — stated by a source I opened; the source is linked.

Unverified — plausible or remembered but not confirmed by a source I opened. Treat as a lead to check against a longplay, not as fact.

Proposed — my suggestion for a remake. Never a claim about the original.
What this means for the rebuild: story order, stage names, boss names, character move lists and item behavior can be sourced from text. Scene-by-scene enemy counts, exact camera lock coordinates, hitbox sizes, startup/active/recovery frames and shot timings generally cannot, and need frame-by-frame capture of an arcade longplay or emulator (MAME) before you commit them to data files.

## Part 1 — Boot, title, character select and rooftop opening
The text sources confirm the setup, the first stage's location, the continue screen and, from one translated source, the opening exchange with Vice followed by a scripted special that clears the surrounding henchmen. None describes the Capcom logo, title screen, attract mode or character-select animations, and none gives timings, camera movement or coordinates. Those items are marked Unverified and need a longplay capture.

### What is confirmed
| 
Item | 
What sources say | 
Status | 
| 
Hardware and controls | 
Capcom CP System family board; 8-way joystick, two buttons (attack, jump); up to three players at once ([Wikipedia](https://en.wikipedia.org/wiki/Cadillacs_and_Dinosaurs_(video_game))) | 
Verified | 
| 
Playable roster | 
Four heroes: Jack (balanced), Hannah (skill), Mustapha (speed), Mess (power) ([Capcom Wiki](https://capcom.fandom.com/wiki/Cadillacs_and_Dinosaurs)) | 
Verified | 
| 
Flyer character blurbs | 
Jack: nature-loving team leader, strong slide. Hannah: scientist, item expert, high jumper, triangle jump. Mustapha: agile engineer, very fast. Mess: gentle giant, strongest, flying lariat ([Capcom Wiki](https://capcom.fandom.com/wiki/Cadillacs_and_Dinosaurs)) | 
Verified (paraphrased) | 
| 
First location | 
The heroes arrive on top of the Eastcoast 2513 building and are surrounded by Vice Terhune's men ([Capcom Wiki](https://capcom.fandom.com/wiki/Cadillacs_and_Dinosaurs)) | 
Verified | 
| 
First section | 
"Top of the Building": short, with Ferris and Gneiss bikers as basic enemies ([PrinceWatercress, Part 1](https://princewatercress.blogspot.com/2017/12/crrversion2-plays-cadillacs-and.html)) | 
Verified by one walkthrough | 
| 
Opening music | 
A fast rock track plays on the first screen for roughly 20 seconds and returns only at the ending ([Hardcore Gaming 101](https://www.hardcoregaming101.net/cadillacs-dinosaurs/)) | 
Verified by one review | 
| 
Dialogue presentation | 
Japanese version shows a speaker portrait beside each line; the Western version does not ([TCRF](https://new.tcrf.net/Cadillacs_and_Dinosaurs)) | 
Verified | 
| 
Region differences | 
Some lines or scenes are missing in Western releases; many enemies have different names ([Hardcore Gaming 101](https://www.hardcoregaming101.net/cadillacs-dinosaurs/)) | 
Verified | 
| 
Continue screen | 
Close-up view of a boss aiming a gun at the player with a 20-second countdown; no continue means he fires and the game ends ([Wikipedia](https://en.wikipedia.org/wiki/Cadillacs_and_Dinosaurs_(video_game))) | 
Verified | 
| 
Continue reward | 
After a continue the player reappears holding a bazooka and a couple of rockets ([PrinceWatercress, Part 1](https://princewatercress.blogspot.com/2017/12/crrversion2-plays-cadillacs-and.html)) | 
Verified by one walkthrough | 

### Opening beats: known and unknown
| 
Beat | 
Known | 
Status | 
| 
1. Heroes arrive on the rooftop | 
Surrounded by Vice's men at the top of the building | 
Verified (plot text) | 
| 
2. Vice speaks, hands the fight to his henchmen, and leaves | 
Japanese-version lines: Vice says he will not let the heroes interfere and leaves the rest to his men ([NM](https://en.namu.wiki/w/%EC%BA%90%EB%94%9C%EB%9D%BD%26%EB%8B%A4%EC%9D%B4%EB%85%B8%EC%86%8C%EC%96%B4/%EC%8A%A4%ED%85%8C%EC%9D%B4%EC%A7%80)). A fan wiki adds that Vice wears blue/gray here, the palette later reused for Walther ([Villains Wiki](https://villains.fandom.com/wiki/Vice_Terhune)) | 
Order of events verified by one translated source; palette Unverified | 
| 
3. The hero throws all surrounding henchmen away with the 360° special | 
NM states this directly after Vice's line and calls the move a "mega crash" | 
Verified by one translated source; animation frames Unverified | 
| 
4. First fight against bikers | 
Ferris and Gneiss; first-stage music plays | 
Verified | 
| 
5. Exact frame where input is enabled | 
Not stated anywhere | 
Unverified | 
| 
6. Camera start, hero spawn coordinates, enemy start positions | 
Not stated anywhere | 
Unverified | 
The scripted special in your brief is confirmed by one machine-translated source (NM). It gives the dialogue and the order of events but no timings, camera movement or coordinates, so the storyboard fields in your brief remain Unverified. NM also states that a map of the New York area appears at the start of every stage; its layout and animation are Unverified.

### Not found in any opened source

Capcom logo and boot sequence, title-screen art and animation, attract-mode content, insert-coin screen.

Character-select layout, portrait art, stat display and selection animation.

Any shot-by-shot camera data for the opening.

### How to capture the missing data

Run the Japanese romset in MAME and record input-free attract mode, then a start-to-first-input capture for each hero.

Use the debug mode documented on [TCRF](https://new.tcrf.net/Cadillacs_and_Dinosaurs) (hold Player 1 A, B, Coin, Up and Right on boot). Its DIP switches give a stage select, a slow-motion divider, a cross-hatch overlay, object-debug sprites and a turbo-cutscene flag. TCRF notes MAME does not expose these DIPs, so they must be set by overriding memory at the addresses listed there.

Step the capture frame by frame and log each shot against the 13 fields in your brief; this is the only reliable source for coordinates and timings.

## Part 2 — The eight episodes
Episode order, names, bosses and the main enemy types are well sourced. Per-scene spawn positions, enemy counts, camera lock coordinates and trigger conditions are not published anywhere I could open, so those fields are Unverified and need capture. Scene IDs below (S1.1, S1.2…) are my own numbering of the sections the walkthroughs describe.
Sources: PW1–PW4 = [PrinceWatercress parts 1](https://princewatercress.blogspot.com/2017/12/crrversion2-plays-cadillacs-and.html), [2](https://princewatercress.blogspot.com/2017/12/crrversion2-plays-cadillacs-and_21.html), [3](https://princewatercress.blogspot.com/2017/12/crrversion2-plays-cadillacs-and_22.html), [4](https://princewatercress.blogspot.com/2017/12/crrversion2-plays-cadillacs-and_1.html) (opened). CW = [Capcom Wiki](https://capcom.fandom.com/wiki/Cadillacs_and_Dinosaurs) (opened). WP = [Wikipedia](https://en.wikipedia.org/wiki/Cadillacs_and_Dinosaurs_(video_game)) (opened). HG = [Hardcore Gaming 101](https://www.hardcoregaming101.net/cadillacs-dinosaurs/) (opened). AQ = [Arcade Quartermaster stage guide](https://www.arcadequartermaster.com/cad_bosses.html) (search snippet only; the site blocks automated fetching).
NM = [Namu Wiki, Stage page](https://en.namu.wiki/w/%EC%BA%90%EB%94%9C%EB%9D%BD%26%EB%8B%A4%EC%9D%B4%EB%85%B8%EC%86%8C%EC%96%B4/%EC%8A%A4%ED%85%8C%EC%9D%B4%EC%A7%80) (opened; machine-translated from Korean, so treat names and nuance with caution; its dialogue lines come from the Japanese version).

### Episode overview
| 
# | 
Name used here | 
Other names seen | 
Boss | 
Notes | 
| 
1 | 
City in the Sea (Eastcoast 2513) | 
"Eastcoast 2515" in PW1, likely a typo; "City" in AQ | 
Vice Terhune and Rock Hopper | 
Rooftop, building interior, street | 
| 
2 | 
The Swamp Forest | 
"Swamp" in AQ | 
The Butcher ("Atomic" in Japan, per a fan-wiki snippet) | 
Waist-deep swamp; dead dinosaurs | 
| 
3 | 
Hell Road (Desert of Death) | 
"Hellroad" in PW2; "Wasteland" in AQ | 
Hogg | 
Cadillac chase; scrolls automatically (HG) | 
| 
4 | 
Jack's Garage | 
"Garage" in AQ | 
Slice | 
Gate, two garage sections | 
| 
5 | 
Village of Flame | 
"Village" in AQ | 
Morgan, then Morgue | 
First mutation | 
| 
6 | 
Jungle and Mine | 
"Coal Mane" is the game's own spelling for the mine (PW3) | 
Tyrog, three bodies | 
Giant stomping dinosaur | 
| 
7 | 
The Vault | 
"Vault" in AQ | 
Two Slisaurs | 
Rooftop at evening, elevator, library, computer room | 
| 
8 | 
The Under World | 
"Deep Down" in AQ | 
Dr. Fessenden, two forms | 
Lab, cave, escape ending | 
Your brief lists Slisaurs as a boss in a separate position; sources place them as the Episode 7 boss in the computer room, not Episode 8.

### Episode 1 — City in the Sea
Start. The heroes reach the top of the Eastcoast 2513 building and are surrounded by Vice's men (CW). The opening presentation is Unverified (see Part 1).
| 
Scene | 
Known content | 
Status | 
| 
S1.1 Top of the Building | 
Short rooftop fight against Ferris and Gneiss bikers (PW1) | 
Verified, one source | 
| 
S1.2 In the Building | 
Hallway with new enemies: Blade (knife thrower), the fat enemies Wrench T., Hammer T. and Blk Elmer who ram the player, and the small fast Punk who somersaults (PW1). Breakable props hold points, weapons (pistols, shotguns, Uzis) and food (PW1). Armor suits can release items (AQ) | 
Verified, PW1 plus AQ snippet | 
| 
S1.3 Window drop | 
The hero jumps from a building window down to the street (AQ) | 
Unverified (one snippet) | 
| 
S1.4 47th Street and Vice | 
Boss fight (PW1) | 
Verified | 
Boss: Vice Terhune. Leaps around, uses a fast multi-punch combo, jump kicks and a revolver, and fires into the air to call extra henchmen (PW1). He also brings Rock Hopper, a velociraptor that fights the player; beating the dinosaur turns it from orange to green and it walks away instead of dying (PW1). That it is chained and that Vice frees it comes from a fan-wiki snippet ([Villains Wiki](https://villains.fandom.com/wiki/Vice_Terhune)): Unverified.
End. After the beating Vice tells the heroes the Butcher is in the northern woods (CW, WP). The transition animation and any map screen are Unverified.

### Episode 2 — The Swamp Forest
Start. The heroes follow Vice's tip into the swamp forest and find many dead dinosaurs (WP). The Cadillac arriving here, as in your brief, is not mentioned in any source I opened: Unverified.
| 
Scene | 
Known content | 
Status | 
| 
S2.1 Swamp approach | 
NM lists the stage order as jungle, swamp, slaughterhouse. A scoring item is hidden under three foreground trees at the start; a triceratops appears and can be hit or avoided; a T-Rex sleeps in one area and a fat enemy will beat it awake if left alone (NM). Mack the triceratops charges when orange; Poacher J and Skinner carry guns and may attack green dinosaurs, turning them hostile; Blades throw knives from the sides (PW1) | 
Verified, NM and PW1 | 
| 
S2.2 Mud Swamp | 
Gutter and Lash T. (mace on a rope, also kicks) appear; Lash Terhune acts as the area boss on leaving the brine (PW1, AQ); NM says Lash appears in the middle and late part of the stage | 
Verified | 
| 
S2.3 Butcher's clearing | 
Dead dinosaur being chopped; barrels can be broken for food and guns (AQ snippet); Razor (a Blade variant) appears in the stage (PW1) | 
Partly Unverified | 
Boss: The Butcher. Two large knives and a body-squash attack; knocking him down makes him drop a knife, and the knives are weaker than his standing combo (PW1). He summons henchmen (PW1).

### Episode 3 — Hell Road (Desert of Death)
Start. The heroes head for the desert; Hogg has decided to take Jack's garage (CW, WP). NM gives the Japanese opening lines: Hogg tells his men to smash Jack's garage while the team is away, then a hero says the road ahead is dangerous and to blow the enemies away with the Cadillac. A visible radio pickup appears a little way into the stage; taking it summons the Cadillac, which a teammate drives in and hands over (NM; a [HonestGamers review](https://www.honestgamers.com/3723/arcade/cadillacs-dinosaurs/review.html) snippet says another team member shows up in the car). The car is optional, and it shortens the stage; it can only take so much damage before it is destroyed (PW2).
| 
Scene | 
Known content | 
Status | 
| 
S3.1 Desert of Death | 
Hordes of poachers along the Desert of Death; Driver (a Ferris palette swap with more health) fights on foot (PW2). A radio pickup appears shortly after the start. Skipping it makes the stage much harder: no recovery items appear and time-outs are nearly unavoidable (NM). Guns appear in a fixed order: submachine gun, shotgun, rifle, shotgun, shotgun, M16, submachine gun, with ammo twice (NM). With the car the section scrolls automatically (HG) | 
Verified; gun order is one source | 
| 
S3.2 Hogg | 
Hogg (called Gaspar in Japan, per an AQ snippet) rides a motorcycle and throws grenades; he can also run the player down (CW, PW2, TV Tropes snippet). Phase 1 must be fought with the Cadillac, which his grenades can destroy (NM). AQ says he has only 3 vitality gauges and that ramming his bike works. Phase 2 is on foot: he flies across the screen in different directions while grunts close in from all sides (HG); grenade shadows show landing spots (PW2); a dropped shotgun makes it easy (AQ snippet) | 
Verified, with some snippet-only detail | 
See Part 3 for the vehicle event sequence.

### Episode 4 — Jack's Garage
Start. After Hogg's defeat Jack realizes gangsters have seized the garage (WP).
| 
Scene | 
Known content | 
Status | 
| 
S4.1 Gate arrival | 
Japanese lines: a hero sees the garage has been taken; Slice orders the gate shut because the heroes will return; the player, driving the Cadillac, blows away the gate defenders and three henchmen while Slice retreats and declares the hideout his (NM). CW says barrels block the entrance and the car drives through them | 
Verified, NM and CW agree | 
| 
S4.2 Garage, first section | 
Walther (a yellow Vice palette swap), Thug (a Punk variant) and bikers who die in one hit; a gate inside can be broken by attacking it on foot (PW2) | 
Verified, one source | 
| 
S4.3 Second section | 
Hammer T. and Wrench T. surround the player quickly; a special move is recommended (PW2) | 
Verified, one source | 
| 
S4.4 Factory | 
NM lists the stage order as garage exterior, parking lot, factory. Fat enemies run in at a set point where thrown dynamite catches them; a log holds an Uzi and a hunting rifle. How PW2's two sections map onto NM's three areas is my inference | 
Partly Unverified | 
| 
S4.5 Slice | 
Boomerangs, a slash, multi-boomerang throw and grenades; fast and hard to hit while moving (PW2); HG notes his "shadow" attacks | 
Verified | 
End. Defeating Slice returns the garage; the heroes then receive a radio message from an old villager about the dinosaurs' strange behavior (CW, WP, PW3).

### Episode 5 — Village of Flame
Start. The garage radio message sends the heroes to a village where dinosaurs are attacking people and someone has set the village on fire (WP).
| 
Scene | 
Known content | 
Status | 
| 
S5.1 Approach | 
Three enemies at the start, then dinosaurs that turn orange and attack if provoked; falling rocks in the mountain area; new enemy Bludge, a green tail-bearing enemy whose tongue grabs and stuns the player (PW3) | 
Verified, one source | 
| 
S5.2 Village | 
A few more enemies; water buckets hold items (PW3) | 
Verified, one source | 
| 
S5.3 The old villager | 
He says the village was razed because the villagers would not join someone's plans; Morgan shoots him with an Uzi before he names the culprit (PW3, CW) | 
Verified | 
| 
S5.4 Morgan | 
Jumps to the screen center and fires an Uzi, kicks rocks; also uses grenades and knives (PW3, CW) | 
Verified | 
Boss: Morgan, then Morgue. After being beaten Morgan speaks of the "doctor" and transforms into the dinosaur-like Morgue (CW). Morgue has a ranged slap and a quick tackle, and drops the Uzi Morgan used, which works against him (PW3). Beating Morgue reveals that Dr. Simon Fessenden is creating the hybrids (CW, WP).

### Episode 6 — Jungle and Mine
Start. The heroes reach a coal mine and another jungle (WP).
| 
Scene | 
Known content | 
Status | 
| 
S6.1 Mine entrance | 
A wall at the start can be punched open to enter the mine (PW3) | 
Verified, one source | 
| 
S6.2 Coal Mane | 
Many enemies including Lash T. and Walther; all must be cleared to advance (PW3) | 
Verified, one source | 
| 
S6.3 Jungle | 
Giant dinosaur feet near the water stomp for heavy damage and also hurt enemies (PW3); HG calls it a stomping diplodocus | 
Verified | 
| 
S6.4 Tyrog | 
Gangsters with a metal box; it opens and a tentacled creature attaches to a gangster (CW) | 
Verified | 
Boss: Tyrog, three bodies. First host is a Gneiss: slap attack and a belly slash (PW3). Second host is Wrench T., orange, spits fireballs. Third host is Walther, gray, spits acid in a circle (PW3). Tyrog finally dies on its own (CW). Jack identifies the hideout's location from a photograph found during this fight (CW).

### Episode 7 — The Vault
Start. Jack concludes Dr. Fessenden is the mastermind and the team goes to the underground hideout beneath the City in the Sea (WP).
| 
Scene | 
Known content | 
Status | 
| 
S7.1 Evening rooftop | 
Background is a palette swap of Episode 1's rooftop at evening; a pterodactyl dies in one hit (PW4) | 
Verified, one source | 
| 
S7.2 Elevator | 
Descends about 40 floors; stops periodically for enemy waves; enemies keep dropping in (PW4) | 
Verified, one source | 
| 
S7.3 Library | 
Reached by breaking a gate; two Lash T. and two Bludges at once (PW4) | 
Verified, one source | 
| 
S7.4 Computer room | 
Reached through a second gate; fat enemies first, then two Slisaurs (PW4) | 
Verified, one source | 
Boss: two Slisaurs. Copies of Slice with dinosaur DNA and the same attack patterns (PW4). The doctor appears on a computer screen, calls himself creator of a new world and orders them to finish the team; they fail (CW).

### Episode 8 — The Under World
Start. The heroes move deeper into the basement after the Slisaurs (CW).
| 
Scene | 
Known content | 
Status | 
| 
S8.1 Underground lab | 
Tubes break to release Bludges, dinosaurs and Tyrog-2 (a one-form Tyrog); wall flamethrowers are destructible (PW4). The bio-lab holds jars containing Rock Hopper, Bludges and Tyrogs (CW) | 
Verified | 
| 
S8.2 Cave | 
A downhill run with rolling barrels, then two Walthers and two Lash T. at once (PW4) | 
Verified, one source | 
| 
S8.3 Fessenden's lab | 
Boss fight | 
Verified | 
Boss: Dr. Fessenden. First form fights like Morgue (PW4, CW). The second form is a large multi-headed creature that slashes, drops fireballs on the ground and calls groups of Blades to throw knives (PW4). HG criticizes the final boss as a recolor followed by an easy final form.
Ending. Fessenden, crippled, sets the lab to self-destruct; the heroes run; Jack stops to help Hannah; Mustapha and Mess escape and walk back believing the other two died; Jack and Hannah then arrive behind them in the Cadillac and all four go home (WP, CW, PW4). The credits sequence and exact ending animation timing are Unverified.

### Source conflicts to settle with a capture
| 
Topic | 
What sources say | 
Status | 
| 
Fessenden's last form | 
Two dinosaur heads grow out of the body with Fessenden in the center, so three heads in total (NM, PW4); WP and CW say three-headed | 
Resolved | 
| 
Who falls in the escape | 
Hannah falls, Jack goes to save her and is caught in the blast (NM, WP, CW) | 
Resolved | 
| 
Garage entrance | 
CW and NM agree the Cadillac clears the Stage 4 gate; PW2 describes a gate broken on foot, likely a second gate inside; HG places a barrel crash in Stage 3 | 
Mostly resolved; confirm gate count by capture | 
| 
Episode 1 label | 
Eastcoast 2513 (CW, WP, a YouTube chapter title in a search snippet) versus 2515 (PW1) | 
Open; read the title card | 

### More scene details from NM (Episodes 5–8)

Episode 5. A radio call from the village chief opens the stage. Enemies Phantom and Bludge debut in earnest; barrels hide a bazooka mid-stage; a tough group (Walther/Colt, gunmen, Phantoms) appears just before a bridge. Morgan's second form is a dome-headed dinosaur that rises after he drops his gun, charges at range, and brings henchmen back (NM).

Episode 6. Order is abandoned mine (outside), mine (inside), jungle; it is the only stage with no firearms. Hitting Tyrog in the air always knocks him down, which can waste time on the clock. Tyrog's host changes after each defeat, with spoken lines (NM).

Episode 7. Order is rooftop, elevator, library, computer room. Zeke (the pterodactyl) is best removed with a jump attack. Fessenden greets the team on a screen and sends the Slisaurs; there are two for one player and three for two or more players, with higher health in co-op (NM).

Episode 8. Order is gene vault, sewers, secret lab cave. Bludge (jar 3) and Rock Hopper (jar 4) break their glass; Tyrog 2 emerges from jar 11 and does not revive, worth only 1000 points. Wall flamethrowers in the sewer are destroyed by punching. Barrels roll down the cave slope, and running ahead causes scroll-lock to remove some of them before their items can be taken. A meat barrel precedes the boss. Fessenden's first form is a dome-headed dinosaur like Morgan but without henchmen (NM).

Stage-start map. A map of the New York area appears at the start of each stage (NM).

## Part 3 — Cadillac sequences
The Cadillac appears in four places, not three: the Episode 3 playable chase, the Episode 4 gate arrival, the ending, and (per your brief) Episode 2. Episodes 3, 4 and the ending are supported by sources. An Episode 2 car arrival is not mentioned in any source I opened. The sources describe events and rules but not camera angles, wheel animation, hitboxes or controls, so those fields are Unverified.
| 
Claim in your brief | 
Finding | 
Status | 
| 
Ep 2: Cadillac arrives at the swamp, heroes get out | 
Not in the Wikipedia, Capcom Wiki, walkthrough or NM stage descriptions; NM's Stage 2 text never mentions the car | 
Unverified; check the Episode 2 intro in a capture | 
| 
Ep 3: Cadillac is summoned and becomes a playable vehicle | 
A radio pickup summons it; a teammate drives it in and hands it over (NM) | 
Verified | 
| 
Ep 4: Cadillac crashes into the garage entrance | 
The player in the Cadillac blows away the gate's defenders and three henchmen (NM); barrels blocking the entrance are driven through (CW) | 
Verified | 

### Episode 3 — Desert of Death chase
| 
Step | 
Trigger | 
Event | 
Status | 
| 
1 | 
Stage starts | 
Player fights on foot; a hero line says to blow the enemies away with the Cadillac (NM) | 
Verified, one source | 
| 
2 | 
A little way into the stage | 
A visible radio pickup appears (NM) | 
Verified, one source | 
| 
3 | 
Player takes the radio | 
The Cadillac is summoned; a teammate drives it in and hands it over. If the player is Jack, the delivering teammate follows the order Mustapha, Hannah, Mess (NM) | 
Verified, one source | 
| 
4 | 
Car available | 
Seats: the player who took the radio drives; the first other player sits in the passenger seat; a third player sits in the trunk. Jack, though the owner, always sits in the trunk (NM) | 
Verified, one source | 
| 
5 | 
Driving | 
The stage scrolls automatically; the car runs down poachers and crashes through barrels (HG). Steering around barrels and lining up with Hogg is the advised play (HG; a game-site snippet) | 
Verified | 
| 
6 | 
Hogg appears | 
Phase 1 is a car-versus-motorcycle fight; Hogg throws grenades and can run the player down. AQ says he has 3 vitality gauges and that ramming his bike works | 
Verified, partly snippet | 
| 
7 | 
Car health reaches zero | 
The grenade destroys the Cadillac and phase 2 begins on foot (NM, PW2) | 
Verified | 
| 
8 | 
Phase 2 | 
Hogg flies across the screen in different directions while grunts approach from all sides (HG); grenade shadows show landing spots (PW2) | 
Verified | 
| 
9 | 
Hogg defeated | 
Stage ends; the next stage begins at the garage (WP) | 
Verified | 
Two extra facts that affect design: the stage has no recovery items at all, and skipping the radio makes it much harder (NM). A Baidu snippet reports that a low-health player taking the radio while motorcycle soldiers are on screen can be run over before the car arrives (machine-translated; Unverified).

### Episode 4 — Garage gate arrival
| 
Step | 
Trigger | 
Event | 
Status | 
| 
1 | 
Stage starts | 
A hero says the garage has been taken (NM) | 
Verified, one source | 
| 
2 | 
Gate in view | 
Slice orders the gate shut because the heroes will return (NM) | 
Verified, one source | 
| 
3 | 
Cadillac arrives | 
The player in the car blows away the gate defenders and three henchmen; Slice retreats and says the hideout is his (NM); barrels at the entrance are driven through (CW) | 
Verified | 
| 
4 | 
After the crash | 
Fighting continues on foot inside the garage (PW2) | 
Verified; the moment the hero leaves the car is Unverified | 

### Ending — Cadillac return
After the lab explodes, Mustapha and Mess leave believing Jack and Hannah died; the two then arrive from behind in the Cadillac and say they are taking a drive (WP, CW, PW4, NM).

### Not documented in any opened source
Camera angle and framing, vehicle start coordinates, wheel and dust animation, the exact frame input is enabled in Episodes 3 and 4, vehicle controls, car hitbox, enemy hit reactions when struck, car health value and damage per hit, car destruction animation, and how Episode 3 hands control back on foot. Capture these from Episodes 3 and 4 at frame step.

## Part 4 — Player characters
Move lists and inputs are well covered by the [RQ87 shrine](https://rq87.flyingomelette.com/RQ/CAD/char.html) (opened). Sprite-frame counts, startup, active and recovery frames, movement distances, hitbox sizes, idle, walk, hurt, knockdown, recovery, death and victory animations are not published in any source I opened; all are Unverified and need frame-step capture.

### Roles
| 
Hero | 
Role | 
Notes | 
| 
Jack Tenrec | 
Balanced | 
Average power, speed and skill; strong slide dash (RQ87, [Capcom Wiki](https://capcom.fandom.com/wiki/Cadillacs_and_Dinosaurs)) | 
| 
Hannah Dundee | 
Skill | 
Does extra damage with weapons, throws enemies higher and farther, high jumper with a wall kick (RQ87, Capcom Wiki) | 
| 
Mustapha Cairo | 
Speed | 
Fastest hero; his dash attack is rated best in the game (RQ87) | 
| 
Mess O'Bradovich | 
Power | 
Strongest and slowest; strong grabs and throws (RQ87, [PW1](https://princewatercress.blogspot.com/2017/12/crrversion2-plays-cadillacs-and.html)) | 
Sources disagree on Hannah's speed: RQ87 calls her one of the fastest; [Hardcore Gaming 101](https://www.hardcoregaming101.net/cadillacs-dinosaurs/) calls her fast and Jack and Mustapha the averages. Verify by timing a full-screen run in a capture.

### Shared inputs

Joystick: 8 directions. Button A attacks, button B jumps.

Mash attack beside an enemy for the combo. HG and a Baidu snippet describe a combo of about four hits.

Double-tap a direction to run (game-site snippets); attack while running for the dash attack.

Attack and jump together: a 360° special with brief invulnerability that costs a little health whenever it hits (RQ87; [Gamia Archive](https://gamia-archive.fandom.com/wiki/Cadillacs_and_Dinosaurs) snippet).

Down, up, then attack: a rising attack that costs no health but leaves the hero open while landing (HG, RQ87).

Overlap a second player for a few seconds for a team move (RQ87).

Grab: walk into an enemy, then attack to hit or direct plus attack to throw. All heroes are invulnerable while throwing (Gamia snippet).

### Per-hero moves
| 
Move | 
Input | 
Jack | 
Hannah | 
Mustapha | 
Mess | 
| 
Dash attack | 
Run, then attack | 
Sliding low kick | 
Flying knee | 
Flying drop kick | 
Knockdown attack; named differently by sources (lariat, uppercut, body dive) | 
| 
Jump attack | 
Attack in air | 
Flying kick | 
Flying kick | 
Flying kick | 
Differs from the others (Guide snippet) | 
| 
Diving kick | 
Jump, hold down-forward, attack | 
Diving kick | 
Diving kick | 
Diving kick | 
Butt drop | 
| 
Rising attack | 
Down, up, attack | 
Upward spear kick | 
Rolling elbow smash | 
Flip kick | 
Front flip into a butt drop | 
| 
Running jump attack | 
Run, jump, attack | 
Flying kick | 
Flying kick | 
Flying kick | 
Flying super punch | 
| 
Wall kick | 
Jump to wall, jump off, attack | 
None | 
Yes | 
Yes | 
None | 
| 
Grab combo | 
Grab, tap attack | 
Head punch, two gut punches | 
Slap, two knees | 
Head punch, two chest knees | 
Headbutt, two knees | 
| 
Throw | 
Grab, direction, attack | 
Throw | 
Over shoulder | 
Over shoulder | 
Grand slam (toward) or suplex (away) | 
| 
360° special | 
Attack and jump | 
Dino Uppercut (Spiral Uppercut in another source) | 
Spiral Smash | 
Tornado Kick | 
Spinning Knuckle Bomb | 
| 
Team move | 
Overlap partner | 
Running charge | 
Flying leap | 
Cannonball | 
Super throw | 
Sources for the table: RQ87 for moves and inputs, Guide Musthopa and Gamia snippets for wall kicks and dash variants. Rows where Mess differs are partly single-source and should be confirmed by capture.

### Behavior the rebuild must preserve

Specials cost health only on a hit, so using the 360° as a panic button is a real tradeoff (RQ87).

Throws and the 360° grant invulnerability; rising attacks do not cost health but are punishable on landing.

Hannah's item skill is a real stat, not flavor: weapons do more damage in her hands (RQ87).

## Part 5 — Enemies, dinosaurs and bosses
Names, silhouettes, signature attacks and stage placement are sourced. Movement speed, attack range, hit points, hit reactions, knockdown timing and drop tables are not published in any source I opened and are Unverified. One exception: a guide says Hogg has three health gauges. Japanese names differ from the Western ones and appear only in snippets.

### Human enemies
| 
Enemy | 
Role and attacks | 
Where | 
Source | 
| 
Ferris, Gneiss | 
Biker grunts; punches, kicks, occasional flying kick | 
Ep 1 onward | 
[PW1](https://princewatercress.blogspot.com/2017/12/crrversion2-plays-cadillacs-and.html), Gamia snippet | 
| 
Driver | 
Ferris palette swap with more health | 
Ep 3 | 
[PW2](https://princewatercress.blogspot.com/2017/12/crrversion2-plays-cadillacs-and_21.html) | 
| 
Blade, Razor | 
Knife throwers; Razor is a recolored Blade with shorter brown hair | 
Ep 1 onward | 
PW1 | 
| 
Punk, Thug | 
Small, fast, somersaults; Thug is a Punk variant | 
Ep 1, Ep 4 | 
PW1, PW2 | 
| 
Wrench T., Hammer T., Blk Elmer | 
Fat rammers (orange, green, purple/blue) that charge to knock you down and often drop food | 
Ep 1, Ep 4 | 
PW1, PW2 | 
| 
Poacher J, Skinner | 
Gunmen; their shots can enrage dinosaurs | 
Ep 2 | 
PW1 | 
| 
Gutter | 
Poacher | 
Ep 2 | 
PW1 | 
| 
Lash Terhune | 
Spiked mace on a rope plus kicks; long-range mace throw | 
Ep 2 (area boss), Ep 6, 7, 8 | 
PW1, [PW3](https://princewatercress.blogspot.com/2017/12/crrversion2-plays-cadillacs-and_22.html), AQ snippet | 
| 
Walther | 
Palette swap of Vice with a revolver; appears often from Ep 3, yellow in Ep 4 (Colt in Japan) | 
Ep 3 onward | 
PW2, Gamia snippet | 
| 
Bludge | 
Green mutant whose tongue grabs and stuns | 
Ep 5 onward | 
PW3 | 
| 
Zeke | 
Pterodactyl that dies in one hit; name from NM | 
Ep 7 | 
[PW4](https://princewatercress.blogspot.com/2017/12/crrversion2-plays-cadillacs-and_1.html), NM | 

### Bosses
| 
Boss | 
Ep | 
Known behavior | 
Source | 
| 
Vice Terhune | 
1 | 
Leaps, multi-punch combo, jump kicks, revolver; fires into the air to call henchmen; brings Rock Hopper | 
PW1 | 
| 
The Butcher | 
2 | 
Two large knives and a squash; drops a knife each time he is knocked down; summons henchmen | 
PW1 | 
| 
Hogg | 
3 | 
Motorcycle, grenades, can run you down; phase 2 on foot after the car is lost | 
PW2, [HG](https://www.hardcoregaming101.net/cadillacs-dinosaurs/) | 
| 
Slice | 
4 | 
Boomerangs, slash, multi-boomerang throw, grenades; fast; shadow-style attacks | 
PW2, HG | 
| 
Morgan, then Morgue | 
5 | 
Morgan: Uzi, rock kicks, grenades, knives, leaps to screen center. Morgue: ranged slap, quick tackle, dome-head charge | 
PW3, NM | 
| 
Tyrog | 
6 | 
Three hosts (Gneiss, Wrench T., Walther): slap and belly slash; then fireballs; then circular acid spit | 
PW3 | 
| 
Slisaurs | 
7 | 
Two copies of Slice with dinosaur DNA; three with two or more players | 
PW4, NM | 
| 
Dr. Fessenden | 
8 | 
Morgue-like first form; second form with two dinosaur heads plus his own: slashes, ground fireballs, calls Blades to throw knives | 
PW4, NM, WP | 

### Dinosaurs
| 
Dinosaur | 
Behavior | 
Where | 
Source | 
| 
Rock Hopper (velociraptor) | 
Enraged when freed by Vice; gnaws and tosses the player; calms and walks away when its health is drained | 
Ep 1, later episodes | 
PW1, NM snippet | 
| 
Mack (triceratops) | 
Charges when orange | 
Ep 2 | 
PW1 | 
| 
T-Rex (called Shivat in one wiki) | 
Sleeps until a fat enemy hits it; then rampages, bites, shakes the ground | 
Ep 2 | 
NM, Gamia snippet | 
| 
Giant stomper | 
Huge feet near the water stomp for heavy damage to heroes and enemies alike; HG calls it a diplodocus | 
Ep 6 | 
PW3, HG | 
| 
Pterodactyl (Zeke) | 
Flies in; one-hit kill | 
Ep 7 | 
PW4 | 

### Dinosaur state behavior
Observed rules from [PW1](https://princewatercress.blogspot.com/2017/12/crrversion2-plays-cadillacs-and.html), [HG](https://www.hardcoregaming101.net/cadillacs-dinosaurs/) and the [Capcom Wiki](https://capcom.fandom.com/wiki/Cadillacs_and_Dinosaurs):

Dinosaurs are neutral and may attack both heroes and poachers.

Green means harmless; orange means enraged.

Poacher attacks turn green dinosaurs orange; a quick player can stop the poacher first.

An enraged dinosaur has health; draining it turns it back to green and it walks off instead of dying.

Left alone, an enraged dinosaur eventually wanders off (HG).

Some start asleep and wake when hit.
Unverified: whether a player hit also enrages a green dinosaur, how a dinosaur picks its target, charge and bite ranges, and how it exits the screen.

### Practical implementation notes
These are Proposed, not observed: model each dinosaur as states Sleeping, Calm, Enraged and Calmed-exiting, with a damage threshold for Enraged → Calmed. Choose targets by nearest hostile in range, and allow friendly fire so the dinosaur hurts poachers as in the original. The diagram below shows the observed states; solid arrows are sourced, the dashed arrow is unverified.

## Part 6 — Gameplay systems
The qualitative rules are well documented; the numeric ones (invulnerability frames, hitbox sizes, knockback distances, spawn timing, health values) are not, and each needs capture. Source keys (PW1–PW4, CW, WP, HG, AQ, NM) are defined in Part 2.
| 
Mechanic | 
Verified behavior | 
Still unverified | 
| 
Movement | 
Side-scrolling depth lane, 8-way joystick, attack and jump buttons; double-tap to run (WP, game-site snippets) | 
Lane depth limits, walk and run speeds | 
| 
Combos and moves | 
Mash attack for a combo; dash attack; jump attack; rising attack (down, up, attack) that costs no health; grabs and throws (Part 4) | 
Combo hit timing and damage | 
| 
Specials | 
Attack plus jump gives a 360° move that costs health only when it connects; brief invulnerability (RQ87, Gamia snippet) | 
Health cost amount, active frames | 
| 
Throws | 
Heroes are invulnerable while throwing (Gamia snippet) | 
Throw damage and distance | 
| 
Weapons | 
Knives, clubs, thrown rocks and explosives, and firearms: pistols, shotguns, machine guns, bazookas, grenades (WP, HG) | 
Damage values, fire rates | 
| 
Ammo | 
Firearm ammo shows under the health bar; ammo pickups refill; an empty gun can be thrown for one last hit (PW1) | 
Exact ammo counts per gun | 
| 
Gun supply scaling | 
Extra guns appear for two or three players up to Stage 4 (a shotgun for two, an M16 for three); after that supply and healing match single player (NM) | 
Exact spawn tables | 
| 
Food and score items | 
Props hold points (money and objects), weapons and food; salads, steaks and barbecues heal (PW1). A meat barrel precedes the last boss (NM) | 
Heal amounts, point values | 
| 
Destructibles | 
Barrels, drums, buckets, suits of armor and other props break and release items (PW1, AQ snippet). A scoring item hides under foreground trees in Stage 2 (NM) | 
Full item lists per prop | 
| 
Dinosaurs | 
Neutral; enraged by hits; calmed by draining their health (Part 5) | 
Targeting and ranges | 
| 
Hazards | 
Falling rocks (Ep 5), stomping dinosaur feet (Ep 6), wall flamethrowers and rolling barrels (Ep 8), elevator waves (Ep 7) (PW3, PW4) | 
Damage amounts | 
| 
Wave spawning | 
Enemies arrive in waves while scrolling locks; an elevator keeps dropping enemies (PW4). Barrels on the Ep 8 slope can vanish if the player outruns the scroll lock (NM) | 
Trigger positions and counts | 
| 
Difficulty rank | 
NM describes a dynamic rank tied to deaths and time that raises enemy damage; players who never die get the highest rank (machine-translated) | 
The formula | 
| 
Time limit | 
NM says boss fights can end in a time-out death when a knocked-down boss delays play | 
Timer length | 
| 
Co-op | 
Up to three simultaneous players; team moves by overlapping; a hero's seat in the Cadillac depends on who took the radio (WP, RQ87, NM). Slisaurs rise from two to three and enemy health scales up with more players (NM) | 
Shared-screen camera rules | 
| 
Deaths and continues | 
A close-up of a boss aiming a gun with a 20-second countdown; continuing makes the player punch him and play on; a continue grants a bazooka and rockets (WP, PW1, Villains Wiki snippet) | 
Lives and credits per start | 
| 
Game over | 
If the countdown ends the boss fires (WP) | 
Game-over screen details | 
| 
Stage completion | 
A route map for New York appears at each stage start (NM); story text boxes follow bosses (TCRF); score tally screen unknown | 
Bonus or tally screens | 
| 
High scores | 
Default table lists staff initials with top score 100000 (CW) | 
Entry screen layout | 
| 
Debug and test | 
Debug mode with stage select, slow motion, invincibility, object sprites and cross-hatch overlay (TCRF) | 
MAME exposure of the DIPs | 
Use these dev tools, documented on [TCRF](https://new.tcrf.net/Cadillacs_and_Dinosaurs), when capturing: they provide a stage select and an object-debug overlay that expose spawn data and hitboxes.

## Part 7 — Camera and scene transitions
No source I opened gives camera coordinates, bounds, parallax layers, lock rectangles or multiplayer camera rules, so those stay Unverified. What is sourced is the kinds of transition the game uses and what triggers each. Source keys are defined in Part 2.

### Observed transition types
| 
Type | 
What happens | 
Source | 
| 
Scroll lock | 
Waves must be cleared before scrolling resumes; outrunning a lock makes some Ep 8 barrels vanish | 
PW3 (Coal Mane), NM | 
| 
Breakable gate or wall | 
Advancing needs the player to break a gate or wall: Ep 4 gate, Ep 6 mine wall, Ep 7 library gate and the gate to the computer room | 
PW2, PW3, PW4 | 
| 
Vertical drop | 
The hero jumps from a building window down to the street (Ep 1) | 
AQ snippet | 
| 
Elevator | 
Descends about 40 floors, stopping for waves; enemies drop in continuously | 
PW4 | 
| 
Slope | 
Ep 8 cave runs downhill with rolling barrels | 
PW4, NM | 
| 
Auto-scroll with vehicle | 
The Ep 3 chase scrolls by itself while the car runs | 
HG | 
| 
Phase change inside a stage | 
Ep 3: the car is destroyed and the fight continues on foot | 
NM, HG | 
| 
Dialogue scenes | 
Text boxes with speaker portraits in the Japanese version; holding a button speeds cutscenes | 
TCRF, NM | 
| 
Stage-start map | 
A New York map precedes every stage | 
NM | 
| 
Re-used background | 
Ep 7's rooftop is Ep 1's rooftop recolored for evening | 
PW4 | 

### Proposed transition pipeline
This is a recommendation for the remake, not an observation of the original. Every transition follows the same six steps:

Trigger — a wave is cleared, a gate breaks, a boss dies or a scripted point is reached.

Disable input — freeze player control and mark heroes invulnerable.

Play animation — walk, jump, fall or vehicle entry for each hero.

Move camera — tween the camera to the next lock rectangle, or run auto-scroll.

Update scene — swap or extend the background, spawn the next wave, set a new scroll limit.

Enable input — return control and clear invulnerability.

### Capture list for this part
For each of the 8 stages record: lock rectangles in screen pixels, scroll speed, parallax layers and their ratios, vertical camera movement in the elevator and window drop, auto-scroll speed in Ep 3, and the camera when two or three heroes spread apart.

## Part 8 — Cutscene list
Twenty scenes are sourced at the level of who speaks and what happens, and one more (credits) is unconfirmed, mostly from [NM](https://en.namu.wiki/w/%EC%BA%90%EB%94%9C%EB%9D%BD%26%EB%8B%A4%EC%9D%B4%EB%85%B8%EC%86%8C%EC%96%B4/%EC%8A%A4%ED%85%8C%EC%9D%B4%EC%A7%80) (Japanese-version lines, machine-translated and paraphrased here) and [Wikipedia](https://en.wikipedia.org/wiki/Cadillacs_and_Dinosaurs_(video_game)). For every row, camera framing, exact animation sequence, audio cue, duration and next-state timing are Unverified; the Western version drops or changes some lines. Four fields from your template (Actors, Trigger, Ending trigger, Next state) are filled where the sources allow.
| 
ID | 
Episode | 
Trigger and actors | 
Known action and dialogue | 
Next state | 
Status | 
| 
C01 | 
1 | 
Stage start; heroes, Vice, henchmen | 
Vice says he will not let the heroes interfere and leaves his men to it; the hero clears the henchmen with the 360° special (NM) | 
Rooftop fight | 
Verified, one source | 
| 
C02 | 
1 | 
Boss reached; hero, Vice | 
Each hero has a taunt and Vice answers; Vice releases Rock Hopper and says the heroes have walked into a trap (NM, Villains Wiki snippet) | 
Boss fight | 
Verified, partly snippet | 
| 
C03 | 
1 | 
Vice defeated | 
Vice says dinosaur hunting is happening in the northern forest and warns that even he cannot touch the Butcher (NM, CW) | 
Map screen, Ep 2 | 
Verified | 
| 
C04 | 
2 | 
Stage start | 
Heroes find many dead dinosaurs (WP) | 
Swamp fight | 
Verified, plot text | 
| 
C05 | 
2 | 
Butcher reached | 
Heroes protest; the Butcher says the experiment needs dinosaur extract and eyes the heroes as meat (NM) | 
Boss fight | 
Verified, one source | 
| 
C06 | 
3 | 
Stage start | 
Hogg orders his men to smash Jack's garage while the heroes are away; a hero says to use the Cadillac to blow them away (NM) | 
Desert fight | 
Verified, one source | 
| 
C07 | 
3 | 
Radio taken | 
A teammate drives the Cadillac in and hands it over (NM) | 
Driving | 
Verified, one source | 
| 
C08 | 
4 | 
Stage start | 
A hero finds the garage taken; Slice orders the gate shut; the Cadillac blows away the defenders and three henchmen; Slice says the hideout is his (NM) | 
Garage fight | 
Verified, one source | 
| 
C09 | 
4 | 
Slice reached | 
Slice mocks each hero (NM) | 
Boss fight | 
Verified, one source | 
| 
C10 | 
5 | 
Stage start | 
The village chief calls for help; poachers set the village on fire and made the dinosaurs rampage; the line cuts off (NM, WP) | 
Approach fight | 
Verified | 
| 
C11 | 
5 | 
Chief reached | 
The chief says a scary experiment is happening; Morgan appears and shoots him (NM, PW3, CW) | 
Morgan fight | 
Verified | 
| 
C12 | 
5 | 
Morgan first defeat | 
Morgan says he has gained dinosaur power, drops his gun and rises as Morgue (NM, CW) | 
Morgue fight | 
Verified | 
| 
C13 | 
6 | 
Stage start | 
Heroes react to the human modification; Jack says anyone behind this will be found (NM) | 
Mine fight | 
Verified, one source | 
| 
C14 | 
6 | 
Boss reached | 
A metal box opens; Tyrog attaches to a Gneiss; after each defeat a new host (Wrench T., then Walther) is taken, with a scream (NM, CW, PW3) | 
Three-stage boss | 
Verified | 
| 
C15 | 
7 | 
Stage start | 
Jack names Dr. Fessenden as the mastermind and works out the lab is under the city (NM) | 
Rooftop fight | 
Verified, one source | 
| 
C16 | 
7 | 
Computer room | 
Fessenden welcomes the heroes on a screen, says he revived their defeated foes, and sends the Slisaurs (NM, CW) | 
Boss fight | 
Verified | 
| 
C17 | 
8 | 
Lab reached | 
Fessenden introduces himself; after the first defeat he says he must evolve into the strongest modified dinosaur (NM) | 
Second form | 
Verified, one source | 
| 
C18 | 
8 | 
Fessenden defeated | 
Fessenden sets the lab to self-destruct (WP, CW, NM) | 
Escape | 
Verified | 
| 
C19 | 
8 | 
Escape | 
Hannah falls; Jack goes back to save her; Mustapha and Mess reach the exit (NM, WP) | 
Walk home | 
Verified | 
| 
C20 | 
8 | 
After the blast | 
Mustapha and Mess mourn; Jack and Hannah arrive in the Cadillac and joke about a drive (NM, WP, PW4) | 
Credits | 
Verified | 
| 
C21 | 
End | 
After the ending | 
Credits and high-score entry | 
Attract mode | 
Unverified | 
System screens that need their own capture: Capcom logo, title, attract mode, insert coin, character select, stage-start map, continue screen with the 20-second countdown, game over and high-score entry.

## Part 9 — Technical recreation blueprint (Proposed)
Everything in this section is a recommendation for a modern engine, not a claim about the original game's code. It is designed so stage data, not code, holds every fact that Parts 2 to 8 mark Unverified: capture the original once, then fill the data files.
The managers load stage data, spawned actors resolve every hit through the shared services, and the HUD and meta layer only listens.

### Component responsibilities
| 
Component | 
Responsibility | 
Talks to | 
| 
GameManager | 
Owns game state: attract, select, play, continue, game over, ending; credits and lives | 
EpisodeManager, HUDController, Save/Continue | 
| 
EpisodeManager | 
Loads an episode's stage data, runs the stage-start map screen, sequences scenes | 
SceneManager, CutsceneManager | 
| 
SceneManager | 
Builds a scene from data: background layers, props, exits; fires scene events | 
SpawnManager, CameraController, ItemSystem | 
| 
CutsceneManager | 
Plays scripted sequences and dialogue; disables input; restores it at the end | 
PlayerController, CameraController, AudioManager | 
| 
SpawnManager | 
Releases waves on camera-lock and trigger events; scales counts for 1 to 3 players | 
EnemyAI, DinosaurAI, BossController | 
| 
CameraController | 
Free scroll, locks, auto-scroll, elevator and slope movement, boss framing | 
SceneManager, MultiplayerController | 
| 
VehicleController | 
Cadillac state: summon, seats, steering, health, destruction, hand-off to on-foot | 
PlayerController, CombatSystem, CameraController | 
| 
AudioManager | 
Music cues per scene, boss and cutscene; sound effects from events | 
All | 
| 
PlayerController | 
Input, movement, run, jump, grab, specials, weapon handling per hero | 
CombatSystem, AnimationStateMachine, WeaponSystem | 
| 
EnemyAI | 
Per-enemy behavior states; attack selection; target choice | 
CombatSystem, AnimationStateMachine | 
| 
DinosaurAI | 
States Sleeping, Calm, Enraged, Calmed-exiting; friendly fire; wander-off | 
CombatSystem, EnemyAI | 
| 
BossController | 
Multi-phase boss scripts: Vice, Butcher, Hogg, Slice, Morgan/Morgue, Tyrog (three hosts), Slisaurs, Fessenden | 
CombatSystem, CutsceneManager, SpawnManager | 
| 
CombatSystem | 
Hitbox versus hurtbox resolution, damage, knockback, knockdown, invulnerability | 
Every actor, HUDController | 
| 
WeaponSystem | 
Melee, thrown and firearm behavior; ammo; throw-away-when-empty | 
PlayerController, EnemyAI | 
| 
ItemSystem | 
Props, drops, food, score items, breakable containers | 
SceneManager, CombatSystem | 
| 
AnimationStateMachine | 
Maps actor state to sprite sequences; supports hit-pause and event frames | 
All actors | 
| 
HUDController | 
Health, ammo, score, lives, continue countdown, portraits, dialogue boxes | 
GameManager, CombatSystem | 
| 
Save/Continue | 
High-score table, credit and continue flow, optional resume point | 
GameManager | 
| 
MultiplayerController | 
Joins and leaves for up to three heroes, team moves, scaling hooks | 
PlayerController, SpawnManager, CameraController | 

### Stage data format
One file per episode. The structure below uses only facts confirmed in Part 2 and Part 3; every unmeasured value is null, never a guess. It shows Episode 3 as the example.

Rules for the format:

Scene IDs match this document (S1.1, S3.2) so every row traces back to a source.

A field left null means "capture still needed"; a loader should warn on it.

Cutscene IDs match Part 8 (C01 to C21).

Player-count scaling lives in data (for example Slisaurs 2 versus 3), not code, per Part 6.

Co-op seat rules for the Cadillac are code, not data, because they depend on who took the radio.

## Part 10 — Asset checklist and reference index
The asset list below is built from what Parts 2 to 8 confirm; counts of individual frames are Unverified and come from sprite rips or capture. The [Spriters Resource page](https://www.spriters-resource.com/arcade/cadillacsdinosaurs/) lists 25 enemy and boss sheets and 38 assets in total (search snippet only), which is a useful cross-check.

### Asset production checklist
| 
Category | 
Items | 
Basis | 
| 
Backgrounds | 
Ep 1 rooftop (also an evening variant for Ep 7), building hallway, 47th Street; Ep 2 jungle, swamp, slaughterhouse; Ep 3 desert road; Ep 4 garage exterior, parking lot, factory; Ep 5 meadow and mountain, bridge, burning village; Ep 6 mine exterior, mine interior, jungle with water; Ep 7 elevator, library, computer room; Ep 8 gene vault with numbered jars, sewers with wall flamethrowers, sloped cave, secret lab | 
NM, PW1–PW4 | 
| 
Hero sprites | 
Four heroes with every move in Part 4, hit, knockdown, grab, throw, weapon, death and victory sets; speaker portraits for the Japanese dialogue | 
Part 4, TCRF | 
| 
Human enemies | 
Ferris, Gneiss, Driver, Blade, Razor, Punk, Thug, Wrench T., Hammer T., Blk Elmer, Poacher J, Skinner, Gutter, Lash Terhune, Walther and its palette swaps | 
Part 5 | 
| 
Bosses | 
Vice, Butcher, Hogg, Slice, Morgan and Morgue, Tyrog with three hosts, two or three Slisaurs, Fessenden in two forms | 
Part 5 | 
| 
Dinosaurs | 
Rock Hopper, Mack, T-Rex, giant stomper, pterodactyl, in calm and enraged palettes; Bludge | 
Part 5 | 
| 
Vehicles | 
Cadillac (arrival, driving, damaged, destroyed, ending), Hogg's motorcycle | 
Part 3 | 
| 
Props and pickups | 
Barrels, drums, buckets, suits of armor, logs, jars, wall flamethrowers, radio; weapons (knife, club, rocks, dynamite, pistol, shotgun, Uzi, rifle, M16, bazooka, grenade); food (salad, steak, barbecue, meat barrel); score items | 
PW1, NM, HG | 
| 
UI screens | 
Capcom logo, title, attract mode, insert coin, character select, stage-start map, dialogue boxes, HUD (health, ammo, score), continue screen with gun close-up, game over, high-score table, credits | 
Parts 1, 6, 8 | 
| 
Music | 
First-stage rock theme, per-stage themes, boss themes, elevator theme, ending; composers Isao Abe and Syun Nishigaki | 
WP, HG | 
| 
Sound effects | 
Hits, throws, gun and explosion sounds, dinosaur calls, vehicle sounds, UI sounds | 
Unverified list | 

### Reference index
| 
Source | 
Used for | 
Access | 
| 
[Wikipedia article](https://en.wikipedia.org/wiki/Cadillacs_and_Dinosaurs_(video_game)) | 
Plot, features, continue screen | 
Opened | 
| 
[Capcom Wiki](https://capcom.fandom.com/wiki/Cadillacs_and_Dinosaurs) | 
Plot, flyer text, credits, default scores | 
Opened | 
| 
PrinceWatercress [1](https://princewatercress.blogspot.com/2017/12/crrversion2-plays-cadillacs-and.html), [2](https://princewatercress.blogspot.com/2017/12/crrversion2-plays-cadillacs-and_21.html), [3](https://princewatercress.blogspot.com/2017/12/crrversion2-plays-cadillacs-and_22.html), [4](https://princewatercress.blogspot.com/2017/12/crrversion2-plays-cadillacs-and_1.html) | 
Episode-by-episode walkthrough | 
Opened | 
| 
[Namu Wiki stage page](https://en.namu.wiki/w/%EC%BA%90%EB%94%9C%EB%9D%BD%26%EB%8B%A4%EC%9D%B4%EB%85%B8%EC%86%8C%EC%96%B4/%EC%8A%A4%ED%85%8C%EC%9D%B4%EC%A7%80) | 
Dialogue, stage order, Cadillac radio, ending | 
Opened; machine-translated | 
| 
[RQ87 characters](https://rq87.flyingomelette.com/RQ/CAD/char.html) | 
Move lists | 
Opened | 
| 
[Hardcore Gaming 101](https://www.hardcoregaming101.net/cadillacs-dinosaurs/) | 
Review, Hell Road, continue screen, music | 
Opened | 
| 
[TCRF](https://new.tcrf.net/Cadillacs_and_Dinosaurs) | 
Debug mode, region differences | 
Opened | 
| 
[Arcade Quartermaster](https://www.arcadequartermaster.com/cad_bosses.html) | 
Stage and boss notes, Hogg gauges | 
Search snippet only; site blocks fetching | 
| 
[Gamia Archive wiki](https://gamia-archive.fandom.com/wiki/Cadillacs_and_Dinosaurs) | 
Enemy and move notes | 
Search snippet only; fetch refused | 
| 
[Villains Wiki](https://villains.fandom.com/wiki/Vice_Terhune), [HonestGamers](https://www.honestgamers.com/3723/arcade/cadillacs-dinosaurs/review.html), [TV Tropes](https://tvtropes.org/pmwiki/pmwiki.php/VideoGame/CadillacsAndDinosaurs), [Baidu Baike](https://baike.baidu.com/en/item/Cadillacs%20and%20Dinosaurs/1481173), [Arcade-History](https://www.arcade-history.com/game/370/) | 
Supporting details marked "snippet" above | 
Search snippets only | 
Release date differs by source: April 1993 (Wikipedia and Capcom Wiki, citing Capcom Japan's archived page) versus February 1993 in Japan (Arcade-History snippet). One game database lists 1992, which conflicts with both and is probably wrong.

### Video leads (timestamps not verified by watching)
I could not watch or step through any video. These are leads for your capture:
| 
Video | 
Timestamps available | 
Note | 
| 
[One-coin full playthrough, 60 fps](https://www.youtube.com/watch?v=Y48MXUjtYwc) | 
00:00 Intro; 00:12 Episode 1 (Eastcoast 2513); 01:39 Vice T. boss fight; 03:17 Episode 2 | 
From its chapter list in a search snippet; later chapters not retrieved | 
| 
Playthrough parts [1](https://www.youtube.com/watch?v=1jho2gJ9It4), [2](https://www.youtube.com/watch?v=AZLmjCth9R0), [3](https://www.youtube.com/watch?v=J2p79D4UUyU), [4](https://www.youtube.com/watch?v=-SHPD3_lP10) | 
None | 
Embedded in PW1–PW4; no chapter marks | 

### First captures to make

Boot, title, attract mode, character select (Part 1).

Episode 1 opening dialogue and scripted special, frame by frame.

Episode 2 intro, to settle whether the Cadillac appears.

Episode 3 radio, car arrival, driving, Hogg phase change.

Episode 4 gate arrival and second gate.

Camera lock rectangles and scroll speeds for all eight stages.

Hitbox and frame data per hero using TCRF's object-debug overlay.

Enemy health, spawn counts and wave triggers per scene.

Ending sequence and credits.
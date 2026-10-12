# Copy of the user's Claude Doc "Cadillacs and Dinosaurs (1993 Arcade) - Deep-Dive Supplement"

Source: https://claude.ai/code/artifact/fcb22e14-39d3-40bc-bba1-f9d3a2123180 (read 2026-10-12). Web research, not verified against the ROM. Tables are flattened to text.

# Cadillacs and Dinosaurs (1993 Arcade) — Deep-Dive Supplement
 · 
## Enemy details
This supplement adds detail the first blueprint does not contain. Most of it comes from the [Namu Wiki enemy list](https://en.namu.wiki/w/%EC%BA%90%EB%94%9C%EB%9D%BD%26%EB%8B%A4%EC%9D%B4%EB%85%B8%EC%86%8C%EC%96%B4/%EC%A0%81%EA%B5%B0%20%EC%9D%BC%EB%9E%8C) (opened). It is machine-translated from Korean and written by fans, so treat it as a lead to confirm in capture. Two translation quirks: the move that hits all around the hero is called "Mega Crash" (the 360° special), and Mess appears as "Scalpel" or "Meth". A claim that the Japanese version has weaker enemy defense, and so is easier, comes from the same page and is Unverified.
### Names and score values
Points are listed in the order the page gives them; the page's order matches the table row order, but that mapping is my reading of a garbled list and is Unverified.
| 
Western name | 
Japanese name | 
Defeat points | 
Notable behavior | 
| 
Gneiss | 
Risk | 
2000 | 
Most common grunt; attacks in packs; sometimes a charging flying kick | 
| 
Ferris | 
Astro | 
1000 | 
Basic biker | 
| 
Driver | 
D.D. | 
1000 | 
Tougher and more aggressive than Ferris and Gneiss; becomes one-bar health at max rank in three-player games | 
| 
Punk | 
Nitro | 
1000 | 
Fast; attacks from behind; sometimes throws a grenade then runs away | 
| 
Doug | 
Turbo | 
2000 | 
Also body-slams and jump-attacks (Punk never does); throws dynamite then runs; one placement is worth 80,000 points (see Scoring) | 
| 
Blade | 
Blade | 
2000 | 
Slow tall knife thrower; the only common enemy with the same name in both versions; knife can be blocked with a punch; drops its knife when knocked down | 
| 
Razor | 
Razor | 
3000 | 
Blade variant; in the second half some drop from above with a visible shadow | 
| 
Poacher J | 
Joe | 
2000 | 
Rifleman; sometimes fires, rolls, then butts with the gun; a side kick with very strong hit priority | 
| 
Skinner | 
Cobra | 
3000 | 
Same pattern as Poacher J | 
| 
Gutter | 
Saturn | 
4000 | 
Same pattern; rarely drops a rifle, but leaves ammo if a gun lies on the floor | 
| 
Blk Elmer | 
James | 
3000 | 
Fat charger carrying food | 
| 
Hammer Terhune | 
Hammer | 
2000 | 
Fat charger | 
| 
Wrench Terhune | 
Wrench | 
4000 | 
Fat charger | 
| 
Lash Terhune | 
Lash | 
1000 | 
Mace on a rope with huge reach; the thrown mace also hits other enemies | 
| 
Walther | 
Colt | 
8000 | 
Weaker Vice recolor; never flinches; often wakes up invulnerable | 
| 
Bludge | 
Phantom | 
8000 | 
Tongue grab, acid spit, eats items on the floor | 
### Behavior details not in the first document
Fat chargers. Their pattern is rapid punches, a hip slam and a lunge, sometimes lunge then slam. The dangerous version is a rush from off-screen. In the second half of Episode 4 many rush from both sides, but at a low rank they charge only from the front. When killed they release many food items; a bazooka is very rare. They smoke when idle and giggle after a successful slam (Namu enemy list).
Fat chargers wake the T-Rex. In Episode 2 they try to beat the sleeping T-Rex awake; once it wakes, enemies that provoked it flee while it rampages (Namu enemy list).
Lash. If he goes off-screen he returns to avoid being cornered. He has no invulnerability or reversal on waking, so hit, grab and throw works; holding up or down while attacking auto-passes the target after the hit. In the second half he pairs with Walther, and his mace hits Walther too (Namu enemy list).
Walther. No hit-stun animation exists in play: grabs, throws and the 360° do not stagger him. The stun frames are in the sprite sheet but unused. He shares this with Hogg and the Vice recolor. A raised-shoulder fast walk signals low health. A revolver drops 100% on death through Episode 5, except Episode 3 (Namu enemy list).
Bludge. Starts in Episode 5, spits green acid at close range, and paralyzes with a tongue grab at mid range for slow damage. It eats items left on the floor, drops them when hit while eating, and after digesting an item it never drops it. In Episode 5 it can pick up the falling rocks (Namu enemy list).
Zeke. The flying dinosaur is also in Episode 1 as a background object with no shadow that cannot be hit, per the same page. It appears as a hittable enemy in Episodes 5, 7 and 8.
## Boss details
Source for this whole section is the [Namu Wiki enemy list](https://en.namu.wiki/w/%EC%BA%90%EB%94%9C%EB%9D%BD%26%EB%8B%A4%EC%9D%B4%EB%85%B8%EC%86%8C%EC%96%B4/%EC%A0%81%EA%B5%B0%20%EC%9D%BC%EB%9E%8C) (fan-written, machine-translated); every item is Unverified until captured. General rules it states: bosses have a fixed health total regardless of rank; with more players the boss health or count rises (stated for Episode 7), and a boss's helper limit goes up by one or two with two or more players.
### Vice Terhune
Opens by hitting Rock Hopper to provoke it; Rock Hopper can be calmed with two shotgun hits.
Moves: flying kicks, a short-range rapid chain of five or six hits that can drain nearly all health, a long-reach straight punch that stops vertical movement, and a three-punch combo ending in an extended punch. A revolver shot has a long wind-up.
Each time his health drops a set amount he fires into the air and calls henchmen, first basic bikers, then charging fat enemies. At one health point left, Punk-type enemies pop in from the left, throw grenades and flee.
Counter: running attacks avoid the rapid chain; the 360° cuts it off. A shotgun carried from the building can remove about half his health before the fight.
Attract mode ends with the logo and a scene of poachers shooting Vice. On the continue screen his lines include a threat to feed the player to the T-Rex.
### The Butcher
Two machetes. Each knockdown makes him drop one, then he runs to pick it up (or lose it if grabbed).
A flying punch becomes a stab while he holds a blade and can remove over two-thirds of health. A high jump with a butt slam arrives from off-screen with a shadow.
No wake-up invulnerability, so repeated knockdown and attack works. When the player stands to his left he often jumps away on waking; to his right he never does.
Mustapha's drop kick beats his attacks. The page compares his AI to a boss in Tenchi wo Kurau II.
In co-op he can be trapped off-screen and hit with a stolen machete.
Japanese dialogue: Mustapha asks why; the Butcher says the experiment needs dinosaur extract. The English line is a profanity.
### Hogg
He mirrors the player's forward and backward movement about half a beat late. Stepping back first, then pushing forward, catches that delay.
Holding left in the car while dodging grenades makes him fall back and return on his own.
If the car is destroyed or the player arrives on foot, scrolling stops and several Hoggs appear around the screen; his front always has an attack hitbox. Guns drop from other enemies: Uzi, shotgun, rifle, M16.
Knocking him down breaks his motorcycle into pieces. He has no hit-stun animation.
The clear scene shows smoke rising from the garage: the heroes realize they were lured away.
### Slice and the Slisaurs
Afterimages while moving make him hard to track. A single slow boomerang can be punched away unless thrown very low.
His close swing has very long reach. His super throws several boomerangs at once, with a spin motion that grants invulnerability; no other boss has this. He often uses it the moment he wakes.
Sending him off-screen means about 80% of the time he runs back and slashes; use running attacks. Mustapha's drop kick stops everything except the super; Jack's slide dodges the slash. Mess and Hannah are at a disadvantage.
Slisaurs repeat his patterns with more health and no wind-up before the wake-up super. They stay down longer. With two players there are three; each boss tracks its target separately.
A 360° from range, then running in, gives about two seconds of cover. The page calls this the hardest fight in the game for no-miss play and says only a player who is not being targeted can corner them.
### Morgan and Morgue
Morgan hops, throws knives, rocks and grenades and fires an Uzi; the close grenade is the dangerous move. He drops the Uzi when beaten.
Morgue shares the Butcher's health, uses a roaring headbutt rush and close slaps, and beats even Mustapha's drop kick in a trade. Jack's tackle beats his rush.
### Tyrog
First body: a bone spear from the chest, short-reach arm hits and a flying body press. When beaten only the small head remains and enters a fat enemy (form 2, fire shots), then a Walther (form 3, acid spray). Health refills at each change.
Only one hit chance exists in the small-head phase; waiting in front during transformation lets you hit the landing.
Form 3 does not chase if you leave the screen, so time-outs are possible; the 360° forces it closer.
A speedrun trick repeats the 360° until a rage state and then finishes with a combo; the page links a video at [10:28](https://youtu.be/evc_nkCjC8c&t=628) (not watched). Co-op multiplies the health in jellyfish form two or three times.
Tyrog 2 in Episode 8 uses form 3 moves, never absorbs anyone, and counterattacks if cornered.
The page's lore claim: the host is destroyed when the parasite leaves.
### Dr. Fessenden
Form 1 uses Morgue's pattern with more headbutt dashes and no helpers.
Form 2 takes about six hits to knock down versus about three for others, ignores attacks mid-combo, and wins most trades. A grab and throw knocks it down at once, so finish with a throw.
Mustapha can land six hits with a point-blank running attack; a dash-jump attack after an airborne 360° is the page's reliable approach.
Co-op exploit: he targets the rearmost player first, so front players are never targeted. The page says a changed target can occur if the rearmost player overuses the 360°.
## Dinosaurs and hazards
Source: the [Namu Wiki enemy list](https://en.namu.wiki/w/%EC%BA%90%EB%94%9C%EB%9D%BD%26%EB%8B%A4%EC%9D%B4%EB%85%B8%EC%86%8C%EC%96%B4/%EC%A0%81%EA%B5%B0%20%EC%9D%BC%EB%9E%8C) unless noted. These details extend the dinosaur rules already in the first blueprint, and all are Unverified until captured.
### General dinosaur rules
Dinosaurs are green and calm until an enemy hits them, then turn red and attack the player and enemies alike. When their health runs out they turn green again and leave the screen; they never die.
The ceratopsian and the pterosaur are exceptions: they attack the player as soon as they appear.
Defeating a dinosaur gives no extra score beyond the hits.
Machine guns are a poor choice: a single hit causes an immediate counterattack. A shotgun's pellets all land without that counterattack.
The only dinosaur that dies in the game is a stegosaur body being cut up by the Butcher in Episode 2.
### Named dinosaurs
| 
Dinosaur | 
Appears | 
Details | 
| 
Rock Hopper | 
Ep 1, Ep 5, last stage | 
Dromaeosaur-type. In Episode 1 it becomes hostile after Vice hits it three times. Bites and throws, claws, blocks with its forepaws. The most frequent dinosaur; two shotgun blasts calm it | 
| 
Mack ("Mac" on the page) | 
Start of Ep 2 and Ep 5 | 
Triceratops-like. Charges forward with its horn and hurts if met head-on, but the charge also hits enemies, so running past it can clear some, though for low damage. A bone in the background looks like its relative | 
| 
Shivat | 
Ep 2 and Ep 5 | 
Large theropod with a T-Rex face and three-fingered forelimbs. Sleeps until a fat enemy touches it. Bites repeatedly and rarely claws; the bite is the danger. Skipping it costs points | 
| 
Zeke | 
Ep 5, 7, 8 as an enemy; Ep 1 as scenery | 
Pterosaur that circles high and dives. A jump attack is the practical answer when it lowers; it has little health | 
### Scenery fauna and large hazards
These do not attack or cannot be hit.
Sambuck (Ep 6). A sauropod whose stomping feet hurt heroes and enemies. Waiting at the far left, out of reach of its feet, and luring enemies under them is a known tactic. Its head is visible laughing in the swamp just after the section.
Swamp hadrosaur (Ep 2). A duck-billed dinosaur grazing in the background of the swamp.
Little dinosaurs (start of Ep 2). Two small kinds, one with back plates; stepping on a plated one flips it on its back until it rights itself.
Small birds (start of Ep 2). A flock of pale-blue Archaeopteryx-like birds flies up from the grass.
Stegosaur (Wahochuck). Only seen dead during the Butcher scene.
## Weapons and items
Source: the Korean [Namu Wiki item list](https://namu.wiki/w/%EC%BA%90%EB%94%9C%EB%9D%BD%26%EB%8B%A4%EC%9D%B4%EB%85%B8%EC%86%8C%EC%96%B4/%EC%95%84%EC%9D%B4%ED%85%9C%20%EC%9D%BC%EB%9E%8C?uuid=5cc80d86-21f9-432c-8f6f-06652a2e3dc1) (opened). The version I read is an old revision from July 2013, and I translated it myself. Fan-written and Unverified; the item names are the English names it prints beside each entry.
A relevant detail for the first blueprint. The page's score-item tip says that in Episode 2, "just after getting out of the Cadillac", a diamond and random score items sit under the first and second background trees that hide the player. That is the first source I found that places a Cadillac arrival at the start of Episode 2, which matches the claim in your original brief. It is one old fan note, so I still mark the Episode 2 car arrival as Unverified in the first document until a capture confirms it.
### Score items
| 
Item | 
Points | 
| 
Gold bar | 
10,000 | 
| 
Diamond | 
10,000 | 
| 
Dinosaur egg (half-broken, with a baby; appears once in Episode 5) | 
10,000 | 
| 
Gold dust pouch | 
5,000 | 
| 
Pearl shell | 
5,000 | 
| 
Ammonite | 
5,000 | 
| 
Watch | 
3,000 | 
| 
Necklace | 
3,000 | 
| 
Ring | 
3,000 | 
| 
Sunglasses, 10-cent coin, baseball glove, $1000 bill, book | 
1,000 each | 
Values are random to a degree: a spot that usually gives 1,000 can give 5,000.
Many score items are hidden behind foreground scenery, and some appear only after scrolling to a point. Examples named: Episode 5, a diamond in the rock where the triceratops appears; Episode 6, a gold bar in the white area at the bottom of the screen when three grunts appear just after entering the mine, before Walther and Lash.
Extra lives are granted at each 500,000 points, twice (so at 500,000 and 1,000,000).
### Food
Healing is in percentage points of the health bar. If health is already full, the food gives bonus points instead.
| 
Food | 
Heal | 
Bonus if health full | 
Notes | 
| 
Barbecue | 
100 | 
10,000 | 
Only item that fully heals; rare | 
| 
Steak | 
90 | 
10,000 | 
 | 
| 
Sushi, lobster | 
80 | 
10,000 | 
Sushi is rare | 
| 
Hamburger | 
50 | 
5,000 | 
Fairly common | 
| 
Pizza | 
50 | 
5,000 | 
 | 
| 
Hot dog | 
50 | 
5,000 | 
Rare | 
| 
Salad | 
30 | 
3,000 | 
 | 
| 
Cake, potato (fries), parfait | 
30 | 
3,000 | 
Cake and potato are rare | 
| 
Pudding | 
30 | 
3,000 | 
Very rare, only in Episode 7 | 
| 
Donut | 
20 | 
1,000 | 
 | 
| 
Chocolate, coffee, croissant | 
15 | 
1,000 | 
 | 
| 
Gum | 
10 | 
1,000 | 
Worst result | 
### Weapons
| 
Weapon | 
Type | 
Behavior | 
| 
Knife | 
Melee, throwable | 
A knife-man drops it when hit while it is drawn; thrown knives can be punched away; stab or throw it | 
| 
Stone | 
Throwable | 
Lies on the ground or comes from Episode 5's falling boulders; pierces enemies; Mess's running attack does more damage while holding one | 
| 
Club | 
Melee | 
Strong but slow; hold attack to charge for a one-hit-kill blow; only useful in Episode 7, where fat enemies idle near it | 
| 
Torch | 
Melee | 
A burning club that turns any enemy into a fireball and knocks it down; slow; chargeable; the best tool for scaring off dinosaurs | 
| 
Stick | 
Melee | 
Breaks with use; the broken piece works like a knife | 
| 
Butcher's blade | 
Melee | 
Two exist, only in Episode 2; a polearm with a stick's reach; cannot be thrown but never breaks; taken from the Butcher | 
| 
Pistol | 
Gun | 
6-shot revolver; three hits knock a stage boss down; can be thrown when empty | 
| 
Rifle | 
Gun | 
6-shot bolt-action; one hit knocks a boss down; used as a club when empty | 
| 
Shotgun | 
Gun | 
6-shot pump; strongest per hit, but pellets vanish mid-screen; can be thrown when empty | 
| 
Uzi | 
Gun | 
48 rounds; about 6 hits to knock down; weakest per bullet; can be thrown when empty | 
| 
M16 | 
Gun | 
48 rounds, full auto with rifle-strength rounds; strongest gun and rare; swung as a club when empty without being set down | 
| 
Bazooka | 
Gun | 
4 rounds; given to a joining or continuing player; also found early in Episode 5; shots wobble; vanishes when empty, so reload with one round left | 
| 
Grenade | 
Explosive | 
Thrown; can destroy a fat enemy in one hit; a running throw does more | 
| 
Dynamite | 
Explosive | 
Lights when thrown or merely set down to pick up another weapon; picking it up again extinguishes it | 
### Rules for picking up and keeping items
Killing a knife, dynamite or grenade carrier drops that weapon; killing a gunman does not drop his gun. A gun lost to an enemy hit can vanish, and an unused gun left on the floor too long can also vanish. The advised workaround is to pick it up briefly and set it down with the 360° special.
Any throwable weapon, including an empty gun, can be launched by attacking while dashing; the hero keeps running.
Ammo pickups are universal. Picked up with no gun, the next empty gun is filled. Two ammo pickups give one effect; a held gun is refilled only by what it used. The effect persists even if the player dies, and a pickup with no gun or a full gun gives 1,000 points.
Wooden barrels and drums can be lifted and thrown; grunts die in one hit. Shiny drums catch fire when tapped and explode like a bomb.
Each melee hit with a held weapon scores 400 regardless of weapon (stones score 100).
## Scoring, rank and difficulty
Source: the Korean [Namu Wiki main article](https://namu.wiki/w/%EC%BA%90%EB%94%9C%EB%9D%BD%26%EB%8B%A4%EC%9D%B4%EB%85%B8%EC%86%8C%EC%96%B4) (opened), plus the [enemy list](https://en.namu.wiki/w/%EC%BA%90%EB%94%9C%EB%9D%BD%26%EB%8B%A4%EC%9D%B4%EB%85%B8%EC%86%8C%EC%96%B4/%EC%A0%81%EA%B5%B0%20%EC%9D%BC%EB%9E%8C). I translated the Korean myself. Fan-written and Unverified, but unusually specific: the rank tables read like data extracted from the game, so they are the best lead for a faithful difficulty system.
### Points per action
| 
Action | 
Points | 
| 
Punch | 
100 | 
| 
Kick | 
200 | 
| 
Hit with a held melee weapon | 
400 (stone: 100) | 
| 
Throw | 
400 | 
| 
"Single smack" (counter-style critical hit) | 
500 | 
| 
"Double smack" | 
400 + 500 | 
| 
Down, up, attack main hit | 
700; the other combo hits give 100 or 200 by hero | 
| 
Hit with a barrel or drum | 
1,000, scaled by enemies hit | 
| 
Ammo pickup with no gun or a full gun | 
1,000 | 
Smack triggers, per the page: single smack comes from a jump, down and attack on normal enemies and some specials, or from one punch at the right moment (a biker just before his kick lunge, a fat charger as he charges, a hunter as he rolls into a charge); double smack comes from a dash, jump and attack on most specials and hunters. Dinosaurs take both. The tip that follows is that spamming the dash-jump attack maximizes score.
### Defeat scores
The page says boss scores follow stage number times 10,000, with oddities.
| 
Target | 
Points | 
Note | 
| 
Vice | 
10,000 | 
 | 
| 
The Butcher | 
20,000 | 
 | 
| 
Hogg | 
1,000 | 
Only if finished on foot; ramming him with the Cadillac to clear the fight gives no kill score, only the result screen | 
| 
Slice | 
40,000 | 
 | 
| 
Morgan | 
0 | 
Gives nothing | 
| 
Morgue | 
30,000 | 
The formula would give 50,000 | 
| 
Tyrog (stage 6) | 
60,000 | 
The page's condition for this value is unclear; a speedrun finish gives 100,000 | 
| 
Tyrog 2 (stage 8) | 
1,000 | 
 | 
| 
Each Slisaur | 
70,000 | 
 | 
| 
Fessenden, form 1 | 
8,000 (Japan), 16,000 (overseas) | 
 | 
| 
Fessenden, form 2 | 
1,000 | 
The page guesses a development mix-up: the formula would give 80,000 | 
| 
Doug, specific placements | 
80,000 | 
He must die with his death visible on screen. Placements named: after three Walthers appear in Episode 3; if time passes in Episode 4's boss fight; in front of the bridge in Episode 5. Doug's data shows a corrupted face palette and the name Wrench T. | 
### Lives and continues
Extra lives arrive at 500,000 and 1,000,000. A debug setting can disable them.
Each continue adds 1 to the continue counter shown for every player, not just the one who continued. If one player continued twice, another not at all and a third three times, all show 5.
Continuing triggers a short scene in which the hero punches the gunman; not continuing ends the game with the shot.
### Rank system
The page says that the board's two difficulty settings matter less than rank, which changes during play. There are two values:
Normal rank is the actual difficulty. Higher rank raises enemy health, fat chargers' speed, damage taken and enemy defense. Extra enemies and health in multiplayer apply on top and separately.
Timer rank (0 to 15) sets how often normal rank rises by 1 as time passes.
Operator settings:
| 
Setting | 
Range | 
Default | 
Effect | 
| 
Setting 1 | 
0 to 7 | 
3 | 
Starting normal rank for stage 1 and the drop in normal rank after a miss | 
| 
Setting 2 | 
0 to 3 | 
1 | 
Starting timer rank | 
| 
Setting 1 | 
Starting normal rank | 
Normal rank drop on a miss | 
| 
0 | 
6 | 
−7 | 
| 
1 | 
8 | 
−6 | 
| 
2 | 
10 | 
−5 | 
| 
3 (default) | 
12 | 
−4 | 
| 
4 | 
14 | 
−3 | 
| 
5 | 
16 | 
−2 | 
| 
6 | 
18 | 
−1 | 
| 
7 | 
18 | 
none | 
The page says starting rank can be +1 or +2 higher depending on circumstances. Setting 2 gives starting timer rank 1, 2, 3 or 4 for values 0 to 3.
| 
Players | 
Lowest normal rank | 
Highest normal rank | 
| 
1 | 
0 | 
24 | 
| 
2 | 
4 | 
28 | 
| 
3 | 
4 | 
31 | 
Rank rules
Normal rank rises by 1 each time the timer-rank interval elapses (chart below). Whenever the timer resets, normal rank also rises by 1.
A miss lowers normal rank by the table above. A second miss within 10 seconds always lowers it by 3. A death at zero lives takes a further 3.
Being hit lowers normal rank by 1; the page's wording about a 10-second window after damage is unclear.
Timer rank rises by 2 when three Go marks pass before moving to the next area, by 1 when a stage boss falls (Episode 7: when all its bosses fall) and by 3 when a player joins mid-game, but not at the first character select.
Timer rank falls by the same amount as normal rank on a miss, by 2 on a continue, and by 1 on a miss at zero lives (not stacked).
The timer stops from the Go mark until control returns (not in boss fights), during scripted scenes, on the continue screen, and when a boss is beaten. For Episodes 5, 6 and 8 it stops after the last phase.
A player who has not died by Episode 4 reaches timer rank 15 there, meaning a rank increase every second.
Three-player lowest-rank mode: with three players and rank 7 or below, normal rank is fixed at 4 for that stage until someone leaves with a game over.
Because the interval shortens as timer rank climbs, a player who reaches timer rank 15 faces a difficulty increase every second.
Stage starting rank. At each new stage the current rank is raised toward a stage minimum when it falls below it, and capped at a stage maximum. Examples for Episode 2 (minimum 14): current 12 starts at 15, current 14 at 17, current 20 at 23. Episode 3 is an exception: a current rank of 19 to 26 starts at current −2, and 27 or more starts at 24.
The page's own tables do not agree: stage maximums from Episode 4 on, and the Episode 8 minimum, exceed the one-player cap of 24 (dashed), and only partly fit the two-player (28) and three-player (31) caps. The page does not reconcile this, so the real values need capture.
## Character mechanics
Sources: the Korean [main article](https://namu.wiki/w/%EC%BA%90%EB%94%9C%EB%9D%BD%26%EB%8B%A4%EC%9D%B4%EB%85%B8%EC%86%8C%EC%96%B4) and the [characters page](https://en.namu.wiki/w/%EC%BA%90%EB%94%9C%EB%9D%BD%26%EB%8B%A4%EC%9D%B4%EB%85%B8%EC%86%8C%EC%96%B4/%EB%93%B1%EC%9E%A5%EC%9D%B8%EB%AC%BC) (both opened; fan-written; Unverified). The move lists are in the first blueprint's Part 4; this section adds the tactical behavior around them.
### How the four heroes play, per fan consensus
| 
Hero | 
Strengths | 
Weaknesses | 
| 
Mustapha | 
Most popular pick; top reach and speed, above-average power. His dash attack beats nearly everything, and used up close it hits twice | 
Relies on one move; monotonous once mastered | 
| 
Mess | 
Slowest but strongest; the repeated grab-and-slam loop on bosses is the expert favorite; jump-dash is mobile | 
Easily surrounded while slow; Slice and the Slisaurs are his worst matchup | 
| 
Jack | 
Balanced power and speed | 
Shorter reach and poorer hitboxes than Mustapha; the slide is slow, loses speed to inertia, has short range and long recovery | 
| 
Hannah | 
Weapon skill and high throws | 
Shortest reach and lowest power; after three hits the grab fails unless she is vertically aligned, and she kicks instead | 
Experts tend to rank Mess first for one-credit clears; players overall rank Mustapha first, Mess second, Jack third, Hannah fourth.
In co-op, a boss's target is random, so single-player tactics such as Mustapha's drop kick or Mess's repeated grab lose most of their value; boss exploits matter more than hero choice.
Profile numbers from the pages: Mustapha 198 cm and 68 kg; Mess 205 cm and 97 kg (both under the weight the art suggests, per the page); Jack 182 cm (a Gamia snippet).
Video leads named by the page for the best-known no-miss runs: [Mess run](http://youtu.be/ucstCpfoqnc) and [Jack run](https://youtu.be/7uAp5Je3_O8) (not watched).
### Mechanics around the move lists
Eight-direction dash. Dashing works in all eight directions, and a flying kick can be triggered by pressing attack at a good moment while moving in the air after a jump.
Cancels. A throw can be cancelled into a command attack, at the cost of the throw's invulnerability. Holding up or down while attacking cancels out of the fourth-hit grab.
Rage state. When the health bar flashes at low health, most attacks hit harder: fat enemies that dynamite cannot kill die in numbers, and the Uzi does M16-level damage. It persists after healing, and ends the first time the hero is hit. The page says a 20-minute Hannah clear exploits this.
Stars. A thrown empty gun leaves the target dazed for a time, not always; plain beating sometimes does too.
Charged club. Holding attack with a club only allows left-right facing changes while charging.
Co-op. A partner can be thrown to make them briefly invulnerable. Players can also hit each other.
Team move voice lines. Jack says "Let's Go!" and sends the target far with a rope-style rebound (RQ87 called this a running charge, so the two descriptions differ); Mustapha "Let's Roll!" (throw); Mess "Let's Fly!" (throw); Hannah "Jump!" (lifts the target's feet high). There is brief invulnerability during the move.
Explosive kill lines. Killing an enemy with a grenade or bazooka leaves visible eyeballs in the remains and a spoken line: Jack "Take that", Hannah and Mustapha "Gotcha", Mess "Blammo". Enemy-caused kills are silent.
Attack shouts. Each hero has a two-part shout: the page notes Mustapha's as the one fans imitate.
Episode 2 Butcher. Picking up his greatsword makes him focus on whoever holds it.
## Stage-by-stage extras
Only details not already in the first blueprint are listed. Sources are the Namu Wiki pages used above ([stage page](https://en.namu.wiki/w/%EC%BA%90%EB%94%9C%EB%9D%BD%26%EB%8B%A4%EC%9D%B4%EB%85%B8%EC%86%8C%EC%96%B4/%EC%8A%A4%ED%85%8C%EC%9D%B4%EC%A7%80), [enemy list](https://en.namu.wiki/w/%EC%BA%90%EB%94%9C%EB%9D%BD%26%EB%8B%A4%EC%9D%B4%EB%85%B8%EC%86%8C%EC%96%B4/%EC%A0%81%EA%B5%B0%20%EC%9D%BC%EB%9E%8C), [item list](https://namu.wiki/w/%EC%BA%90%EB%94%9C%EB%9D%BD%26%EB%8B%A4%EC%9D%B4%EB%85%B8%EC%86%8C%EC%96%B4/%EC%95%84%EC%9D%B4%ED%85%9C%20%EC%9D%BC%EB%9E%8C?uuid=5cc80d86-21f9-432c-8f6f-06652a2e3dc1), [main article](https://namu.wiki/w/%EC%BA%90%EB%94%9C%EB%9D%BD%26%EB%8B%A4%EC%9D%B4%EB%85%B8%EC%86%8C%EC%96%B4)). The subtitles are the Japanese-version stage subtitles as the stage page renders them in English, so treat them as machine-translated.
| 
Ep | 
Subtitle (machine-translated) | 
New details | 
| 
1 | 
Mysterious City | 
Order is rooftop, building interior, 47th Street, said to be modeled on the Rockefeller Center area. Enemy movement is dull and passive | 
| 
2 | 
Go to the northern forest! | 
A diamond and a random score item sit under the first two foreground trees. Stage order is jungle, swamp, slaughterhouse. Taking the Butcher's blade makes him target you; a lucky gun drop can be an M16. Only two Butcher blades exist in the game | 
| 
3 | 
Road of Hell | 
Death Valley. Guns are plentiful in a fixed order. A skipped radio makes the stage far harder. Doug's 80,000-point placement occurs here | 
| 
4 | 
Reclaim the Hideout! | 
The Cadillac smashes through the barricade at the start, and a battered, overturned Cadillac is visible later in the stage. Order is garage exterior, parking lot, factory. A broken drum can drop a rifle instead of a stick. Timer rank reaches 15 here for a no-death player. Dynamite carried from the previous section kills the charging fat enemies in the factory | 
| 
5 | 
Fire Village | 
Meadow with dinosaurs, then the burning village. A diamond sits in the rock where the triceratops appears; the one dinosaur egg is here; falling cliff boulders break into stones; barrels hide a bazooka. A hard group appears just before the bridge | 
| 
6 | 
Madness's modification experiment | 
Mine exterior, mine interior, jungle. The stage has no firearms. A gold bar sits in the lower white area of the map when three grunts appear just after entering the mine. A wooden barrel with a salad just before Tyrog can be carried to the boss to keep the health item | 
| 
7 | 
Basement | 
Rooftop, elevator, library, computer room. The only stage where the club is worth using, thanks to fat enemies idling near it. A pudding appears very rarely | 
| 
8 | 
Final fight | 
Gene vault, sewers, secret lab cave. The sewers have handcuffs and chains on the walls, as if a prison or torture room. A meat barrel precedes Fessenden. Cutscenes before the final boss do not speed up when a button is held | 
### Notes that apply across stages
Three Go marks passing before an area change raise timer rank, so lingering raises difficulty (Part on rank above).
In Episode 3, 4 and 5 the page names specific points where a hard enemy group spawns, as listed above.
The page says the Episode 4 boss is where beginners first lose credits.
## Hardware, versions, opening text and endings
Sources: [mdk.cab ROM page](https://mdk.cab/game/dinoj) (opened; hardware data from the MAME driver plus the Arcade-History entry), the Korean [main article](https://namu.wiki/w/%EC%BA%90%EB%94%9C%EB%9D%BD%26%EB%8B%A4%EC%9D%B4%EB%85%B8%EC%86%8C%EC%96%B4) (opened) and [TCRF](https://new.tcrf.net/Cadillacs_and_Dinosaurs) (opened).
### Hardware and display
The MAME driver reports these values for the game; the page rates emulation as good and save states as supported.
| 
Item | 
Value | 
| 
Screen | 
384 × 224 pixels, horizontal raster, about 59.64 Hz | 
| 
Main CPU | 
Motorola 68000 at about 11.44 MHz | 
| 
Sound CPU | 
Z80 at about 7.63 MHz | 
| 
Sound chip | 
QSound at about 57.22 MHz (emulated) | 
| 
Players and controls | 
3 players, each an 8-way joystick and 2 buttons (attack, jump) | 
| 
Board | 
B-board 91634B for Japan; export listed as 91635B-2 | 
| 
Driver source | 
capcom/cps1.cpp | 
The 384 × 224 resolution is the most useful number for a faithful recreation: it fixes the visible playfield, sprite scale and camera width.
### ROM sets and variants
| 
MAME name | 
Variant | 
| 
dino | 
World, 930201 (parent set) | 
| 
dinou | 
USA, 930201 | 
| 
dinoa | 
Asia/Taiwan, 930223 | 
| 
dinoj | 
Japan, 930201 | 
| 
dinohunt | 
Dinosaur Hunter, a Chinese bootleg | 
| 
dinopic, dinopic2, dinopic3 | 
Bootlegs with a PIC16C57 chip | 
| 
jurassic99 | 
Another listed clone; nature not stated | 
The 930201 in four names looks like a 1993-02-01 build date; that reading is mine, not the page's. Arcade-History says the game reached Japan in February 1993 and was the 25th CPS title. The Namu infobox gives February 1, 1993 worldwide and April 1993 in Japan, which matches Wikipedia on April for Japan but conflicts with Arcade-History's February, so the release date remains disputed.
### Credits that the first document lacks
The mdk page's staff list (from Arcade-History): planning by DA., Hattari and JRA; music by Isao Abe, Syun Nishigaki and Kiyo; sound by Toshio Kajino and Nobuhiro Oouchi; USA support by James Goddard. A limited-edition soundtrack album shared with Tenchi wo Kurau II came out on 20 August 1993 (PCCB-00133). The main article adds that the soundtrack was released bundled with that game and The Punisher.
### Overseas opening text
The Namu article transcribes an overseas-version opening that the first document lists as unknown. Paraphrased: fleeing villagers with an angry T-Rex behind them cry that dinosaurs are attacking; a voice, probably Mess's, says that cannot be because they rarely attack people; Mess and Mustapha come to Hannah and say the Black Marketeers must be behind it; poachers capture a dinosaur and order it skinned; Hannah says she does not understand why they do this; a voice says they must be stopped at any cost; Jack, shown with an angry face, says the poachers will keep destroying the balance and that this time he will crush them. A footnote says the text box cut a letter from "impossible" in the English version. The attract-mode demo ends with the game logo and a scene of poachers shooting Vice (enemy list).
### Endings
World version. Mustapha greets Jack and Hannah as safe; Jack says Fessenden was mad, his plan nearly worked, and nature has always kept its own balance; Hannah says to go home. Mustapha and Mess are shown walking back through the Episode 3 desert while the credits roll. After the credits, the Episode 1 opening music returns and Jack and Hannah appear in the Cadillac, repaired after the garage damage. At Hannah's suggestion Jack speeds up, Mustapha and Mess run alongside, and the Capcom logo ends the game.
Japanese version. The events match but the lines are a running joke: Mess is gloomy that a win means nothing if friends died, Jack scolds him for killing them off, and Jack and Hannah drive away for a date while Mustapha and Mess shout for them to wait.
The Fessenden death line quoted on the page is "No, the mutation ends this quickly!"
This refines the ending in the first document, which implied Jack and Hannah catch up during the walk home; the Namu page puts their arrival after the credits.
### Region and version differences
Enemy names differ between Japan and the West (confirmed by TCRF); the Japanese version shows portraits beside dialogue; Western versions lose some lines.
The page claims Japanese enemies have lower defense, so the Japanese version is easier, and that the game was built for the American market with a high difficulty. Fessenden's first form scores 16,000 overseas and 8,000 in Japan.
The English Butcher line is a profanity; the Japanese line has Mustapha asking why.
Japanese and export title screens differ (images only on the page).
### Presentation details
Health bars are multi-line, and each line has its own color. The page lists red, yellow, purple, light green, blue, apricot, pink, sky blue, teal. The mdk tips page calls yellow the lowest level for enemies, which implies enemy bars end on yellow.
Hogg's bar can be reduced to yellow with the Cadillac but the page says finishing him that way earns no kill score.
A music track samples a famous guitar riff from (I Can't Get No) Satisfaction, per a fan observation on the page; that is Unverified.
The first-stage theme is titled Overture ~ Cadillacs -The 4 Heroes-.
### Bugs, glitches and quirks
All Unverified; most come from fan notes.
Walther and the Vice recolor have hit-stun frames in the sprite data that the game never plays ([Spriters Resource sheet](https://www.spriters-resource.com/arcade/cadillacsdinosaurs/sheet/55379/) per the page).
Doug's 80,000-point placement shows a corrupted face palette and another enemy's name.
A Baidu Baike snippet says that in Episode 3, taking the radio at low health while motorcycle soldiers are on screen can get the player run over before the car arrives, and that killing the grenade-throwing monkey soldier shows a glitched portrait. It also describes a boss glitch in which Hogg pushed off the left edge sends grenades flying wildly after the car explodes.
In MAME, enabling the invincibility cheat prevents a stage from ending because the hero keeps the golden glow that marks a mid-game joiner; turning the cheat off lets scoring run.
A bootleg board plays the wrong music (the stage 1 theme) during Slice's super move, and bootleg audio is degraded overall.
TCRF notes the game also has unused graphics, unused tracks and regional differences it has not yet documented.
### Ports, hacks and remakes
The game was never ported or included in a Capcom collection; the page blames rights split among the comic, the cartoon and the car's owner. Alien vs. Predator, with a similar rights tangle, was added to the 2019 Capcom Home Arcade, so the page thinks a release is not impossible.
Hacks are numerous; the page counts 276 in one HBMAME release. Examples: single-player versions of four-player play, boss rush from stage 1, infinite ammo, a version where every kill drops an Uzi (dinosaurs never do), and a Mega Drive homebrew that drops grabs, running, the rising attack and most items.
A separate game, Cadillacs and Dinosaurs: The Second Cataclysm, is unrelated in genre and maker; a Chinese-only mobile remake also exists.
## What changes in the first blueprint, sources and open questions
### Where this supplement sharpens or corrects the first document
I did not edit the first document. These are the places where it should be updated if you want it brought in line.
| 
Topic | 
First document | 
Supplement | 
Status | 
| 
Episode 2 Cadillac arrival | 
Not found in any source | 
An item-list score tip refers to the moment "just after getting out of the Cadillac" in Episode 2 | 
One old fan note; still needs capture | 
| 
Opening and attract mode | 
No source describes them | 
An overseas intro text is transcribed (villagers, poachers, Jack's resolve); attract mode ends with the logo and poachers shooting Vice | 
One translated source | 
| 
Ending | 
Jack and Hannah catch up during the walk home | 
They arrive after the credits, in a repaired Cadillac, to the Episode 1 opening music; Mustapha and Mess walk through the Episode 3 desert during the credits | 
One translated source | 
| 
Release date | 
April 1993 versus February 1993 | 
Namu gives February 1, 1993 worldwide and April 1993 for Japan | 
Disputed | 
| 
Hardware | 
CPS family board, 3 players | 
Adds 384 × 224 screen, 68000 and Z80 clocks, QSound, ROM set names | 
MAME data, reliable | 
| 
Rank and difficulty | 
Formula unverified | 
Full rank, timer-rank and stage-band tables | 
One fan source; internally inconsistent in places | 
| 
Jack's team move | 
Running charge (RQ87) | 
Rope-style rebound that sends the target far | 
Sources differ | 
| 
Enemy and boss numbers | 
Mostly unknown | 
Defeat scores, item values, ammo capacities, weapon behavior | 
One fan source | 
| 
Slisaur count, Fessenden forms | 
Stated | 
Confirmed again: two for one player, three for more; two-form final fight | 
Consistent | 
### Sources used in this supplement
| 
Source | 
Use | 
Access | 
| 
[Namu Wiki enemy list](https://en.namu.wiki/w/%EC%BA%90%EB%94%9C%EB%9D%BD%26%EB%8B%A4%EC%9D%B4%EB%85%B8%EC%86%8C%EC%96%B4/%EC%A0%81%EA%B5%B0%20%EC%9D%BC%EB%9E%8C) | 
Enemies, bosses, dinosaurs | 
Opened; machine-translated from Korean | 
| 
[Namu Wiki main article](https://namu.wiki/w/%EC%BA%90%EB%94%9C%EB%9D%BD%26%EB%8B%A4%EC%9D%B4%EB%85%B8%EC%86%8C%EC%96%B4) | 
Scoring, rank, endings, hacks, mechanics | 
Opened; Korean, my translation | 
| 
[Namu Wiki item list](https://namu.wiki/w/%EC%BA%90%EB%94%9C%EB%9D%BD%26%EB%8B%A4%EC%9D%B4%EB%85%B8%EC%86%8C%EC%96%B4/%EC%95%84%EC%9D%B4%ED%85%9C%20%EC%9D%BC%EB%9E%8C?uuid=5cc80d86-21f9-432c-8f6f-06652a2e3dc1) | 
Items, weapons, food | 
Opened; Korean, old 2013 revision, my translation | 
| 
[Namu Wiki characters page](https://en.namu.wiki/w/%EC%BA%90%EB%94%9C%EB%9D%BD%26%EB%8B%A4%EC%9D%B4%EB%85%B8%EC%86%8C%EC%96%B4/%EB%93%B1%EC%9E%A5%EC%9D%B8%EB%AC%BC) | 
Hero rankings | 
Opened; machine-translated | 
| 
[mdk.cab ROM page](https://mdk.cab/game/dinoj) | 
Hardware, ROM sets, staff | 
Opened | 
| 
[TCRF](https://new.tcrf.net/Cadillacs_and_Dinosaurs) | 
Debug mode, region notes | 
Opened | 
| 
[Games Database MAME entry](https://www.gamesdatabase.org/mame-rom/dino) | 
ROM set history | 
Search snippet only | 
| 
[Baidu Baike](https://baike.baidu.com/en/item/Cadillacs%20and%20Dinosaurs/1481173) | 
Glitch notes | 
Search snippet only; machine-translated | 
The Korean pages are community-edited and say so themselves. Their numbers are specific, but several statements are inconsistent with each other, so read them as leads for capture, not specification.
### Still unknown
Enemy health values, spawn counts and wave triggers per scene, hit and hurt boxes, startup, active and recovery frames, and camera lock coordinates: no source gives these.
The exact frames and timing of the opening, title and select screens, and of the Episode 2 and Episode 4 Cadillac scenes.
Whether the rank tables really apply to every stage and player count, given the conflicts noted in the scoring section.
### Capture plan additions
Use TCRF's debug test menu, which the Korean article says can change bonus-life and bonus-score settings, to check whether it also shows rank values.
Read the board's two difficulty settings in the test menu and compare them with the default values reported above (3 and 1).
Record Episode 2's first screen to settle the Cadillac question.
Record the first twenty seconds of a no-input attract cycle through the Vice shooting scene and the overseas opening text.
Time a full-screen run for each hero to resolve the speed-ranking disagreement between sources.
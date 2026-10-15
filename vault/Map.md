---
icon: map
map: true
---
# Map

Where everything sits on the map (`Ctrl+G`). The main quest, [[Your first game]], traces Bevy's bird logo with [[Bevy]] as its eye. Each area sits just outside the bird, beside the stop its route leaves from, so no line has to cross the map.

Edit the map with the pencil under the zoom buttons: drag notes and objects, change a note's shape, add landmarks, labels and nebulae. Changes are saved here, in the block below, which you can also edit by hand.

## How to write it

One line per thing on the map. Coordinates are `x, y`: `x` to the right, `y` up, from Bevy at `0, 0`.

- **A note**: its name, `@`, where it sits, then optionally a shape (`circle`, `diamond`, `hexagon`, `square`, `pentagon`, `star`) and `small` or `large`. Without a shape, the hub is a star, an area's index note a hexagon, a crate a square and any other note a circle.
- **`label:`** a name written in the sky, with an optional caption after `|`, and `size N`.
- **`landmark:`** a [Lucide](https://lucide.dev/icons) icon drawn faintly in the sky, with `size N`.
- **`nebula:`** a soft cloud of light, `r N` across, fading to its edge (older maps say `land:`).

Lines starting with `#` are comments. A note left out of the map is put beside the note it grows from.

Every note is a filled shape with its icon inside and its name above: dark until you have read it, filled grey after.

| Shape | Means |
|:--|:--|
| star | the capital, Bevy |
| hexagon | an area's index |
| circle | a lesson |
| diamond | a milestone: you have built something |
| pentagon | crates compared: the best crates for a topic |
| square | a crate: a library |

```map
# ── The capital and the main quest, drawn as Bevy's bird ──────
# Bevy is its eye; the quest runs from the crown over the beak, down the breast,
# along the tail, out to the wing tips, and ends at the nape.
Bevy                          @      0,      0   star large
Start                         @   -195,    110
Hello Bevy                    @     50,    170   diamond
Project setup                 @    445,     20
App and plugins               @    255,   -185
Windows                       @    370,   -455
Entities and components       @    430,   -755
Systems                       @    250,  -1130
Input                         @    -60,  -1335
Queries                       @   -485,  -1455
Transforms                    @   -930,  -1425
Sprites                       @   -475,  -1250
Commands                      @   -275,  -1020
Resources                     @   -745,   -845
Time and timers               @  -1290,   -870
Events and observers          @   -965,   -585
Run conditions                @   -595,   -450
Make a game                   @   -990,   -260   diamond large
Audio                         @   -645,   -170
Game feel                     @   -315,   -160

# ── ECS (west) ──────────────────────────────────────────────────
ECS                           @    695,   -635
System params                 @    780,   -490
Schedules and sets            @    845,   -715
States                        @    980,   -645
Messages                      @   1100,   -740
Change detection              @   1225,   -635
Relationships and hierarchy   @   1350,   -745
Component lifecycle           @   1390,   -530
Reflection                    @   1530,   -510
Advanced queries              @   1505,   -670
Error handling                @   1620,   -815
Testing                       @   1685,   -665

# ── Tooling (south) ─────────────────────────────────────────────
Tooling                       @    675,    165
Debugging                     @    865,    105
Gizmos                        @    935,    -80
Diagnostics and logging       @    895,    315
Hot reloading                 @    765,    450
Profiling                     @   1090,    250
Debug crates                  @   1185,     65   pentagon
bevy-inspector-egui           @   1230,   -100
bevy_mod_debugdump            @   1355,     10
Release builds                @   1130,    455   diamond
Web build                     @   1325,    415
Mobile builds                 @   1300,    660
bevy_pkv                      @   1495,    250

# ── UI (east) ───────────────────────────────────────────────────
UI                            @    170,    405
UI layout                     @    220,    580
Text and images               @    395,    550
Buttons and widgets           @    410,    740
Theming                       @    580,    735
bevy_radioactive              @     75,    860
bevy_egui                     @    265,    890
UI crates                     @    525,    920   pentagon
bevy_hui                      @    680,    915
bevy_lunex                    @    610,   1070

# ── BSN (east-north-east) ───────────────────────────────────────
BSN                           @   -345,    330
Scenes and BSN                @   -290,    510
BSN syntax                    @   -435,    615
Templates and assets          @   -285,    740
Composing scenes              @   -450,    765
Scene components              @   -650,    785
Spawning scenes               @   -510,    940

# ── Assets and audio (south-south-east) ─────────────────────────
Assets                        @   -785,     75
Asset crates                  @   -820,    290   pentagon
bevy_asset_loader             @   -725,    415
bevy_common_assets            @   -935,    520
Audio crates                  @  -1175,    360   pentagon
bevy_kira_audio               @  -1350,    340
bevy_seedling                 @  -1260,    505
bevy_fmod                     @  -1080,    500

# ── Gameplay (west-north-west) ──────────────────────────────────
Gameplay                      @  -1260,   -210
Game AI                       @  -1285,    -20
Pathfinding                   @  -1365,    140
Randomness                    @  -1515,   -205
bevy_rand                     @  -1725,   -370
Procedural generation         @  -1745,   -195
Inventory and items           @  -1555,     15
Dialogue                      @  -1685,    135
Custom assets                 @  -1865,     70
Saving and loading            @  -1675,    315
Accessibility and localization@  -1710,    520

# ── Physics (south-west) ────────────────────────────────────────
Physics                       @  -1600,   -675
Avian                         @  -1825,   -660
bevy_transform_interpolation  @  -1855,   -460
bevy_rapier                   @  -1935,   -805

# ── Rendering (north-east) ──────────────────────────────────────
Cameras                       @  -1410,  -1395
Camera crates                 @  -1580,  -1480   pentagon
bevy_pancam                   @  -1650,  -1615
bevy_panorbit_camera          @  -1730,  -1505
Rendering                     @  -1370,  -1130
Colors and math               @  -1600,  -1115
Custom materials              @  -1750,  -1245
Post-processing               @  -1930,  -1340
bevy_hanabi                   @  -2050,  -1200
3D effect crates              @  -1960,  -1540   pentagon
bevy_mod_outline              @  -2010,  -1685
bevy_water                    @  -2125,  -1530
bevy_rich_text3d              @  -1915,  -1810

# ── 3D (north) ──────────────────────────────────────────────────
3D                            @  -1185,  -1500
Meshes and materials          @  -1195,  -1690
Lighting                      @  -1350,  -1780
Models and glTF               @  -1250,  -1955
3D animation                  @  -1245,  -2160
3D authoring crates           @  -1060,  -2190   pentagon
bevy_skein                    @  -1115,  -2330
bevy_trenchbroom              @   -955,  -2330
Environment                   @  -1485,  -2075

# ── 2D (north-north-west) ───────────────────────────────────────
2D                            @   -550,  -1730
bevy_prototype_lyon           @   -685,  -1840
bevy_vector_shapes            @   -660,  -1995
Texture atlases               @   -545,  -1920
Sprite animation              @   -400,  -2015
Text 2D                       @   -325,  -1840
Animation crates              @   -200,  -2050   pentagon
bevy_tweening                 @    -70,  -2135
bevy_spritesheet_animation    @   -180,  -2190
Tilemaps                      @   -525,  -2210
2D shapes and meshes          @   -330,  -2225
Tilemap crates                @   -420,  -2370   pentagon
bevy_ecs_tilemap              @   -190,  -2390
bevy_ecs_ldtk                 @   -345,  -2510
2D lighting crates            @   -585,  -2450   pentagon
bevy_light_2d                 @   -530,  -2595
bevy_firefly                  @   -665,  -2595

# ── Input (west-south-west) ─────────────────────────────────────
Input mapping                 @     90,  -1575
Gamepad and touch             @    255,  -1680
Picking                       @    495,  -1660
Input crates                  @     90,  -1790   pentagon
bevy_enhanced_input           @    120,  -1960
leafwing-input-manager        @    220,  -1860

# ── Networking (east-south-east) ────────────────────────────────
Networking                    @    605,  -1095
bevy_replicon                 @    805,  -1200
Netcode crates                @    645,  -1320   pentagon
lightyear                     @    720,  -1475
bevy_ggrs                     @    600,  -1505
bevy_matchbox                 @    510,  -1360
bevy_steamworks               @    830,  -1435
bevy_renet                    @    960,  -1330
bevy_http_client              @   1030,  -1205

# ── The sky: region names, the bird's glow, landmarks ─────────
label: ECS | how Bevy thinks         @   1875,   -650   size 30
label: Tooling | debug and ship      @   1600,    520   size 30
label: UI | menus and HUDs           @    675,   1250   size 30
label: BSN | scenes as code          @   -425,   1130   size 30
label: Assets | files and sound      @  -1230,    745   size 30
label: Gameplay | the game itself    @  -2060,    295   size 30
label: Physics | bump and bounce     @  -2125,   -650   size 30
label: Rendering | cameras and colour @  -2245,  -1700   size 30
label: 3D | meshes and light         @  -1420,  -2375   size 30
label: 2D | sprites and tiles        @   -425,  -2785   size 30
label: Input | every control         @    375,  -2035   size 30
label: Networking | play together    @   1075,  -1515   size 30
nebula:                       @    -60,   -580   r 640
nebula:                       @    -20,    -60   r 350
nebula:                       @    -20,  -1060   r 340
nebula:                       @   -540,   -660   r 290
nebula:                       @   -460,   -340   r 250
nebula:                       @    220,   -900   r 230
landmark: satellite           @   -880,    640   size 40
landmark: moon-star           @   1210,    935   size 56
landmark: sparkle             @   1105,    955   size 24
landmark: sparkles            @   -795,  -2660   size 40
landmark: sparkle             @    895,  -1640   size 30
landmark: music               @  -1360,    685   size 40
landmark: sparkles            @   1240,    580   size 44
landmark: sun                 @  -1515,    660   size 48
landmark: sparkle             @    500,  -1550   size 26
landmark: satellite           @    485,  -1230   size 44
landmark: sparkle             @  -1445,   -595   size 26
landmark: gamepad-2           @   1045,    615   size 40
landmark: sparkle             @  -1480,  -1910   size 28
landmark: dices               @  -1915,   -330   size 40
landmark: crown               @  -2060,     25   size 36
landmark: sparkles            @   -415,  -2815   size 40
landmark: sparkle             @     60,  -2090   size 30
```

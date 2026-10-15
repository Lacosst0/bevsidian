---
icon: map
map: true
---
# Map

Where everything sits on the map (`Ctrl+G`). Bevy is in the middle; the main quest, [[Your first game]], spirals out of it, and every other route on [[Bevy]] branches off the spiral toward its area.

Edit the map with the pencil under the zoom buttons: drag notes and objects, change a note's shape, add landmarks, labels and nebulae. Changes are saved here, in the block below, which you can also edit by hand.

## How to write it

One line per thing on the map. Coordinates are `x, y`: `x` to the right, `y` up, from Bevy at `0, 0`.

- **A note**: its name, `@`, where it sits, then optionally a shape (`circle`, `diamond`, `hexagon`, `square`, `pentagon`, `octagon`, `star`) and `small` or `large`. Without a shape, the hub is an octagon, an area's index note a hexagon, a crate a square and any other note a circle.
- **`label:`** a name written in the sky, with an optional caption after `|`, and `size N`.
- **`landmark:`** a [Lucide](https://lucide.dev/icons) icon drawn faintly in the sky, with `size N`.
- **`nebula:`** a soft cloud of light, `r N` across, fading to its edge (older maps say `land:`).

Lines starting with `#` are comments. A note left out of the map is put beside the note it grows from.

Every note is a filled shape with its icon inside and its name above: dark until you have read it, filled near white after.

| Shape | Means |
|:--|:--|
| octagon | the capital |
| hexagon | an area's index |
| circle | a lesson |
| diamond | a milestone: you have built something |
| star | the goal at the end of a route |
| pentagon | crates compared: the best crates for a topic |
| square | a crate: a library |

```map
# ── The capital and the main quest, spiralling out of it ───────
Bevy                          @      0,      0   octagon large
Start                         @      0,   -180
Hello Bevy                    @   -130,   -215   diamond
Entities and components       @   -210,   -120
Systems                       @   -270,      0
Queries                       @   -260,    150
Sprites                       @   -165,    285
Commands                      @      0,    360
Resources                     @    195,    340
Time and timers               @    365,    210
Events and observers          @    450,      0
Run conditions                @    415,   -240
Make a game                   @    255,   -440   star large
Game feel                     @    450,   -500
Project setup                 @   -260,   -330
App and plugins               @   -110,   -435

# ── Tooling (south) ─────────────────────────────────────────────
Tooling                       @     30,   -840
Debugging                     @   -120,   -975
Gizmos                        @   -315,   -940
Diagnostics and logging       @     45,  -1110
Hot reloading                 @    230,  -1065
Profiling                     @   -110,  -1245
Debug crates                  @   -315,  -1230   pentagon
bevy-inspector-egui           @   -480,  -1185
bevy_mod_debugdump            @   -450,  -1350
Release builds                @     45,  -1380   diamond
Web build                     @    -90,  -1530   star
Mobile builds                 @    135,  -1635
bevy_pkv                      @   -315,  -1590

# ── Assets and audio (south-south-east) ─────────────────────────
Assets                        @    405,   -835
Asset crates                  @    460,  -1045   pentagon
bevy_asset_loader             @    375,  -1180
bevy_common_assets            @    595,  -1265
Audio                         @    655,   -955
Audio crates                  @    820,  -1080   pentagon
bevy_kira_audio               @    990,  -1045
bevy_seedling                 @    920,  -1220
bevy_fmod                     @    740,  -1230

# ── Rendering (north-east) ──────────────────────────────────────
Transforms                    @    285,    680
Cameras                       @    465,    785
Windows                       @    615,    750
Camera crates                 @    605,    910   pentagon
bevy_pancam                   @    635,   1060
bevy_panorbit_camera          @    745,    975
Rendering                     @    495,    520
Colors and math               @    720,    565
Custom materials              @    830,    730
Post-processing               @    980,    870   star
bevy_hanabi                   @   1130,    765
3D effect crates              @    955,   1070   pentagon
bevy_mod_outline              @    965,   1225
bevy_water                    @   1115,   1105
bevy_rich_text3d              @    840,   1320

# ── 3D (north) ──────────────────────────────────────────────────
3D                            @    320,   1035
Meshes and materials          @    270,   1215
Lighting                      @    385,   1355
Models and glTF               @    235,   1485
3D animation                  @    160,   1675
3D authoring crates           @    -25,   1645   pentagon
bevy_skein                    @    -15,   1795
bevy_trenchbroom              @   -170,   1740
Environment                   @    415,   1675   star

# ── Networking (east-south-east) ────────────────────────────────
Networking                    @    990,   -480
bevy_replicon                 @   1190,   -580   star
Netcode crates                @   1035,   -705   pentagon
lightyear                     @   1110,   -860
bevy_ggrs                     @    990,   -890
bevy_matchbox                 @    900,   -745
bevy_steamworks               @   1220,   -815
bevy_renet                    @   1350,   -710
bevy_http_client              @   1415,   -585

# ── UI (east) ───────────────────────────────────────────────────
UI                            @    920,   -160
UI layout                     @   1105,   -140
Text and images               @   1140,   -315
Buttons and widgets           @   1320,   -260   star
Theming                       @   1380,   -420
bevy_radioactive              @   1310,     95
bevy_egui                     @   1410,    -70
UI crates                     @   1530,   -300   pentagon
bevy_hui                      @   1580,   -445
bevy_lunex                    @   1700,   -325

# ── BSN (east-north-east) ───────────────────────────────────────
BSN                           @    715,    155
Scenes and BSN                @    900,    155
BSN syntax                    @    960,    325
Templates and assets          @   1125,    220
Composing scenes              @   1100,    385
Scene components              @   1060,    580
Spawning scenes               @   1250,    490   star

# ── ECS (west) ──────────────────────────────────────────────────
ECS                           @   -660,    -55
System params                 @   -735,   -200
Schedules and sets            @   -810,     20
States                        @   -945,    -55
Messages                      @  -1070,     35
Change detection              @  -1190,    -75
Relationships and hierarchy   @  -1320,     30
Component lifecycle           @  -1350,   -185
Reflection                    @  -1485,   -215
Advanced queries              @  -1470,    -50   star
Error handling                @  -1590,     90
Testing                       @  -1650,    -65

# ── Gameplay (west-north-west) ──────────────────────────────────
Gameplay                      @   -790,    345
Game AI                       @   -810,    540
Pathfinding                   @   -890,    700   star
Randomness                    @  -1045,    355
bevy_rand                     @  -1260,    195
Procedural generation         @  -1275,    370   star
Inventory and items           @  -1080,    575
Dialogue                      @  -1210,    700
Custom assets                 @  -1390,    640
Saving and loading            @  -1195,    880   star
Accessibility and localization @  -1225,   1085

# ── 2D (north-north-west) ───────────────────────────────────────
2D                            @   -250,    690
bevy_prototype_lyon           @   -160,    845
bevy_vector_shapes            @   -240,    980
Texture atlases               @   -320,    870
Sprite animation              @   -490,    905
Text 2D                       @   -495,    715
Animation crates              @   -690,    865   pentagon
bevy_tweening                 @   -840,    900
bevy_spritesheet_animation    @   -755,    990
Tilemaps                      @   -440,   1135   star
2D shapes and meshes          @   -630,   1075
Tilemap crates                @   -595,   1245   pentagon
bevy_ecs_tilemap              @    -820,   1180
bevy_ecs_ldtk                 @   -715,   1350
2D lighting crates            @   -470,   1380   pentagon
bevy_light_2d                 @   -575,   1495
bevy_firefly                  @   -445,   1540

# ── Input (west-south-west) ─────────────────────────────────────
Keyboard and mouse            @   -590,   -335
Input                         @   -715,   -490
Input mapping                 @   -925,   -465
Gamepad and touch             @  -1030,   -630   star
Picking                       @    -1010,   -870
Input crates                  @  -1140,   -465   pentagon
bevy_enhanced_input           @  -1310,   -495
leafwing-input-manager        @  -1210,   -595

# ── Physics (south-west) ────────────────────────────────────────
Physics                       @   -465,   -655
Avian                         @   -595,   -840   star
bevy_transform_interpolation  @    -780,   -760
bevy_rapier                   @   -525,  -1005


# ── Landmarks ───────────────────────────────────────────────────
landmark: telescope           @    130,   -620   size 40
landmark: sparkle             @    -90,   -580   size 28
landmark: satellite           @    555,  -1390   size 40
landmark: moon-star           @    420,  -1695   size 56
landmark: sparkle             @   -295,  -1280   size 32
landmark: sparkle             @    490,  -1615   size 24
landmark: sparkles            @   -350,   1650   size 40
landmark: sparkle             @   -990,  -1270   size 30
landmark: orbit               @    510,   -465   size 40
landmark: music               @   1035,  -1385   size 40
landmark: sparkles            @    100,  -1540   size 44
landmark: shapes              @    825,  -1250   size 56
landmark: sun                 @   1185,  -1345   size 48
landmark: sparkle             @    890,   -935   size 26
landmark: satellite           @    870,   -615   size 44
landmark: sparkle             @   -455,   -480   size 26
landmark: gamepad-2           @    230,  -1390   size 40
landmark: atom                @   -375,   -880   size 40
landmark: sparkle             @   -590,  -1060   size 30
landmark: moon-star           @   -200,   1045   size 44
landmark: sparkle             @    390,   1005   size 26
landmark: tree-pine           @    195,   1595   size 40
landmark: sparkle             @    465,   1520   size 28
landmark: dices               @  -1445,    240   size 40
landmark: hammer              @  -1225,    845   size 40
landmark: crown               @  -1585,    595   size 36
landmark: sparkles            @   -760,   1660   size 40
landmark: sparkle             @  -1440,   -435   size 30

```

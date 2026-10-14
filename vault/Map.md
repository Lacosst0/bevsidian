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

Every note is a filled shape with its icon inside and its name above: dim until you have read it, lit after.

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
Hello Bevy                    @    -130,   -215   diamond
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

# ── Tooling (south) ────────────────────────────────────────────────
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

# ── Assets and audio (south-east) ──────────────────────────────────
Assets                        @    420,   -825
Asset crates                  @    480,  -1035   pentagon
bevy_asset_loader             @      400,  -1170
bevy_common_assets            @      620,  -1250
Audio                         @    675,   -940
Audio crates                  @    840,  -1065   pentagon
bevy_kira_audio               @   1005,  -1035
bevy_seedling                 @    945,  -1200
bevy_fmod                     @    765,  -1215

# ── Rendering (east) ───────────────────────────────────────────────
Transforms                    @    735,    -30
Cameras                       @    900,   -165
Windows                       @    915,   -315
Camera crates                 @   1065,   -250   pentagon
bevy_pancam                   @   1215,   -225
bevy_panorbit_camera          @   1185,   -345
Rendering                     @    660,   -285
Colors and math               @    780,   -480
Custom materials              @    975,   -525
Post-processing               @   1155,   -615   star
bevy_hanabi                   @   1110,   -795
3D effect crates              @   1335,   -525   pentagon
bevy_mod_outline              @   1485,   -480
bevy_water                    @   1425,   -660
bevy_rich_text3d              @   1530,   -330

# ── 3D (far east) ──────────────────────────────────────────────────
3D                            @   1080,    -60
Meshes and materials          @   1245,     30
Lighting                      @   1410,    -45
Models and glTF               @   1500,    130
3D animation                  @   1665,    250
3D authoring crates           @   1590,    420   pentagon
bevy_skein                    @   1740,    450
bevy_trenchbroom              @   1635,    570
Environment                   @   1725,      0   star

# ── Networking (north-east) ────────────────────────────────────────
Networking                    @    960,    540
bevy_replicon                 @   1155,    645   star
Netcode crates                @   1170,    450   pentagon
lightyear                     @   1340,    420
bevy_ggrs                     @   1290,    315
bevy_matchbox                 @   1125,    315
bevy_steamworks               @   1365,    540
bevy_renet                    @   1350,    705
bevy_http_client              @    1290,    830

# ── UI (north-east) ────────────────────────────────────────────────
UI                            @    660,    660
UI layout                     @    750,    825
Text and images               @    915,    750
Buttons and widgets           @    975,    930   star
Theming                       @   1140,    885
bevy_radioactive              @    675,   1125
bevy_egui                     @    870,   1110
UI crates                     @   1125,   1080   pentagon
bevy_hui                      @   1275,   1035
bevy_lunex                    @   1245,   1200

# ── BSN (north-north-east) ─────────────────────────────────────────
BSN                           @    285,    675
Scenes and BSN                @    390,    825
BSN syntax                    @    285,    975
Templates and assets          @    465,   1050
Composing scenes              @    315,   1125
Scene components              @    135,   1200
Spawning scenes               @    315,   1305   star

# ── ECS (north) ────────────────────────────────────────────────────
ECS                           @    -45,    660
System params                 @   -190,    740
Schedules and sets            @     30,    810
States                        @    -45,    945
Messages                      @     50,   1070
Change detection              @    -60,   1190
Relationships and hierarchy   @     45,   1320
Component lifecycle           @   -170,   1350
Reflection                    @   -195,   1490
Advanced queries              @    -30,   1470   star
Error handling                @    110,   1590
Testing                       @    -45,   1650

# ── Gameplay (north-west) ──────────────────────────────────────────
Gameplay                      @   -420,    750
Game AI                       @   -315,    920
Pathfinding                   @   -285,   1095   star
Randomness                    @   -615,    915
bevy_rand                     @   -885,    915
Procedural generation         @   -790,   1065   star
Inventory and items           @   -510,   1110
Dialogue                      @   -540,   1290
Custom assets                 @   -720,   1350
Saving and loading            @   -420,   1425   star
Accessibility and localization @   -320,   1605

# ── 2D (west-north-west) ───────────────────────────────────────────
2D                            @   -570,    465
bevy_prototype_lyon           @   -570,    645
bevy_vector_shapes            @   -705,    720
Texture atlases               @   -720,    585
Sprite animation              @   -885,    525
Text 2D                       @   -790,    360
Animation crates              @  -1035,    390   pentagon
bevy_tweening                 @  -1185,    345
bevy_spritesheet_animation    @  -1155,    465
Tilemaps                      @   -960,    750   star
2D shapes and meshes          @  -1095,    585
Tilemap crates                @  -1150,    765   pentagon
bevy_ecs_tilemap              @  -1290,    675
bevy_ecs_ldtk                 @  -1305,    795
2D lighting crates            @  -1110,    945   pentagon
bevy_light_2d                 @  -1260,    990
bevy_firefly                  @  -1170,   1095

# ── Input (west) ───────────────────────────────────────────────────
Keyboard and mouse            @   -675,    -60
Input                         @   -855,   -150
Input mapping                 @  -1035,    -45
Gamepad and touch             @  -1200,   -150   star
Picking                       @  -1170,   -330
Input crates                  @  -1230,     45   pentagon
bevy_enhanced_input           @  -1395,     90
leafwing-input-manager        @  -1350,    -45

# ── Physics (south-west) ───────────────────────────────────────────
Physics                       @   -645,   -480
Avian                         @   -825,   -615   star
bevy_transform_interpolation  @  -1005,   -525
bevy_rapier                   @   -810,   -795


# ── Landmarks ───────────────────────────────────────────────────
landmark: telescope           @    130,   -620   size 40
landmark: sparkle             @    -90,   -580   size 28
landmark: satellite           @    585,  -1380   size 40
landmark: moon-star           @    420,  -1695   size 56
landmark: sparkle             @   -675,  -1125   size 32
landmark: sparkle             @    525,  -1605   size 24
landmark: sparkles            @   1425,    900   size 40
landmark: sparkle             @  -1425,   -750   size 30
landmark: orbit               @    675,    150   size 40
landmark: music               @   1065,  -1365   size 40
landmark: sparkles            @   1320,   -795   size 44
landmark: shapes              @   1500,    -45   size 56
landmark: sun                 @   1785,    195   size 48
landmark: sparkle             @   1275,    190   size 26
landmark: satellite           @   1005,    360   size 44
landmark: sparkle             @   -615,   -250   size 26
landmark: gamepad-2           @  -1395,   -210   size 40
landmark: atom                @   -630,   -720   size 40
landmark: sparkle             @   -975,   -720   size 30
landmark: moon-star           @   -705,    795   size 44
landmark: sparkle             @   -180,   1065   size 26
landmark: tree-pine           @   -645,   1470   size 40
landmark: sparkle             @   -375,   1545   size 28
landmark: dices               @  -1005,   1065   size 40
landmark: hammer              @    525,   1395   size 40
landmark: crown               @    195,   1680   size 36
landmark: sparkles            @   1290,   1290   size 40
landmark: sparkle             @  -1410,    525   size 30

```

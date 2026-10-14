---
icon: map
map: true
---
# Map

Where everything sits on the map (`Ctrl+G`). Bevy is the brightest star, in the middle; the main quest, [[Your first game]], spirals out of it, and every other route on [[Bevy]] branches off the spiral toward its own region of the sky.

Edit the map with the pencil under the zoom buttons: drag notes and objects, change a note's shape, add landmarks, labels and nebulae. Changes are saved here, in the block below, which you can also edit by hand.

## How to write it

One line per thing on the map. Coordinates are `x, y`: `x` to the right, `y` up, from Bevy at `0, 0`.

- **A note**: its name, `@`, where it sits, then optionally a shape (`circle`, `diamond`, `hexagon`, `square`, `pentagon`, `octagon`, `star`) and `small` or `large`. Without a shape, the hub is an octagon, an area's index note a hexagon, a crate a square and any other note a circle.
- **`label:`** a region's name, with an optional caption after `|`, and `size N`.
- **`landmark:`** a [Lucide](https://lucide.dev/icons) icon drawn faintly in the sky, with `size N`.
- **`nebula:`** a soft cloud of light, `r N` across, fading to its edge (older maps say `land:`).

Lines starting with `#` are comments. A note left out of the map is put beside the note it grows from.

Every note is a star: dim until you have read it, lit after. Its shape is a thin outline around it.

| Shape | Means |
|:--|:--|
| octagon | the capital, the brightest star |
| hexagon | a region's town: an area's index |
| circle | a lesson (a plain star; the circle shows when hovered) |
| diamond | a milestone: you have built something |
| star | the goal at the end of a road |
| square | a crate: a library |

```map
# ── The capital and the main quest, spiralling out of it ───────
Bevy                          @      0,      0   octagon large
Start                         @      0,   -240
Hello Bevy                    @   -140,   -242   diamond
Entities and components       @   -277,   -160
Systems                       @   -360,      0
Queries                       @   -346,    200
Sprites                       @   -220,    381
Commands                      @      0,    480
Resources                     @    260,    450
Time and timers               @    485,    280
Events and observers          @    600,      0
Run conditions                @    554,   -320
Make a game                   @    340,   -589   star large
Game feel                     @    602,   -669
Project setup                 @   -345,   -441
App and plugins               @   -145,   -582

# ── The Shipyard: tooling (south) ────────────────────────────────
Tooling                       @     40,  -1120
Debugging                     @   -160,  -1300
Gizmos                        @   -420,  -1250
Diagnostics and logging       @     60,  -1480
Hot reloading                 @    310,  -1420
Profiling                     @   -150,  -1660
Debug crates                  @   -420,  -1640
bevy-inspector-egui           @   -640,  -1580
bevy_mod_debugdump            @   -600,  -1800
Release builds                @     60,  -1840   diamond
Web build                     @   -120,  -2040   star
Mobile builds                 @    180,  -2180
bevy_pkv                      @   -420,  -2120

# ── Asset Docks and Echo Valley: assets and audio (south-east) ──
Assets                        @    560,  -1100
Asset crates                  @    640,  -1380
bevy_asset_loader             @    560,  -1580
bevy_common_assets            @    800,  -1600
Audio                         @    900,  -1250
Audio crates                  @   1120,  -1420
bevy_kira_audio               @   1340,  -1380
bevy_seedling                 @   1260,  -1600
bevy_fmod                     @   1020,  -1620

# ── Render Ridge and the Lightworks: rendering (east) ───────────
Transforms                    @    980,    -40
Cameras                       @   1200,   -220
Windows                       @   1220,   -420
Camera crates                 @   1420,   -330
bevy_pancam                   @   1620,   -300
bevy_panorbit_camera          @   1580,   -460
Rendering                     @    880,   -380
Colors and math               @   1040,   -640
Custom materials              @   1300,   -700
Post-processing               @   1540,   -820   star
bevy_hanabi                   @   1480,  -1060
3D effect crates              @   1780,   -700
bevy_mod_outline              @   1980,   -640
bevy_water                    @   1900,   -880
bevy_rich_text3d              @   2040,   -440

# ── Polygon Peaks: 3D (far east) ────────────────────────────────
3D                            @   1440,    -80
Meshes and materials          @   1660,     40
Lighting                      @   1880,    -60
Models and glTF               @   2000,    170
3D animation                  @   2220,    330
3D authoring crates           @   2120,    560
bevy_skein                    @   2320,    600
bevy_trenchbroom              @   2180,    760
Environment                   @   2300,      0   star

# ── The Far Shores: networking (north-east) ─────────────────────
Networking                    @   1280,    720
bevy_replicon                 @   1540,    860   star
Netcode crates                @   1560,    600
lightyear                     @   1790,    560
bevy_ggrs                     @   1720,    420
bevy_matchbox                 @   1500,    420
bevy_steamworks               @   1820,    720
bevy_renet                    @   1800,    940
bevy_http_client              @   1640,   1080

# ── Interface Isles: UI (north-east) ────────────────────────────
UI                            @    880,    880
UI layout                     @   1000,   1100
Text and images               @   1220,   1000
Buttons and widgets           @   1300,   1240   star
Theming                       @   1520,   1180
bevy_radioactive              @    900,   1500
bevy_egui                     @   1160,   1480
UI crates                     @   1500,   1440
bevy_hui                      @   1700,   1380
bevy_lunex                    @   1660,   1600

# ── The Scene Forge: BSN (north-north-east) ─────────────────────
BSN                           @    380,    900
Scenes and BSN                @    520,   1100
BSN syntax                    @    380,   1300
Templates and assets          @    620,   1400
Composing scenes              @    420,   1500
Scene components              @    180,   1600
Spawning scenes               @    420,   1740   star

# ── The ECS Heartlands (north) ──────────────────────────────────
ECS                           @    -60,    880
System params                 @   -250,    990
Schedules and sets            @     40,   1080
States                        @    -60,   1260
Messages                      @     70,   1430
Change detection              @    -80,   1590
Relationships and hierarchy   @     60,   1760
Component lifecycle           @   -230,   1800
Reflection                    @   -260,   1990
Advanced queries              @    -40,   1960   star
Error handling                @    150,   2120
Testing                       @    -60,   2200

# ── The Questwood and the Archive: gameplay (north-west) ────────
Gameplay                      @   -560,   1000
Game AI                       @   -420,   1230
Pathfinding                   @   -380,   1460   star
Randomness                    @   -820,   1220
bevy_rand                     @  -1180,   1220
Procedural generation         @  -1050,   1420   star
Inventory and items           @   -680,   1480
Dialogue                      @   -720,   1720
Custom assets                 @   -960,   1800
Saving and loading            @   -560,   1900   star
Accessibility and localization @  -430,   2140

# ── Pixel Plains: 2D (west-north-west) ──────────────────────────
2D                            @   -760,    620
bevy_prototype_lyon           @   -760,    860
bevy_vector_shapes            @   -940,    960
Texture atlases               @   -960,    780
Sprite animation              @  -1180,    700
Text 2D                       @  -1050,    480
Animation crates              @  -1380,    520
bevy_tweening                 @  -1580,    460
bevy_spritesheet_animation    @  -1540,    620
Tilemaps                      @  -1280,   1000   star
2D shapes and meshes          @  -1460,    780
Tilemap crates                @  -1530,   1020
bevy_ecs_tilemap              @  -1720,    900
bevy_ecs_ldtk                 @  -1740,   1060
2D lighting crates            @  -1480,   1260
bevy_light_2d                 @  -1680,   1320
bevy_firefly                  @  -1560,   1460

# ── Controller Crossing: input (west) ───────────────────────────
Keyboard and mouse            @   -900,    -80
Input                         @  -1140,   -200
Input mapping                 @  -1380,    -60
Gamepad and touch             @  -1600,   -200   star
Picking                       @  -1560,   -440
Input crates                  @  -1640,     60
bevy_enhanced_input           @  -1860,    120
leafwing-input-manager        @  -1800,    -60

# ── Collision Canyon: physics (south-west) ──────────────────────
Physics                       @   -860,   -640
Avian                         @  -1100,   -820   star
bevy_transform_interpolation  @  -1340,   -700
bevy_rapier                   @  -1080,  -1060


# ── Landmarks ───────────────────────────────────────────────────
landmark: telescope           @    170,   -830   size 40
landmark: sparkle             @   -120,   -770   size 28
landmark: satellite           @    780,  -1840   size 40
landmark: moon-star           @    560,  -2260   size 56
landmark: sparkle             @   -900,  -1500   size 32
landmark: sparkle             @    700,  -2140   size 24
landmark: sparkles            @   1900,   1200   size 40
landmark: sparkle             @  -1900,  -1000   size 30
landmark: orbit               @    900,    200   size 40
landmark: music               @   1420,  -1820   size 40
landmark: sparkles            @   1760,  -1060   size 44
landmark: shapes              @   2000,    -60   size 56
landmark: sun                 @   2380,    260   size 48
landmark: sparkle             @   1700,    250   size 26
landmark: satellite           @   1340,    480   size 44
landmark: sparkle             @   -820,   -330   size 26
landmark: gamepad-2           @  -1860,   -280   size 40
landmark: atom                @   -840,   -960   size 40
landmark: sparkle             @  -1300,   -960   size 30
landmark: moon-star           @   -940,   1060   size 44
landmark: sparkle             @   -240,   1420   size 26
landmark: tree-pine           @   -860,   1960   size 40
landmark: sparkle             @   -500,   2060   size 28
landmark: dices               @  -1340,   1420   size 40
landmark: hammer              @    700,   1860   size 40
landmark: crown               @    260,   2240   size 36
landmark: sparkles            @   1720,   1720   size 40
landmark: sparkle             @  -1880,    700   size 30

```

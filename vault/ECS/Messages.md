---
icon: mail
tags: [ecs]
---
# Messages

A **message** is a note one system leaves for others. Writers append to a queue; readers walk through whatever is new. Nothing runs at the moment of writing, unlike an [[Events and observers|observer]]. #ecs

## Write and read

```rust example=messages fps=2
use bevy::prelude::*;

#[derive(Message)]
struct Shout(u32);

#[derive(Resource)]
struct Counter(u32);

fn shout(mut counter: ResMut<Counter>, mut writer: MessageWriter<Shout>) {
    counter.0 += 1;
    if counter.0 % 3 == 0 {
        writer.write(Shout(counter.0));
    }
}

fn listen(mut reader: MessageReader<Shout>, mut log: ResMut<Log>) {
    for message in reader.read() {
        log.say(format!("heard Shout({})", message.0));
    }
}

fn main() {
    App::new()
        .add_message::<Shout>()            // creates the queue
        .insert_resource(Counter(0))
        .add_systems(Update, (shout, listen).chain())
        .run();
}
```

Step through. Every third frame `shout` writes; `listen` (running after it) hears it in the same frame. Press **Send a message now** to write one yourself.

## How long do messages live?

`add_message` swaps two buffers each frame. A message survives **two frames**, so a reader that runs before the writer still gets it next frame. A system that does not run for a few frames (because of a [[Run conditions|run condition]]) will *miss* messages.

Each `MessageReader` keeps its own cursor: two systems reading `Shout` both see every message, and neither steals from the other.

## Reading tools

```rust
use bevy::prelude::*;

#[derive(Message)]
struct Hit { amount: u32 }

fn read(mut reader: MessageReader<Hit>) {
    if reader.is_empty() { return; }
    for hit in reader.read() { info!("{}", hit.amount); }
    let _ = reader.len();
}

fn clear(mut reader: MessageReader<Hit>) {
    reader.clear();                     // skip everything unread
}
```

A system that must both read and modify the same message type uses `MessageMutator<M>`.

## Built-in messages

You use messages all the time: keyboard and mouse input ([[Keyboard and mouse]]), `AppExit`, window events, asset events. To quit:

```rust
use bevy::prelude::*;

fn quit(mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}
```

## Choosing

- "Something happened and I want code to run **now**" → [[Events and observers|event]].
- "Lots of things happen; let me process them in a batch, in order" → message.
- Needs to target one entity → event.

## Next

[[Schedules and sets]]: deciding *when* each system runs.

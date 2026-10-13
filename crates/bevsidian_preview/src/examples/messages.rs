//! `Messages.md`: one system writes messages, another reads them a frame at a time.

use bevy::prelude::*;

use crate::example::{Action, Example, Log, Setup, View, update};

pub static EXAMPLE: Example = Example {
    setup,
    base: &[],
    functions: &[update!(shout), update!(listen)],
    tick: Some(tick),
    actions: &[Action {
        name: "Shout",
        label: "Send a message now",
        run: send,
    }],
    also: &[],
    view: View::Cards { fps: 2.0 },
};

fn setup(_: &World) -> Setup {
    Box::new(|world| {
        world.init_resource::<Messages<Shout>>();
        world.insert_resource(Counter(0));
    })
}

/// What `App::add_message` does between frames: swap the message buffers.
fn tick(world: &mut World) {
    world.resource_mut::<Messages<Shout>>().update();
}

#[derive(Message)]
struct Shout(u32);

#[derive(Resource)]
struct Counter(u32);

fn shout(mut counter: ResMut<Counter>, mut writer: MessageWriter<Shout>) {
    counter.0 += 1;
    if counter.0.is_multiple_of(3) {
        writer.write(Shout(counter.0));
    }
}

fn listen(mut reader: MessageReader<Shout>, mut log: ResMut<Log>) {
    for message in reader.read() {
        log.say(format!("heard Shout({})", message.0));
    }
}

fn send(world: &mut World) {
    world.write_message(Shout(0));
}

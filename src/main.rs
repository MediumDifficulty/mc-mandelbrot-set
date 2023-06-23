pub mod util;
pub mod renderer;

use std::time::{SystemTime, UNIX_EPOCH, Instant};

use log::{info, debug};
use renderer::Renderer;
use valence::client::message::SendMessage;
use valence::entity::{ObjectData, item_frame};
use valence::entity::player::PlayerEntityBundle;
use valence::protocol::Encode;
use valence::protocol::encode::WritePacket;
use valence::protocol::packet::map::{MapUpdateS2c, self};
use valence::protocol::var_int::VarInt;
use valence::prelude::*;
use valence::entity::glow_item_frame::GlowItemFrameEntityBundle;


fn main() {
    env_logger::init();
    info!("Staring...");

    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(init_clients)
        .add_system(despawn_disconnected_clients)
        .add_system(update_screen)
        .insert_non_send_resource(Renderer::new(1000, 1000, 1000).unwrap())
        .run();
}

#[derive(Resource)]
struct LocalURL(String);

fn setup(
    mut commands: Commands,
    server: Res<Server>,
    dimensions: ResMut<DimensionTypeRegistry>,
    biomes: ResMut<BiomeRegistry>,
    renderer: NonSend<Renderer>,
) {
    let mut instance = Instance::new(Ident::new("overworld").unwrap().to_string_ident(), &dimensions, &biomes, &server);

    let width = renderer.width as i32;
    let height = renderer.height as i32;

    let width_maps = num_integer::div_ceil(width, 128);
    let height_maps = num_integer::div_ceil(height, 128);

    let width_chunks = num_integer::div_ceil(width_maps, 16);
    let height_chunks = num_integer::div_ceil(height_maps, 16);

    // Initialise chunks
    for x in -1..=width_chunks {
        for z in -1..=height_chunks {
            instance.insert_chunk([x, z], Chunk::default());
        }
    }

    // Set screen blocks
    for x in 0..width_maps {
        for z in 0..height_maps {
            instance.set_block([x, 63, z], BlockState::BLACK_CONCRETE);
        }
    }

    let instance_id = commands.spawn(instance).id();

    // Spawn item frames
    for x in 0..width_maps {
        for z in 0..height_maps {
            let mut nbt = Compound::new();
            nbt.insert("map", z*width_maps + x);

            commands.spawn(GlowItemFrameEntityBundle {
                location: Location(instance_id),
                position: Position(DVec3::new(x as f64, 64., z as f64)),
                item_frame_item_stack: item_frame::ItemStack(ItemStack::new(ItemKind::FilledMap, 1, Some(nbt))),
                object_data: ObjectData(1),
                ..Default::default()
            });
        }
    }

    info!("Initialised Minecraft server");
}

fn init_clients(
    mut clients: Query<(Entity, &UniqueId, &mut Client, &mut GameMode), Added<Client>>,
    instances: Query<Entity, With<Instance>>,
    mut commands: Commands,
) {
    for (entity, uuid, mut client, mut game_mode) in &mut clients {
        *game_mode = GameMode::Creative;

        client.send_chat_message("Welcome to MCVideo V3!");

        let mut brand = Vec::new();
        "MCVideo".encode(&mut brand).unwrap();
        client.send_custom_payload( Ident::new("minecraft:brand").unwrap().as_str_ident(), &brand);

        commands.entity(entity).insert(PlayerEntityBundle {
            location: Location(instances.single()),
            position: Position(DVec3::new(0., 64., 0.)),
            uuid: *uuid,
            ..Default::default()
        });
    }
}

fn update_screen(
    mut renderer: NonSendMut<Renderer>,
    mut clients: Query<&mut Client>,
) { 
    let start_time = Instant::now();

    let frame = renderer.render().unwrap();

    for mut client in clients.iter_mut() {
        for (i, map_data) in frame.iter().enumerate() {
            client.write_packet(&MapUpdateS2c {
                scale: 0,
                icons: None,
                locked: false,
                map_id: VarInt(i as i32),
                data: Some(map::Data {
                    columns: 128,
                    rows: 128,
                    position: [0, 0],
                    data: map_data
                })
            });
        }
    }
    debug!("Updated screen in {}ms", start_time.elapsed().as_millis());
}

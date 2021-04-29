use rltk::{RGB, RandomNumberGenerator};
use specs::prelude::*;
use super::{CombatStats, Player, Renderable, Name, BlocksTile, Position, Fov,
    Monster, Rect, map::MAPWIDTH, Item, Potion, SpawnTable, Equippable, EquipmentSlot};
use std::collections::HashMap;


const MAX_MONSTERS : i32 = 4;

/// Spawns player and returns that entity object
pub fn player(ecs : &mut World, player_x : i32, player_y : i32) -> Entity {
    ecs
        .create_entity()
        .with(Position {x:player_x, y:player_y})
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0
        })
        .with(Player{})
        .with(Fov{visible_tiles : Vec::new(), range : 8, dirty : true})
        .with(Name{name: "Player".to_string()})
        .with(CombatStats{max_hp: 30, hp: 30, defense: 2, power: 5})
        .build()
}


fn orc(ecs: &mut World, x:i32, y:i32) {monster(ecs, x, y, rltk::to_cp437('o'), "Orc");}
fn goblin(ecs: &mut World, x:i32, y:i32) {monster(ecs, x, y, rltk::to_cp437('g'), "Goblin");}

fn monster<S : ToString>(ecs: &mut World, x: i32, y:i32, glyph: rltk::FontCharType, name : S) {
    ecs.create_entity()
        .with(Position{x, y})
        .with(Renderable{
            glyph: glyph,
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 1
        })
        .with(Fov{visible_tiles : Vec::new(), range : 8, dirty : true})
        .with(Monster{})
        .with(Name{name: name.to_string()})
        .with(BlocksTile{})
        .with(CombatStats{max_hp: 16, hp: 16, defense: 1, power: 4})
        .build();
}

pub fn spawn_room(ecs: &mut World, room: &Rect, depth: i32) {
    let spawn_table = room_table(depth);
    let mut spawn_points : HashMap<usize, String> = HashMap::new();

    // Scope
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_spawns = rng.roll_dice(1, MAX_MONSTERS + 3) + (depth - 1) -3;

        for _i in 0..num_spawns {
            let mut added = false;
            let mut tries = 0;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !spawn_points.contains_key(&idx) {
                    spawn_points.insert(idx, spawn_table.roll(&mut rng));
                    added = true;
                } else {
                    tries += 1;
                }
            }
        }
    }

    // Spawning monsters and items
    for spawn in spawn_points.iter() {
        let x = (*spawn.0 % MAPWIDTH) as i32;
        let y = (*spawn.0 / MAPWIDTH) as i32;
    
        match spawn.1.as_ref() {
            "Goblin" => goblin(ecs, x, y),
            "Orc" => orc(ecs, x, y),
            "Health Potion" => health_potion(ecs, x, y),
            "Shield" => shield(ecs, x, y),
            "Dagger" => dagger(ecs, x, y),
            _ => {}
        }
    }
}

/// Spawns health potion
fn health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{x, y})
        .with(Renderable{
            glyph: rltk::to_cp437('!'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{name: "Health Potion".to_string()})
        .with(Item{})
        .with(Potion{heal_amount: 8})
        .build();
}

fn dagger(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{x,y})
        .with(Renderable{
            glyph: rltk::to_cp437('/'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{name : "Dagger".to_string()})
        .with(Equippable{slot: EquipmentSlot::Weapon})
        .with(Item{})
        .build();
}

fn shield(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{x,y})
        .with(Renderable{
            glyph: rltk::to_cp437(']'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{name: "Shield".to_string()})
        .with(Equippable{slot: EquipmentSlot::Shield})
        .with(Item{})
        .build();
}

/// Gives weighted chances for spawns in room
fn room_table(depth: i32) -> SpawnTable {
    SpawnTable::new()
        .add("Goblin", 11)
        .add("Orc", 1 + depth)
        .add("Health Potion", 7)
        .add("Shield", 3)
        .add("Dagger", 3)
}
use rltk::{VirtualKeyCode, Rltk, Point};
use specs::prelude::*;
use std::cmp::{min, max};
use super::{Position, Player, Fov, State, Map, RunState, CombatStats, AttackIntent,
	Item, gamelog::Gamelog, WantsToPickupItem, TileType, Monster};

pub fn player_move(dx: i32, dy: i32, ecs: &mut World) {
	let mut pos = ecs.write_storage::<Position>();
	let mut players = ecs.write_storage::<Player>();
	let mut fovs = ecs.write_storage::<Fov>();
	let map = ecs.fetch::<Map>();
	let combat_stats = ecs.read_storage::<CombatStats>();
	let entities = ecs.entities();
	let mut attack_intent = ecs.write_storage::<AttackIntent>();

	for (entity,_player, p, fov) in (&entities, &mut players, &mut pos, &mut fovs).join() {
		if p.x + dx < 1 || p.x + dx > map.width-1 || p.y + dy < 1 || p.y + dy > map.height-1 {return;}
		let dest_idx = map.xy_idx(p.x + dx, p.y + dy);

		for potential_target in map.tile_content[dest_idx].iter() {
			let target = combat_stats.get(*potential_target);
			if let Some(_target) = target {
				attack_intent.insert(entity, AttackIntent{target: *potential_target}).expect("Add target failed");
				return; // For not moving after attacking
			}
		}

		if !map.blocked[dest_idx] {
			p.x = min(79, max(0, p.x + dx));
			p.y = min(49, max(0, p.y + dy));

			fov.dirty = true;
			let mut ppos = ecs.write_resource::<Point>();
			ppos.x = p.x;
			ppos.y = p.y;
		}
	}
}

fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<Gamelog>();

    let mut target_item : Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog.entries.push("There is nothing here to pick up.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup.insert(*player_entity, WantsToPickupItem{ collected_by: *player_entity, item }).expect("Unable to insert want to pickup");
        }
    }
}

pub fn climb_down(ecs: &mut World) -> bool {
	let player_pos = ecs.fetch::<Point>();
	let map = ecs.fetch::<Map>();
	let player_idx = map.xy_idx(player_pos.x, player_pos.y);
	if map.tiles[player_idx] == TileType::DownStairs {
		true
	} else {
		let mut gamelog = ecs.fetch_mut::<Gamelog>();
		gamelog.entries.push("There is no stairs to go down from here.".to_string());
		false
	}
}

/// Heals if monsters can't be seen, otherwise would have just returned RunState::PlayerTurn
fn skip_turn(ecs: &mut World) -> RunState {
	let player_entity = ecs.fetch::<Entity>();
	let sight = ecs.read_storage::<Fov>();
	let monsters = ecs.read_storage::<Monster>();

	let map = ecs.fetch::<Map>();

	let mut can_heal = true;
	let fov = sight.get(*player_entity).unwrap();
	for tile in fov.visible_tiles.iter() {
		let idx = map.xy_idx(tile.x, tile.y);
		for entity in map.tile_content[idx].iter() {
			let mob = monsters.get(*entity);
			match mob {
				None => {}
				Some(_) => can_heal = false
			}
		}
	}

	if can_heal {
		let mut health = ecs.write_storage::<CombatStats>();
		let player_hp = health.get_mut(*player_entity).unwrap();
		player_hp.hp = i32::min(player_hp.hp + 1, player_hp.max_hp);
	}

	RunState::PlayerTurn
}

pub fn keyboard(gs: &mut State, ctx: &mut Rltk) -> RunState {
	// movement
	match ctx.key {
		None => {return RunState::AwaitingInput} // When nothing happens
		Some(key) => match key {
			VirtualKeyCode::Numpad4 => player_move(-1, 0, &mut gs.ecs),	// left
			VirtualKeyCode::Numpad6 => player_move(1, 0, &mut gs.ecs),	// right
			VirtualKeyCode::Numpad8 => player_move(0, -1, &mut gs.ecs),	// up
			VirtualKeyCode::Numpad2 => player_move(0, 1, &mut gs.ecs),	// down 
			VirtualKeyCode::Numpad9 => player_move(1, -1, &mut gs.ecs),	// up-right
			VirtualKeyCode::Numpad7 => player_move(-1, -1, &mut gs.ecs),// up-left
			VirtualKeyCode::Numpad1 => player_move(-1, 1, &mut gs.ecs),	// down-left
			VirtualKeyCode::Numpad3 => player_move(1, 1, &mut gs.ecs),	// down-right
			VirtualKeyCode::Numpad5 |
				VirtualKeyCode::Space => return skip_turn(&mut gs.ecs),	// skip turn
			VirtualKeyCode::G => get_item(&mut gs.ecs),					// pickup item
			VirtualKeyCode::I => return RunState::ShowInventory,
			VirtualKeyCode::D => return RunState::ShowDropItem,
			VirtualKeyCode::Period => {
				if climb_down(&mut gs.ecs) {
					return RunState::NextLevel;
				}
			}
			_ => {return RunState::AwaitingInput}
		},
	}
	RunState::PlayerTurn
}
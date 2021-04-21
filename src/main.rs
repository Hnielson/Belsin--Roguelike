use rltk::{Rltk, GameState, Point};
use specs::prelude::*;
mod components;
pub use components::*;
mod map;
use map::*;
mod player;
use player::*;
mod rect;
pub use rect::Rect;
mod fov;
use fov::FovSystem;
mod monster_ai_system;
use monster_ai_system::MonsterAI;
mod map_indexing;
use map_indexing::MapIndexSystem;
mod melee_combat_system;
use melee_combat_system::MeleeCombatSystem;
mod damage_system;
use damage_system::DamageSystem;
mod gui;
mod gamelog;
mod spawner;
mod inventory_system;
use inventory_system::{ItemCollectionSystem, PotionUseSystem, ItemDropSystem};
mod spawn_table;
pub use spawn_table::SpawnTable;


#[derive(PartialEq, Copy, Clone)]
pub enum RunState {AwaitingInput, PreRun, PlayerTurn, MonsterTurn,
	ShowInventory, ShowDropItem, Menu {selection: gui::MenuSelection}, NextLevel}

pub struct State {
	pub ecs: World
}

// System Runner
impl State {
	fn run_systems(&mut self) {
		// FOV
		let mut fov = FovSystem{};
		fov.run_now(&self.ecs);
		// Monster
		let mut mob = MonsterAI{};
		mob.run_now(&self.ecs);
		// BlockedList
		let mut mapidx = MapIndexSystem{};
		mapidx.run_now(&self.ecs);
		// Melee System
		let mut melee = MeleeCombatSystem{};
		melee.run_now(&self.ecs);
		// Damage System
		let mut damage = DamageSystem{};
		damage.run_now(&self.ecs);
		// Item Pickup System
		let mut pickup = ItemCollectionSystem{};
		pickup.run_now(&self.ecs);
		// Potions
		let mut potions = PotionUseSystem{};
		potions.run_now(&self.ecs);
		// Dropping Items
		let mut drop_items = ItemDropSystem{};
		drop_items.run_now(&self.ecs);

		self.ecs.maintain();
	}
}

impl State {
	fn clean_level_change(&mut self) -> Vec<Entity> {
		let entities = self.ecs.entities();
		let player = self.ecs.read_storage::<Player>();
		let backpack = self.ecs.read_storage::<InBackpack>();
		let player_entity = self.ecs.fetch::<Entity>();

		let mut to_delete: Vec<Entity> = Vec::new();
		for entity in entities.join() {
			let mut marked_for_delete = true;

			// Prevent Player delete or equipment
			let p = player.get(entity);
			if let Some(_p) = p {
				marked_for_delete = false;
			}
			let bp = backpack.get(entity);
			if let Some(bp) = bp {
				if bp.owner == *player_entity {
					marked_for_delete = false;
				}
			}

			if marked_for_delete {
				to_delete.push(entity);
			}
		}

		to_delete
	}


	fn next_level(&mut self) {
		// Deleting entities that aren't player and inventory
		let to_delete = self.clean_level_change();
		for marked in to_delete {
			self.ecs.delete_entity(marked).expect("Unable to delete entities");
		}

		// New map generation
		let new_level;
		{
			let mut map = self.ecs.write_resource::<Map>();
			let curr_depth = map.depth;
			*map = Map::new_map_rooms_and_corridors(curr_depth + 1);
			new_level = map.clone();
		}
		// Monster Entities
		for room in new_level.rooms.iter().skip(1) {
			spawner::spawn_room(&mut self.ecs, room);
		}
		// Player position
		let (p_x, p_y) = new_level.rooms[0].center();
		let mut player_pos = self.ecs.write_resource::<Point>();
		*player_pos = Point::new(p_x, p_y);
		let mut equip_pos = self.ecs.write_storage::<Position>();
		let player_entity = self.ecs.fetch::<Entity>();
		let player_equip = equip_pos.get_mut(*player_entity);
		if let Some(player_equip) = player_equip {
			player_equip.x = p_x;
			player_equip.y = p_y;
		}

		// FOV
		let mut sight = self.ecs.write_storage::<Fov>();
		let fov = sight.get_mut(*player_entity);
		if let Some(fov) = fov {
			fov.dirty = true;
		}

		// Next level notification
		let mut gamelog = self.ecs.fetch_mut::<gamelog::Gamelog>();
		gamelog.entries.push("You descend.".to_string());
	}
}

/// GameState that requires specific ordering of contents for rendering purposes
impl GameState for State {
    fn tick(&mut self, ctx : &mut Rltk) {

		// Initialize Runstates
		let mut newrunstate;
		{
			let runstate = self.ecs.fetch::<RunState>();
			newrunstate = *runstate;
		}

        ctx.cls();


		// Matching States for Turns and Item menus
		match newrunstate {
			RunState::Menu{..} => {}
			_ => {
				// map needs to be drawn first
				draw_map(&self.ecs, ctx);

				// UI and entities need to be drawn after map
				// otherwise they will be drawn over
				{
					let pos = self.ecs.read_storage::<Position>();
					let ren = self.ecs.read_storage::<Renderable>();
					let map = self.ecs.fetch::<Map>();

					// Render Entities with both a Position and Renderable Component
					let mut data = (&pos, &ren).join().collect::<Vec<_>>();
					data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
					for (p, r) in data.iter() {
						let idx = map.xy_idx(p.x, p.y);
						if map.visible_tiles[idx] {ctx.set(p.x, p.y, r.fg, r.bg, r.glyph)}
					}

					gui::draw_ui(&self.ecs, ctx);
				}
			}
		}

		match newrunstate {
			RunState::PreRun => {
				self.run_systems();
				self.ecs.maintain();
				newrunstate = RunState::AwaitingInput;
			}
			RunState::AwaitingInput => {
				newrunstate = keyboard(self, ctx);
			}
			RunState::PlayerTurn => {
				self.run_systems();
				self.ecs.maintain();
				newrunstate = RunState::MonsterTurn;
			}
			RunState::MonsterTurn => {
				self.run_systems();
				self.ecs.maintain();
				newrunstate = RunState::AwaitingInput;
			}
			RunState::NextLevel => {
				self.next_level();
				newrunstate = RunState::PreRun;
			}
			RunState::ShowInventory => {
				let result = gui::show_inventory(self, ctx);
				match result.0 {
					gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
					gui::ItemMenuResult::NoResponse => {}
					gui::ItemMenuResult::Selected => {
						let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDrinkPotion>();
                        intent.insert(*self.ecs.fetch::<Entity>(), WantsToDrinkPotion{ potion: item_entity }).expect("Unable to insert intent");
                        newrunstate = RunState::PlayerTurn;
					}
				}
			}
			RunState::ShowDropItem => {
				let result = gui::drop_item_menu(self, ctx);
				match result.0 {
					gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
					gui::ItemMenuResult::NoResponse => {}
					gui::ItemMenuResult::Selected => {
						let item_entity = result.1.unwrap();
						let mut intent = self.ecs.write_storage::<WantsToDropItem>();
						intent.insert(*self.ecs.fetch::<Entity>(), WantsToDropItem{item : item_entity}).expect("Unable to insert intent.");
						newrunstate = RunState::PlayerTurn;
					}
				}
			}
			RunState::Menu{..} => {
				let result = gui::menu(self, ctx);
				match result {
					gui::MenuResult::NoSelection{selected} => newrunstate = RunState::Menu{selection:selected},
					gui::MenuResult::Selection{selected} => {
						match selected {
							gui::MenuSelection::NewGame => newrunstate = RunState::PreRun,
							gui::MenuSelection::Quit => {::std::process::exit(0);}
						}
					}
				}
			}
		}

		// Reset Runstate
		{
			let mut runwriter = self.ecs.write_resource::<RunState>();
			*runwriter = newrunstate;
		}

		damage_system::corpse_removal(&mut self.ecs);
    }
}


fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let mut context = RltkBuilder::simple80x50()
        .with_title("Belsin")
        .build()?;
	context.with_post_scanlines(true);
	
	// State Creator
    let mut gs = State{
		ecs: World::new(),
	};
	// Register all components in the Entity Component System in GameState
	gs.ecs.register::<Position>();
	gs.ecs.register::<Renderable>();
	gs.ecs.register::<Player>();
	gs.ecs.register::<Fov>();
	gs.ecs.register::<Monster>();
	gs.ecs.register::<Name>();
	gs.ecs.register::<BlocksTile>();
	gs.ecs.register::<CombatStats>();
	gs.ecs.register::<AttackIntent>();
	gs.ecs.register::<SufferDamage>();
	gs.ecs.register::<Item>();
	gs.ecs.register::<Potion>();
	gs.ecs.register::<InBackpack>();
	gs.ecs.register::<WantsToPickupItem>();
	gs.ecs.register::<WantsToDrinkPotion>();
	gs.ecs.register::<WantsToDropItem>();
		
	// Map making
	let map : Map = Map::new_map_rooms_and_corridors(1);
	let (player_x, player_y) = map.rooms[0].center();

	let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);

	// Must insert Map after all other uses
	// resources
	gs.ecs.insert(rltk::RandomNumberGenerator::new());
	for room in map.rooms.iter().skip(1) {
		spawner::spawn_room(&mut gs.ecs, room);
	}
	gs.ecs.insert(map);
	gs.ecs.insert(Point::new(player_x, player_y));
	gs.ecs.insert(player_entity);
	gs.ecs.insert(RunState::Menu{selection: gui::MenuSelection::NewGame});
	gs.ecs.insert(gamelog::Gamelog{entries : vec!["Welcome to Belsin!".to_string()]});
	// gs.ecs.insert(InBackpack.insert(potion, owner: player_entity))

    rltk::main_loop(context, gs)
}

use rltk::{RGB, Rltk, VirtualKeyCode};
use specs::prelude::*;
use super::{CombatStats, Player, gamelog::Gamelog, InBackpack, Name, State, RunState, Map};


#[derive(PartialEq, Copy, Clone)]
pub enum MenuSelection {NewGame, Quit}

#[derive(PartialEq, Copy, Clone)]
pub enum MenuResult {NoSelection{selected:MenuSelection}, Selection{selected:MenuSelection}}

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    ctx.draw_box(0, 43, 79, 6, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

    // Depth of map
    let map = ecs.fetch::<Map>();
    let depth = format!("Depth: {}", map.depth);
    ctx.print_color(2, 43, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), &depth);

    // health bar
    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    for (_player, stats) in (&players, &combat_stats).join() {
        let health = format!("HP: {} / {}", stats.hp, stats.max_hp);
        ctx.print_color(12, 43, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), &health);

        ctx.draw_bar_horizontal(28, 43, 51, stats.hp, stats.max_hp, RGB::named(rltk::RED), RGB::named(rltk::BLACK));
    }
    
    let log = ecs.fetch::<Gamelog>();
    
    // Console logs
    let mut y = 44;
    for s in log.entries.iter().rev() {
        if y < 49 {ctx.print(2, y, s);}
        y += 1;
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult {Cancel, NoResponse, Selected}

/// Inventory Menu
pub fn show_inventory(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names).join().filter(|item| item.0.owner == *player_entity);
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y-2, 31, (count+3) as i32, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.print_color(18, y-2, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Inventory");
    ctx.print_color(18, y+count as i32+1, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Esc to close");

    let mut equippable : Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _pack, name) in (&entities, &backpack, &names).join().filter(|item| item.1.owner == *player_entity) {
        ctx.set(17, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        ctx.set(18, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), 97+j as rltk::FontCharType);
        ctx.set(19, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => {(ItemMenuResult::Cancel, None)}
                _ => {
                    let selection = rltk::letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        return (ItemMenuResult::Selected, Some(equippable[selection as usize]));
                    }
                    (ItemMenuResult::NoResponse, None)
                }
            }
        }
    }
}

/// Item Menu for dropping items in backpack
pub fn drop_item_menu(gs : &mut State, ctx : &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names).join().filter(|item| item.0.owner == *player_entity );
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y-2, 31, (count+3) as i32, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.print_color(18, y-2, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Drop Which Item?");
    ctx.print_color(18, y+count as i32+1, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Esc to close");

    let mut equippable : Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _pack, name) in (&entities, &backpack, &names).join().filter(|item| item.1.owner == *player_entity ) {
        ctx.set(17, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        ctx.set(18, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), 97+j as rltk::FontCharType);
        ctx.set(19, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => { (ItemMenuResult::Cancel, None) }
                _ => { 
                    let selection = rltk::letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        return (ItemMenuResult::Selected, Some(equippable[selection as usize]));
                    }  
                    (ItemMenuResult::NoResponse, None)
                }
            }
        }
    }
}

/// Main Game Menu
pub fn menu(gs:&mut State, ctx: &mut Rltk) -> MenuResult {
    let runstate = gs.ecs.fetch::<RunState>();

    ctx.print_color_centered(15, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Belsin -- (GCOTP)");

    if let RunState::Menu{selection:select} = *runstate {
        if select == MenuSelection::NewGame {
            ctx.print_color_centered(24, RGB::named(rltk::GREEN), RGB::named(rltk::BLACK), "Start New Game");  
        } else {
            ctx.print_color_centered(24, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Start New Game");
        }

        if select == MenuSelection::Quit {
            ctx.print_color_centered(25, RGB::named(rltk::GREEN), RGB::named(rltk::BLACK), "YOU COWARD!");
        } else {
            ctx.print_color_centered(25, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Quit");
        }

        match ctx.key {
            None => return MenuResult::NoSelection{selected:select},
            Some(key) => {
                match key {
                    VirtualKeyCode::Escape => {return MenuResult::NoSelection{selected:MenuSelection::Quit}}
                    VirtualKeyCode::Up => {
                        let newchoice;
                        match select {
                            MenuSelection::NewGame => newchoice = MenuSelection::Quit,
                            MenuSelection::Quit => newchoice = MenuSelection::NewGame
                        }
                        return MenuResult::NoSelection{selected:newchoice}
                    }
                    VirtualKeyCode::Down => {
                        let newchoice;
                        match select {
                            MenuSelection::NewGame => newchoice = MenuSelection::Quit,
                            MenuSelection::Quit => newchoice = MenuSelection::NewGame
                        }
                        return MenuResult::NoSelection{selected:newchoice}
                    }
                    VirtualKeyCode::Return => return MenuResult::Selection{selected:select},
                    _ => return MenuResult::NoSelection{selected:select}
                }
            }
        }
    }

    MenuResult::NoSelection{selected:MenuSelection::NewGame}
}
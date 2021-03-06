use specs::prelude::*;
use super::{WantsToPickupItem, Name, InBackpack, Position, gamelog::Gamelog, CombatStats,
    Potion, WantsToDrinkPotion, WantsToDropItem, Equipped, Equippable};

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, Gamelog>,
                        WriteStorage<'a, WantsToPickupItem>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, InBackpack>);

    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack) = data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack.insert(pickup.item, InBackpack{owner: pickup.collected_by}).expect("Unable to place item in backpack");

            if pickup.collected_by == *player_entity {
                gamelog.entries.push(format!("{} has been picked up", names.get(pickup.item).unwrap().name));
            }
        }

        wants_pickup.clear();
    }
}

pub struct PotionUseSystem {}

impl<'a> System<'a> for PotionUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, Gamelog>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToDrinkPotion>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, Potion>,
                        WriteStorage<'a, CombatStats>,
                        ReadStorage<'a, Equippable>,
                        WriteStorage<'a, Equipped>,
                        WriteStorage<'a, InBackpack>);

    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut want_potion, names, potions, mut combat_stats, equippable, mut equip, mut backpack) = data;

        for (entity, drink, stats) in (&entities, &want_potion, &mut combat_stats).join() {
            let potion = potions.get(drink.potion);
            match potion {
                None => {}
                Some(potion) => {
                    stats.hp = i32::min(stats.max_hp, stats.hp + potion.heal_amount);
                    if entity == *player_entity {
                        gamelog.entries.push(format!("You drink the {}, healing {} hp", names.get(drink.potion).unwrap().name, potion.heal_amount));
                    }
                    entities.delete(drink.potion).expect("Delete potion failed");
                }
            }
            
            // Pretty raw way to equip things while they are still called potions. Gonna fix
            let equip_item = equippable.get(drink.potion);
            match equip_item {
                None => {}
                Some(can_equip) => {
                    let target_slot = can_equip.slot;
                    
                    // Remove any item in equipment slot first before trying to insert a new one
                    let mut to_unequip : Vec<Entity> = Vec::new();
                    for(item, already_equipped, name) in (&entities, &equip, &names).join() {
                        if already_equipped.owner == *player_entity && already_equipped.slot == target_slot {
                            to_unequip.push(item);
                            if entity == *player_entity {
                                gamelog.entries.push(format!("{} has been unequipped.", name.name));
                            }
                        }
                    }
                    for thing in to_unequip.iter() {
                        equip.remove(*thing);
                        backpack.insert(*thing, InBackpack{owner: entity}).expect("Can't put into backpack");
                    }

                    // The actual equipping
                    equip.insert(drink.potion, Equipped{owner: entity, slot: target_slot}).expect("Can't insert equipped item");
                    backpack.remove(drink.potion);
                    if entity == *player_entity {
                        gamelog.entries.push(format!("You have equipped {}.", names.get(drink.potion).unwrap().name));
                    }
                }
            }
        }

        want_potion.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, Gamelog>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToDropItem>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, InBackpack>);

    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut want_drop, names, mut positions, mut backpack) = data;

        for (entity, to_drop) in (&entities, &want_drop).join() {
            let mut dropper_pos : Position = Position{x:0, y:0};
            {
                let dropped_pos = positions.get(entity).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }
            positions.insert(to_drop.item, Position{x:dropper_pos.x, y:dropper_pos.y}).expect("Unable to insert position");
            backpack.remove(to_drop.item);

            if entity == *player_entity {
                gamelog.entries.push(format!("You dropped the {}", names.get(to_drop.item).unwrap().name));
            }
        }

        want_drop.clear();
    }
}
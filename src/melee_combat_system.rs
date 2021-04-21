use specs::prelude::*;
use super::{CombatStats, AttackIntent, Name, SufferDamage, gamelog::Gamelog};

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = ( Entities<'a>,
                        WriteExpect<'a, Gamelog>,
                        WriteStorage<'a, AttackIntent>,
                        ReadStorage<'a, CombatStats>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, SufferDamage>);

    fn run(&mut self, data : Self::SystemData) {
        let (entities, mut log, mut atk_int, combat_stats, names, mut inflict_damage) = data;

        for (_entity, atk_int, name, stats) in (&entities, &atk_int, &names, &combat_stats).join() {
            if stats.hp > 0 {
                let target_stats = combat_stats.get(atk_int.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(atk_int.target).unwrap();

                    let damage = i32::max(0, stats.power - target_stats.defense);

                    if damage == 0 {
                        log.entries.push(format!("{} Unable to hurt {}", &name.name, &target_name.name));
                    } else {
                        log.entries.push(format!("{} hits {}, for {} hp", &name.name, &target_name.name, damage));
                        SufferDamage::new_damage(&mut inflict_damage, atk_int.target, damage);
                    }
                }
            }
        }
        atk_int.clear();
    }
}
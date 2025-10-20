use raphael_data::*;

fn find_consumable(consumables: &[Consumable], item_id: u32, hq: bool) -> Option<Consumable> {
    for consumable in consumables {
        if consumable.item_id == item_id && consumable.hq == hq {
            return Some(*consumable);
        }
    }
    None
}

#[test]
fn test_rroneek_steak() {
    let item_id = 44091;
    assert_eq!(
        get_item_name(item_id, false, Locale::EN).unwrap(),
        "Rroneek Steak"
    );
    let consumable = find_consumable(MEALS, item_id, false).unwrap();
    assert_eq!((consumable.craft_rel, consumable.craft_max), (0, 0));
    assert_eq!((consumable.control_rel, consumable.control_max), (4, 77));
    assert_eq!((consumable.cp_rel, consumable.cp_max), (21, 73));
    let consumable = find_consumable(MEALS, item_id, true).unwrap();
    assert_eq!((consumable.craft_rel, consumable.craft_max), (0, 0));
    assert_eq!((consumable.control_rel, consumable.control_max), (5, 97));
    assert_eq!((consumable.cp_rel, consumable.cp_max), (26, 92));
}

#[test]
fn test_unbuffed_single_consumable() {
    for consumable in MEALS.iter().chain(POTIONS) {
        let consumables = &[Some(*consumable)];
        for base_stat in 0..=9999 {
            assert_eq!(
                craftsmanship_unbuffed(
                    base_stat + craftsmanship_bonus(base_stat, consumables),
                    consumables
                ),
                Some(base_stat)
            );
            assert_eq!(
                control_unbuffed(
                    base_stat + control_bonus(base_stat, consumables),
                    consumables
                ),
                Some(base_stat)
            );
            assert_eq!(
                cp_unbuffed(base_stat + cp_bonus(base_stat, consumables), consumables),
                Some(base_stat)
            );
        }
    }
}

#[test]
fn test_unbuffed_consumable_combinations() {
    for food in MEALS {
        for potion in POTIONS {
            let consumables = &[Some(*food), Some(*potion)];
            for base_stat in 0..=9999 {
                assert_eq!(
                    craftsmanship_unbuffed(
                        base_stat + craftsmanship_bonus(base_stat, consumables),
                        consumables
                    ),
                    Some(base_stat)
                );
                assert_eq!(
                    control_unbuffed(
                        base_stat + control_bonus(base_stat, consumables),
                        consumables
                    ),
                    Some(base_stat)
                );
                assert_eq!(
                    cp_unbuffed(base_stat + cp_bonus(base_stat, consumables), consumables),
                    Some(base_stat)
                );
            }
        }
    }
}

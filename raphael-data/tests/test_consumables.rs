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

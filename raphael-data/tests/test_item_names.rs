use raphael_data::*;

#[test]
fn test_item_name_2341() {
    let item_id = 2341;
    let item_names = [
        get_item_name(item_id, false, Locale::EN).unwrap(),
        get_item_name(item_id, false, Locale::DE).unwrap(),
        get_item_name(item_id, false, Locale::FR).unwrap(),
        get_item_name(item_id, false, Locale::JP).unwrap(),
    ];
    assert_eq!(
        item_names,
        [
            "Bronze Cross-pein Hammer",
            "Bronze-Kreuzschlaghammer", // "<SoftHyphen/>" should not appear in the item name
            "Marteau à panne croisée",
            "クロスペインハンマー"
        ]
    );
}

#[test]
fn test_item_name_44232_collectable() {
    let item_id = 44232;
    let item_names = [
        get_item_name(item_id, false, Locale::EN).unwrap(),
        get_item_name(item_id, true, Locale::DE).unwrap(),
        get_item_name(item_id, false, Locale::FR).unwrap(),
        get_item_name(item_id, true, Locale::JP).unwrap(),
    ];
    assert_eq!(
        item_names,
        [
            "Rarefied Tacos de Carne Asada \u{e03d}",
            "Tacos de Carne Asada (Sammlerstück) \u{e03d}",
            "Tacos de carne asada collectionnables \u{e03d}",
            "収集用のタコス・カルネ・アサーダ \u{e03d}"
        ]
    );
}

#[test]
fn test_item_name_44104_hq() {
    let item_id = 44104;
    let item_names = [
        get_item_name(item_id, true, Locale::EN).unwrap(),
        get_item_name(item_id, true, Locale::DE).unwrap(),
        get_item_name(item_id, true, Locale::FR).unwrap(),
        get_item_name(item_id, true, Locale::JP).unwrap(),
    ];
    assert_eq!(
        item_names,
        [
            "Tacos de Carne Asada \u{e03c}",
            "Tacos mit Carne Asada \u{e03c}",
            "Tacos de carne asada \u{e03c}",
            "タコス・カルネ・アサーダ \u{e03c}"
        ]
    );
}

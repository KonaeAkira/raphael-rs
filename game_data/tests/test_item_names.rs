use game_data::{get_item_name, Locale};

#[test]
fn test_44232() {
    let item_id = 44232;
    assert_eq!(
        get_item_name(item_id, true, Locale::EN),
        "Rarefied Tacos de Carne Asada (HQ)"
    );
    assert_eq!(
        get_item_name(item_id, true, Locale::DE),
        "Tacos de Carne Asada (Sammlerstück) (HQ)"
    );
    assert_eq!(
        get_item_name(item_id, true, Locale::FR),
        "Tacos de carne asada collectionnables (HQ)"
    );
    assert_eq!(
        get_item_name(item_id, true, Locale::JP),
        "収集用のタコス・カルネ・アサーダ (HQ)"
    );
}

use game_data::{get_item_name, Locale};

#[test]
fn test_44232() {
    let item_id = 44232;
    assert_eq!(
        get_item_name(item_id, Locale::EN),
        "Rarefied Tacos de Carne Asada"
    );
    assert_eq!(
        get_item_name(item_id, Locale::DE),
        "Tacos de Carne Asada (Sammlerstück)"
    );
    assert_eq!(
        get_item_name(item_id, Locale::FR),
        "Tacos de carne asada collectionnables"
    );
    assert_eq!(
        get_item_name(item_id, Locale::JP),
        "収集用のタコス・カルネ・アサーダ"
    );
}

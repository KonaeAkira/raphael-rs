use game_data::get_simulator_settings;
use simulator::Settings;

fn get_settings(
    item_name: &'static str,
    craftsmanship: u32,
    control: u32,
    job_level: u32,
) -> Result<Settings, Box<dyn std::error::Error>> {
    get_simulator_settings(
        item_name.to_string(),
        craftsmanship,
        control,
        500,
        job_level,
        true,
    )
}

#[test]
fn test_indagator() -> Result<(), Box<dyn std::error::Error>>  {
    let settings = get_settings("Indagator's Saw", 3858, 4057, 90)?;
    assert_eq!(settings.max_progress, 5720);
    assert_eq!(settings.max_quality, 12900);
    assert_eq!(settings.max_durability, 70);
    assert_eq!(settings.base_progress, 239);
    assert_eq!(settings.base_quality, 271);
    Ok(())
}

#[test]
fn test_ironwood() -> Result<(), Box<dyn std::error::Error>> {
    let settings = get_settings("Ironwood Spear", 3500, 3500, 90)?;
    assert_eq!(settings.max_progress, 3100);
    assert_eq!(settings.max_quality, 6800);
    assert_eq!(settings.max_durability, 80);
    assert_eq!(settings.base_progress, 279);
    assert_eq!(settings.base_quality, 353);
    Ok(())
}

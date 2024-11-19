use clap::Args;
use game_data::{get_game_settings, CrafterStats, MEALS, POTIONS, RECIPES};
use simulator::SimulationState;
use solvers::MacroSolver;

#[derive(Args, Debug)]
pub struct SolveArgs {
    /// Item ID
    #[arg(short, long)]
    pub item_id: u32,

    /// Craftsmanship rating
    #[arg(short, long, requires_all(["control", "cp"]), required_unless_present = "stats")]
    pub craftsmanship: Option<u16>,

    /// Control rating
    #[arg(short = 'o', long, requires_all(["craftsmanship", "cp"]), required_unless_present = "stats")]
    pub control: Option<u16>,

    /// Crafting points
    #[arg(short = 'p', long, requires_all(["craftsmanship", "control"]), required_unless_present = "stats")]
    pub cp: Option<u16>,

    /// Complete stats, in the format '<CRAFTSMANSHIP>/<CONTROL>/<CP>'
    #[arg(short, long, value_parser = parse_stats, required_unless_present_all(["craftsmanship", "control", "cp"]), conflicts_with_all(["craftsmanship", "control", "cp"]))]
    pub stats: Option<[u16; 3]>,

    /// Crafter level
    #[arg(short, long, default_value_t = 100)]
    pub level: u8,

    /// Food to use, in the format '<ITEM_ID>[,HQ]'
    #[arg(long, value_parser = parse_consumable)]
    pub food: Option<ConsumableArg>,

    /// Potion to use, in the format '<ITEM_ID>[,HQ]'
    #[arg(long, value_parser = parse_consumable)]
    pub potion: Option<ConsumableArg>,

    /// Enable Manipulation
    #[arg(short, long, default_value_t = false)]
    pub manipulation: bool,

    /// Enable Heart and Soul
    #[arg(long, default_value_t = false)]
    pub heart_and_soul: bool,

    /// Enable Quick Innovation
    #[arg(long, default_value_t = false)]
    pub quick_innovation: bool,

    /// Enable adversarial simulator (ensure 100% reliability)
    #[arg(long, default_value_t = false)]
    pub adversarial: bool,

    /// Only use Progress-increasing actions at the end of the macro
    #[arg(long, default_value_t = false)]
    pub backload_progress: bool,

    /// Enable unsound branch pruning
    #[arg(long, default_value_t = false)]
    pub unsound: bool,
}

fn parse_stats(s: &str) -> Result<[u16; 3], String> {
    const PARSE_ERROR_STRING: &'static str =
        "Stats are not parsable. Stats must have the format '<CRAFTSMANSHIP>/<CONTROL>/<CP>'";
    let segments: Vec<&str> = s.split("/").collect();
    match segments.len() {
        3 => {
            let mut stats: [u16; 3] = [0; 3];
            for i in 0..stats.len() {
                stats[i] = segments
                    .get(i)
                    .unwrap()
                    .parse()
                    .map_err(|_| PARSE_ERROR_STRING.to_owned())?;
            }

            Ok(stats)
        }
        _ => Err(PARSE_ERROR_STRING.to_owned()),
    }
}

fn parse_consumable(s: &str) -> Result<ConsumableArg, String> {
    const PARSE_ERROR_STRING: &'static str =
        "Consumable is not parsable. Consumables must have the format '<ITEM_ID>[,HQ]'";
    let segments: Vec<&str> = s.split(",").collect();
    let item_id_str = segments.get(0);
    let item_id: u32;
    match item_id_str {
        Some(&str) => item_id = str.parse().map_err(|_| PARSE_ERROR_STRING.to_owned())?,
        None => return Err(PARSE_ERROR_STRING.to_owned()),
    }
    match segments.len() {
        1 => Ok(ConsumableArg::NQ(item_id)),
        2 => {
            let hq_str = segments.get(1).unwrap().to_owned();
            match hq_str {
                "HQ" => Ok(ConsumableArg::HQ(item_id)),
                _ => Err(PARSE_ERROR_STRING.to_owned()),
            }
        }
        _ => Err(PARSE_ERROR_STRING.to_owned()),
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ConsumableArg {
    /// NQ Consumable
    NQ(u32),
    /// HQ Consumable
    HQ(u32),
}

pub fn execute(args: &SolveArgs) {
    let recipe = RECIPES
        .iter()
        .find(|r| r.item_id == args.item_id)
        .expect(&format!(
            "Unable to find Recipe for an item with item ID: {}",
            args.item_id
        ));
    let food = match args.food {
        Some(food_arg) => {
            let item_id;
            let is_hq;
            match food_arg {
                ConsumableArg::NQ(id) => {
                    item_id = id;
                    is_hq = false;
                }
                ConsumableArg::HQ(id) => {
                    item_id = id;
                    is_hq = true;
                }
            };

            Some(
                MEALS
                    .iter()
                    .find(|m| (m.item_id == item_id) && (m.hq == is_hq))
                    .expect(&format!("Unable to find Food with item ID: {item_id}"))
                    .to_owned(),
            )
        }
        None => None,
    };
    let potion = match args.potion {
        Some(potion_arg) => {
            let item_id;
            let is_hq;
            match potion_arg {
                ConsumableArg::NQ(id) => {
                    item_id = id;
                    is_hq = false;
                }
                ConsumableArg::HQ(id) => {
                    item_id = id;
                    is_hq = true;
                }
            };

            Some(
                POTIONS
                    .iter()
                    .find(|m| (m.item_id == item_id) && (m.hq == is_hq))
                    .expect(&format!("Unable to find Potion with item ID: {item_id}"))
                    .to_owned(),
            )
        }
        None => None,
    };

    let craftsmanship = match args.craftsmanship {
        Some(stat) => stat,
        None => args.stats.unwrap()[0],
    };
    let control = match args.control {
        Some(stat) => stat,
        None => args.stats.unwrap()[1],
    };
    let cp = match args.cp {
        Some(stat) => stat,
        None => args.stats.unwrap()[2],
    };

    let crafter_stats = CrafterStats {
        craftsmanship: craftsmanship,
        control: control,
        cp: cp,
        level: args.level,
        manipulation: args.manipulation,
        heart_and_soul: args.heart_and_soul,
        quick_innovation: args.quick_innovation,
    };

    let settings = get_game_settings(*recipe, crafter_stats, food, potion, args.adversarial);
    let state = SimulationState::new(&settings);

    let mut solver = MacroSolver::new(
        settings,
        args.backload_progress,
        args.unsound,
        Box::new(|_| {}),
        Box::new(|_| {}),
    );
    let actions = solver.solve(state).expect("Failed to solve");

    let final_state = SimulationState::from_macro(&settings, &actions).unwrap();
    let quality = final_state.quality;
    let steps = actions.len();
    let duration: i16 = actions.iter().map(|action| action.time_cost()).sum();

    println!("Item ID: {}", recipe.item_id);
    println!("Quality: {}/{}", quality, settings.max_quality);
    println!(
        "Progress: {}/{}",
        final_state.progress, settings.max_progress
    );
    println!("Steps: {}", steps);
    println!("Duration: {} seconds", duration);
    println!("\nActions:");
    for action in actions {
        println!("{:?}", action);
    }
}

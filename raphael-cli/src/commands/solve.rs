use clap::Args;
use raphael_data::{CrafterStats, MEALS, POTIONS, RECIPES, get_game_settings};
use raphael_sim::SimulationState;
use raphael_solver::{AtomicFlag, MacroSolver, SolverSettings};

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

    /// Complete stats, conflicts with setting one or more of the stats separately
    #[arg(short, long, num_args = 3, value_names = ["CRAFTSMANSHIP", "CONTROL", "CP"], required_unless_present_all(["craftsmanship", "control", "cp"]), conflicts_with_all(["craftsmanship", "control", "cp"]))]
    pub stats: Vec<u16>,

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

    /// Set initial quality, value is clamped to 100% quality
    #[arg(long, alias = "initial")]
    pub initial_quality: Option<u16>,

    /// Set HQ ingredient amounts and calculate initial quality from them
    #[arg(long, num_args = 1..=6, value_name = "AMOUNT", conflicts_with = "initial_quality")]
    pub hq_ingredients: Option<Vec<u8>>,

    /// Skip mapping HQ ingredients to entries that can actually be HQ and clamping the amount to the max allowed for the recipe
    #[arg(long, default_value_t = false, requires = "hq_ingredients")]
    pub skip_map_and_clamp_hq_ingredients: bool,

    /// Set target quality, value is clamped to 100% quality
    #[arg(long, alias = "target")]
    pub target_quality: Option<u16>,

    /// Enable adversarial simulator (ensure 100% reliability)
    #[arg(long, default_value_t = false)]
    pub adversarial: bool,

    /// Only use Progress-increasing actions at the end of the macro
    #[arg(long, default_value_t = false)]
    pub backload_progress: bool,

    /// Enable unsound branch pruning
    #[arg(long, default_value_t = false)]
    pub unsound: bool,

    /// Output the provided list of variables. The output is deliminated by the output-field-separator
    ///
    /// <IDENTIFIER> can be any of the following: `item_id`, `recipe`, `food`, `potion`, `craftsmanship`, `control`, `cp`, `crafter_stats`, `settings`, `initial_quality`, `target_quality`, `recipe_max_quality`, `actions`, `final_state`, `state_quality`, `final_quality`, `steps`, `duration`.
    /// While the output is mainly intended for generating CSVs, some output can contain `,` inside brackets that are not deliminating columns. For this reason they are wrapped in double quotes and the argument `output-field-separator` can be used to override the delimiter to something that is easier to parse and process
    #[arg(long, num_args = 1.., value_name = "IDENTIFIER")]
    pub output_variables: Vec<String>,

    /// The delimiter the output specified with the argument `output-format` uses to separate identifiers
    #[arg(long, alias = "OFS", default_value = ",", env = "OFS")]
    output_field_separator: String,
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

fn map_and_clamp_hq_ingredients(recipe: &raphael_data::Recipe, hq_ingredients: [u8; 6]) -> [u8; 6] {
    let ingredients: Vec<(raphael_data::Item, u32)> = recipe
        .ingredients
        .iter()
        .filter_map(|ingredient| match ingredient.item_id {
            0 => None,
            id => Some((*raphael_data::ITEMS.get(&id).unwrap(), ingredient.amount)),
        })
        .collect();

    let mut modified_hq_ingredients: [u8; 6] = [0; 6];
    let mut hq_ingredient_index: usize = 0;
    for (index, (item, max_amount)) in ingredients.into_iter().enumerate() {
        if item.can_be_hq {
            modified_hq_ingredients[index] =
                hq_ingredients[hq_ingredient_index].clamp(0, max_amount as u8);
            hq_ingredient_index = hq_ingredient_index.saturating_add(1);
        }
    }

    modified_hq_ingredients
}

pub fn execute(args: &SolveArgs) {
    let recipe = RECIPES
        .values()
        .find(|recipe| recipe.item_id == args.item_id)
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
        None => args.stats.get(0).unwrap().to_owned(),
    };
    let control = match args.control {
        Some(stat) => stat,
        None => args.stats.get(1).unwrap().to_owned(),
    };
    let cp = match args.cp {
        Some(stat) => stat,
        None => args.stats.get(2).unwrap().to_owned(),
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

    let mut settings =
        get_game_settings(*recipe, None, crafter_stats, food, potion, args.adversarial);
    let target_quality = match args.target_quality {
        Some(target) => target.clamp(0, settings.max_quality),
        None => settings.max_quality,
    };
    let initial_quality = match args.initial_quality {
        Some(initial) => initial.clamp(0, settings.max_quality),
        None => match args.hq_ingredients.clone() {
            Some(mut hq_ingredients) => {
                hq_ingredients.resize(6, 0);
                let amount_array = hq_ingredients.try_into().unwrap();
                raphael_data::get_initial_quality(
                    crafter_stats,
                    *recipe,
                    match args.skip_map_and_clamp_hq_ingredients {
                        true => amount_array,
                        false => map_and_clamp_hq_ingredients(recipe, amount_array),
                    },
                )
            }
            None => 0,
        },
    };
    let recipe_max_quality = settings.max_quality;
    settings.max_quality = target_quality.saturating_sub(initial_quality);

    let solver_settings = SolverSettings {
        simulator_settings: settings,
        backload_progress: args.backload_progress,
        allow_unsound_branch_pruning: args.unsound,
    };

    let mut solver = MacroSolver::new(
        solver_settings,
        Box::new(|_| {}),
        Box::new(|_| {}),
        AtomicFlag::new(),
    );
    let actions = solver.solve().expect("Failed to solve");

    let final_state = SimulationState::from_macro(&settings, &actions).unwrap();
    let state_quality = final_state.quality;
    let final_quality = state_quality + u32::from(initial_quality);
    let steps = actions.len();
    let duration: u8 = actions.iter().map(|action| action.time_cost()).sum();

    if args.output_variables.is_empty() {
        println!("Item ID: {}", recipe.item_id);
        println!("Quality: {}/{}", final_quality, recipe_max_quality);
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
    } else {
        let mut output_string = "".to_owned();

        //let output_format = args.output_variables.clone().unwrap();
        //let segments: Vec<&str> = args.output_variables;
        for identifier in &args.output_variables {
            let map_to_debug_str = |actions: Vec<raphael_sim::Action>| match &*(*identifier) {
                "item_id" => format!("{:?}", args.item_id),
                "recipe" => format!("\"{:?}\"", recipe),
                "food" => format!("\"{:?}\"", food),
                "potion" => format!("\"{:?}\"", potion),
                "craftsmanship" => format!("{:?}", craftsmanship),
                "control" => format!("{:?}", control),
                "cp" => format!("{:?}", cp),
                "crafter_stats" => format!("\"{:?}\"", crafter_stats),
                "settings" => format!("\"{:?}\"", settings),
                "initial_quality" => format!("{:?}", initial_quality),
                "target_quality" => format!("{:?}", target_quality),
                "recipe_max_quality" => format!("{:?}", recipe_max_quality),
                "actions" => format!("\"{:?}\"", actions),
                "final_state" => format!("\"{:?}\"", final_state),
                "state_quality" => format!("{:?}", state_quality),
                "final_quality" => format!("{:?}", final_quality),
                "steps" => format!("{:?}", steps),
                "duration" => format!("{:?}", duration),
                _ => "Undefined".to_owned(),
            };

            output_string += &(map_to_debug_str(actions.clone()) + &args.output_field_separator);
        }

        println!(
            "{}",
            output_string.trim_end_matches(&args.output_field_separator)
        );
    }
}

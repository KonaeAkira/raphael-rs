use clap::Args;
use log::error;
use raphael_data::{
    CrafterStats, CustomRecipeOverrides, MEALS, POTIONS, RECIPES, get_game_settings,
};
use raphael_sim::SimulationState;
use raphael_solver::{AtomicFlag, MacroSolver, SolverSettings};

#[derive(Args, Debug)]
pub struct SolveArgs {
    /// Recipe ID
    #[arg(short, long, /*required_unless_present_any(["item_id", "custom_recipe"]),*/ conflicts_with_all(["item_id", "custom_recipe"]))]
    pub recipe_id: Option<u32>,

    /// Item ID, in case multiple recipes for the same item exist, the one with the lowest recipe ID is selected
    #[arg(short, long, /*required_unless_present_any(["recipe_id, custom_recipe"]),*/ conflicts_with = "custom_recipe")]
    pub item_id: Option<u32>,

    /// Custom recipe. <EXPERT> is optional and must be >0 if the custom recipe is expert, if 0 or not provided, the recipe is assumed to not be an expert recipe
    #[arg(long, num_args = 4..=5, value_names = ["RLVL", "PROGRESS", "QUALITY", "DURABILITY", "EXPERT"], /*required_unless_present_any(["recipe_id", "item_id"])*/)]
    pub custom_recipe: Vec<u16>,

    /// Overrides base progress/quality, i.e. "progress/quality per 100% efficiency". rlvl, crafstamnship, and control are ignored if this argument is provided
    #[arg(long, num_args = 3, value_names = ["LEVEL", "BASE_PROGRESS", "BASE_QUALITY"], requires = "custom_recipe")]
    pub override_base_increases: Vec<u16>,

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
    #[arg(long, num_args = 1..=6, value_name = "AMOUNT", conflicts_with_all = ["initial_quality", "custom_recipe"])]
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

    /// Maximum number of threads available to the solver
    #[arg(long)]
    pub threads: Option<usize>,

    /// Output the provided list of variables. The output is deliminated by the output-field-separator
    ///
    /// <IDENTIFIER> can be any of the following: `recipe_id`, `item_id`, `recipe`, `food`, `potion`, `craftsmanship`, `control`, `cp`, `crafter_stats`, `settings`, `initial_quality`, `target_quality`, `recipe_max_quality`, `actions`, `final_state`, `state_quality`, `final_quality`, `steps`, `duration`.
    /// While the output is mainly intended for generating CSVs, some output can contain `,` inside brackets that are not deliminating columns. For this reason they are wrapped in double quotes and the argument `output-field-separator` can be used to override the delimiter to something that is easier to parse and process
    #[arg(long, num_args = 1.., value_name = "IDENTIFIER")]
    pub output_variables: Vec<String>,

    /// The delimiter the output specified with the argument `output-format` uses to separate identifiers
    #[arg(long, alias = "OFS", default_value = ",", env = "OFS")]
    output_field_separator: String,
}

fn parse_consumable(s: &str) -> Result<ConsumableArg, String> {
    const PARSE_ERROR_STRING: &str =
        "Consumable is not parsable. Consumables must have the format '<ITEM_ID>[,HQ]'";
    let segments: Vec<&str> = s.split(",").collect();
    let item_id_str = segments.first();
    let item_id = match item_id_str {
        Some(&str) => str.parse().map_err(|_| PARSE_ERROR_STRING.to_owned())?,
        None => return Err(PARSE_ERROR_STRING.to_owned()),
    };
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
    if args.recipe_id.is_none() && args.item_id.is_none() && args.custom_recipe.is_empty() {
        error!(
            "One of the arguments '--recipe-id', '--item-id', or '--custom-recipe' must be provided"
        );
        panic!();
    }

    if let Some(threads) = args.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .unwrap();
    }

    let use_custom_recipe = !args.custom_recipe.is_empty();
    let mut recipe: raphael_data::Recipe = if use_custom_recipe {
        raphael_data::Recipe {
            job_id: 0,
            item_id: 0,
            max_level_scaling: 0,
            recipe_level: args.custom_recipe[0],
            progress_factor: 0,
            quality_factor: 0,
            durability_factor: 0,
            material_factor: 0,
            ingredients: Default::default(),
            is_expert: match args.custom_recipe.get(4) {
                Some(value) => *value != 0,
                None => false,
            },
            req_craftsmanship: 0,
            req_control: 0,
        }
    } else if args.recipe_id.is_some() {
        *RECIPES.get(&args.recipe_id.unwrap()).expect(&format!(
            "Unable to find Recipe with ID: {}",
            args.recipe_id.unwrap()
        ))
    } else {
        log::warn!(
            "Item IDs do not uniquely corresponds to a specific recipe config. Consider using the recipe ID instead.\nThe first match, i.e. the recipe with the lowest ID, will be selected."
        );
        *RECIPES
            .values()
            .find(|recipe| recipe.item_id == args.item_id.unwrap())
            .expect(&format!(
                "Unable to find Recipe for an item with item ID: {}",
                args.item_id.unwrap()
            ))
    };
    let recipe_id = RECIPES
        .entries()
        .find(|(_, entry_recipe)| **entry_recipe == recipe)
        .map(|(recipe_id, _)| *recipe_id)
        .unwrap_or_default();
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
        None => args.stats[0],
    };
    let control = match args.control {
        Some(stat) => stat,
        None => args.stats[1],
    };
    let cp = match args.cp {
        Some(stat) => stat,
        None => args.stats[2],
    };

    let crafter_stats = CrafterStats {
        craftsmanship,
        control,
        cp,
        level: args.level,
        manipulation: args.manipulation,
        heart_and_soul: args.heart_and_soul,
        quick_innovation: args.quick_innovation,
    };

    let custom_recipe_overrides = if !use_custom_recipe {
        None
    } else if args.override_base_increases.is_empty() {
        Some(CustomRecipeOverrides {
            max_progress_override: args.custom_recipe[1],
            max_quality_override: args.custom_recipe[2],
            max_durability_override: args.custom_recipe[3],
            ..Default::default()
        })
    } else {
        recipe.recipe_level =
            raphael_data::LEVEL_ADJUST_TABLE[args.override_base_increases[0] as usize];
        Some(CustomRecipeOverrides {
            max_progress_override: args.custom_recipe[1],
            max_quality_override: args.custom_recipe[2],
            max_durability_override: args.custom_recipe[3],
            base_progress_override: Some(args.override_base_increases[1]),
            base_quality_override: Some(args.override_base_increases[2]),
        })
    };
    let mut settings =
        get_game_settings(recipe, custom_recipe_overrides, crafter_stats, food, potion);
    settings.adversarial = args.adversarial;
    settings.backload_progress = args.backload_progress;

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
                    recipe,
                    match args.skip_map_and_clamp_hq_ingredients {
                        true => amount_array,
                        false => map_and_clamp_hq_ingredients(&recipe, amount_array),
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
    let action_ids: Vec<u32> = actions.iter().map(|f| f.action_id()).collect();

    if args.output_variables.is_empty() {
        println!("Recipe ID: {}", recipe_id);
        println!(
            "Progress: {}/{}",
            final_state.progress, settings.max_progress
        );
        println!("Quality: {}/{}", final_quality, recipe_max_quality);
        println!(
            "Durability: {}/{}",
            final_state.durability, settings.max_durability
        );
        println!("Steps: {}", steps);
        println!("Duration: {} seconds", duration);
        println!("\nActions:");
        for action in actions {
            println!("{:?}", action);
        }
    } else {
        let mut output_string = "".to_owned();

        for identifier in &args.output_variables {
            let map_to_debug_str = |actions: Vec<raphael_sim::Action>| match &*(*identifier) {
                "recipe_id" => format!("{:?}", recipe_id),
                "item_id" => format!("{:?}", recipe.item_id),
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
                "action_ids" => format!("\"{:?}\"", action_ids),
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

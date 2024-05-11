extends MacroSolverInterface

@export var item_name: String = "Rinascita Sword"
@export var job_level: int = 90
@export var craftsmanship: int = 4021
@export var control: int = 4023
@export var max_cp: int = 596

func update_configuration() -> void:
	configuration["MAX_CP"] = float(max_cp)
	var recipe = %RecipeDatabase.recipes[item_name]
	var rlvl_info = %RecipeDatabase.rlevels[recipe.rlvl]
	configuration["MAX_PROGRESS"] = round(rlvl_info.difficulty * recipe.progress_mod)
	configuration["MAX_QUALITY"] = round(rlvl_info.quality * recipe.quality_mod)
	configuration["MAX_DURABILITY"] = round(rlvl_info.durability * recipe.durability_mod)
	var clvl = %RecipeDatabase.clevels[job_level]
	var base_prog = float(craftsmanship) * 10 / rlvl_info.progress_div + 2
	var base_qual = float(control) * 10 / rlvl_info.quality_div + 35
	if clvl <= recipe.rlvl:
		base_prog = base_prog * rlvl_info.progress_mod
		base_qual = base_qual * rlvl_info.quality_mod
	configuration["PROGRESS_INCREASE"] = floor(base_prog)
	configuration["QUALITY_INCREASE"] = floor(base_qual)
	reset_result()
	
func _ready():
	update_configuration()

func _process(delta: float) -> void:
	check_result()

func _on_solve_button_pressed():
	print(configuration)
	reset_result()
	solve()

func _on_max_cp_input_value_changed(value: float) -> void:
	max_cp = int(value)
	update_configuration()

func _on_control_input_value_changed(value: float) -> void:
	control = int(value)
	update_configuration()

func _on_craftsmanship_input_value_changed(value: float) -> void:
	craftsmanship = int(value)
	update_configuration()

func _on_recipe_search_result_item_activated(index):
	item_name = %RecipeSearchResult.get_item_text(index)
	update_configuration()

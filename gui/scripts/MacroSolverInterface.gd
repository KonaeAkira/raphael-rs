extends MacroSolverInterface

@export var item_name: String = "Indagator's Saw"
@export var craftsmanship: int = 3858
@export var control: int = 4057

func update_configuration() -> void:
	var recipe = %RecipeDatabase.recipes[item_name]
	var rlvl_info = %RecipeDatabase.rlevels[recipe.rlvl]
	setting_max_progress = round(rlvl_info.difficulty * recipe.progress_mod)
	setting_max_quality = round(rlvl_info.quality * recipe.quality_mod)
	setting_max_durability = int(round(rlvl_info.durability * recipe.durability_mod))
	var clvl = %RecipeDatabase.clevels[setting_job_level]
	var base_prog = float(craftsmanship) * 10 / rlvl_info.progress_div + 2
	var base_qual = float(control) * 10 / rlvl_info.quality_div + 35
	if clvl <= recipe.rlvl:
		base_prog = base_prog * rlvl_info.progress_mod
		base_qual = base_qual * rlvl_info.quality_mod
	setting_base_progress = floor(base_prog)
	setting_base_quality = floor(base_qual)
	reset_result()
	
func _ready():
	update_configuration()

func _process(delta: float) -> void:
	check_result()

func _on_solve_button_pressed():
	reset_result()
	solve()

func _on_max_cp_input_value_changed(value: float) -> void:
	setting_max_cp = int(value)
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

func _on_level_input_value_changed(value: float) -> void:
	setting_job_level = int(value)
	update_configuration()

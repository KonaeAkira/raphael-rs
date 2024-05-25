extends MacroSolverInterface
	
func _ready():
	setting_recipe = "Indagator's Saw"
	setting_craftsmanship = 3858
	setting_control = 4057
	setting_job_level = 90
	reset_simulation()

func _process(delta: float) -> void:
	check_result()

func _on_solve_button_pressed():
	reset_simulation()
	solve()

func _on_max_cp_input_value_changed(value: float) -> void:
	setting_max_cp = int(value)
	reset_simulation()

func _on_control_input_value_changed(value: float) -> void:
	setting_control = int(value)
	reset_simulation()

func _on_craftsmanship_input_value_changed(value: float) -> void:
	setting_craftsmanship = int(value)
	reset_simulation()

func _on_recipe_search_result_item_activated(index):
	setting_recipe = %RecipeSearchResult.get_item_text(index)
	reset_simulation()

func _on_level_input_value_changed(value: float) -> void:
	setting_job_level = int(value)
	reset_simulation()

extends MacroSolverInterface


func _process(delta: float) -> void:
	check_result()


func _on_solve_button_pressed():
	reset_result()
	solve()


func _on_max_progress_input_value_changed(value: float) -> void:
	configuration["MAX_PROGRESS"] = value
	reset_result()


func _on_max_quality_input_value_changed(value: float) -> void:
	configuration["MAX_QUALITY"] = value
	reset_result()


func _on_max_durability_input_value_changed(value: float) -> void:
	configuration["MAX_DURABILITY"] = value
	reset_result()


func _on_max_cp_input_value_changed(value: float) -> void:
	configuration["MAX_CP"] = value
	reset_result()


func _on_base_progress_input_value_changed(value: float) -> void:
	configuration["PROGRESS_INCREASE"] = value
	reset_result()


func _on_base_quality_input_value_changed(value: float) -> void:
	configuration["QUALITY_INCREASE"] = value
	reset_result()

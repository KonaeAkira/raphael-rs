extends MacroSolverInterface


# Called when the node enters the scene tree for the first time.
func _ready():
	pass # Replace with function body.


func _on_solve_button_pressed():
	solve()


func _on_max_progress_input_value_changed(value: float) -> void:
	max_progress = int(value)


func _on_max_quality_input_value_changed(value: float) -> void:
	max_quality = int(value)


func _on_max_durability_input_value_changed(value: float) -> void:
	max_durability = int(value)


func _on_max_cp_input_value_changed(value: float) -> void:
	max_cp = int(value)

extends LineEdit

func _on_text_changed(new_text):
	%RecipeSearchResult.clear()
	if new_text.length() < 3:
		return
	for item_name in %RecipeDatabase.item_names:
		if item_name.to_upper().contains(new_text.to_upper()):
			%RecipeSearchResult.add_item(item_name)

extends Node

class Recipe:
	var rlvl: int
	var progress_mod: float
	var quality_mod: float
	var durability_mod: float

class RLevel:
	var difficulty: int
	var quality: int
	var progress_div: float
	var quality_div: float
	var progress_mod: float
	var quality_mod: float
	var durability: int

@export var item_names: Array = []
@export var recipes: Dictionary = {} # item_name > recipe
@export var rlevels: Dictionary = {} # rlvl > rlvl info
@export var clevels: Dictionary = {} # job_level > clvl

func _ready():
	map_item_name_to_rlvl()
	parse_rlevels()
	parse_clevels()
	
func parse_csv(filepath: String, skip_lines: int) -> Array:
	var file = FileAccess.open(filepath, FileAccess.READ)
	for i in range(skip_lines):
		file.get_line()
	var lines = []
	while !file.eof_reached():
		var line = file.get_csv_line()
		if line.size() > 1:
			lines.append(line)
	file.close()
	return lines

func map_item_name_to_rlvl():
	var item_id_to_name: Dictionary = {}
	var items_csv = parse_csv("res://data/Items.txt", 1)
	for row in items_csv:
		var item_id = int(row[0])
		var item_name = str(row[1])
		if item_name != "":
			item_id_to_name[item_id] = item_name
	var recipes_csv = parse_csv("res://data/Recipe.txt", 3)
	for row in recipes_csv:
		var item_id = int(row[4])
		if !item_id_to_name.has(item_id):
			continue
		var item_name = item_id_to_name[item_id]
		item_names.push_back(item_name)
		var recipe = Recipe.new()
		recipe.set("rlvl", int(row[3]))
		recipe.set("progress_mod", float(row[29]) / 100.0)
		recipe.set("quality_mod", float(row[30]) / 100.0)
		recipe.set("durability_mod", float(row[31]) / 100.0)
		recipes[item_name] = recipe
	
func parse_rlevels():
	var rlvls_csv = parse_csv("res://data/rlvls.txt", 1)
	for row in rlvls_csv:
		var rlvl = RLevel.new()
		rlvl.set("difficulty", int(row[1]))
		rlvl.set("quality", int(row[2]))
		rlvl.set("progress_div", float(row[3]))
		rlvl.set("quality_div", float(row[4]))
		rlvl.set("progress_mod", float(row[5]) / 100.0)
		rlvl.set("quality_mod", float(row[6]) / 100.0)
		rlvl.set("durability", int(row[7]))
		rlevels[int(row[0])] = rlvl

func parse_clevels():
	var clvls_csv = parse_csv("res://data/clvls.txt", 1)
	for row in clvls_csv:
		clevels[int(row[0])] = int(row[1])

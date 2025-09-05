pub struct NciArrayBuilder<V> {
    entries_ordered_monotonically_increasing: bool,
    last_added_entry_index: Option<u32>,
    // Values of the format (start_index, skipped_since_last).
    // The skip amounts are relative to the previous range as opposed to the final NciArray where they are absolute
    index_ranges: Vec<(u32, u32)>,
    entries: Vec<(u32, V)>,
}

pub enum OutputFormat {
    RustCodegen,
    RON,
    RONPretty,
}

pub enum ValueFormatting {
    Display,
    Debug,
    DisplayAlternate,
    DebugAlternate,
}

pub struct BuildConfiguration {
    pub output_format: OutputFormat,
    pub value_formatting: ValueFormatting,
}

impl<V: std::fmt::Display + std::fmt::Debug> Default for NciArrayBuilder<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: std::fmt::Display + std::fmt::Debug> NciArrayBuilder<V> {
    pub fn new() -> Self {
        Self {
            entries_ordered_monotonically_increasing: true,
            last_added_entry_index: None,
            index_ranges: vec![],
            entries: vec![],
        }
    }

    pub fn entry(&mut self, index: u32, value: V) {
        if self.entries_ordered_monotonically_increasing {
            if let Some(last_added_entry_index) = self.last_added_entry_index {
                if index > last_added_entry_index {
                    let index_difference = index - last_added_entry_index;
                    if index_difference != 1 {
                        self.index_ranges.push((index, index_difference - 1));
                    }
                } else {
                    if index == last_added_entry_index {
                        log::warn!("Duplicate index `{}` with new value `{:?}`", index, value);
                    }
                    self.entries_ordered_monotonically_increasing = false;
                }
            } else {
                self.index_ranges.push((index, index));
            }

            self.last_added_entry_index = Some(index);
            if !self.entries_ordered_monotonically_increasing {
                self.last_added_entry_index = None;
                self.index_ranges = vec![];
            }
        }

        self.entries.push((index, value));
    }

    fn ensure_output_preconditions(&mut self) {
        if !self.entries_ordered_monotonically_increasing {
            self.entries
                .sort_by(|(first_index, _), (second_index, _)| first_index.cmp(second_index));
            {
                // filter duplicate entries, first entry with index is retained
                let mut expected_index = self.entries.first().unwrap().0;
                self.entries.retain(|(entry_index, _)| {
                    let index_as_expected = *entry_index == expected_index;
                    if index_as_expected {
                        expected_index += 1;
                    }
                    index_as_expected
                });
            }

            for (index, value) in &self.entries {
                if let Some(last_added_entry_index) = self.last_added_entry_index {
                    if *index > last_added_entry_index {
                        let index_difference = index - last_added_entry_index;
                        if index_difference != 1 {
                            self.index_ranges.push((*index, index_difference - 1));
                        }
                    } else {
                        log::warn!("Duplicate index `{}` with new value `{:?}`", index, value);
                    }
                } else {
                    self.index_ranges.push((*index, *index));
                }

                self.last_added_entry_index = Some(*index);
            }

            self.entries_ordered_monotonically_increasing = true;
        }
    }

    pub fn build(&mut self, build_config: BuildConfiguration) -> String {
        use std::fmt::Write as _;

        self.ensure_output_preconditions();

        let (struct_opening_str, struct_closing_str, array_opening_str, array_closing_str) =
            match build_config.output_format {
                OutputFormat::RustCodegen => ("{", "}", "&[", "]"),
                OutputFormat::RON | OutputFormat::RONPretty => ("(", ")", "(", ")"),
            };
        let (new_line_str, indentation_str, space_str) = match build_config.output_format {
            OutputFormat::RON => ("", "", ""),
            _ => ("\n", "\t", " "),
        };

        let mut output_string = format!("{}{new_line_str}", struct_opening_str);

        write!(
            output_string,
            "{indentation_str}index_range_starting_indices:{space_str}{}{new_line_str}",
            array_opening_str
        )
        .unwrap();
        for (i, (starting_index, _)) in self.index_ranges.iter().enumerate() {
            let comma_str = match build_config.output_format {
                OutputFormat::RON => {
                    if i == self.index_ranges.len() - 1 {
                        ""
                    } else {
                        ","
                    }
                }
                _ => ",",
            };
            write!(
                output_string,
                "{indentation_str}{indentation_str}{:?}{comma_str}{new_line_str}",
                *starting_index
            )
            .unwrap();
        }
        write!(
            output_string,
            "{indentation_str}{},{new_line_str}",
            array_closing_str
        )
        .unwrap();

        write!(
            output_string,
            "{indentation_str}index_range_skip_amounts:{space_str}{}{new_line_str}",
            array_opening_str
        )
        .unwrap();
        let mut total_skip_amount = 0;
        for (i, (_, skip_amount)) in self.index_ranges.iter().enumerate() {
            let comma_str = match build_config.output_format {
                OutputFormat::RON => {
                    if i == self.index_ranges.len() - 1 {
                        ""
                    } else {
                        ","
                    }
                }
                _ => ",",
            };
            total_skip_amount += skip_amount;
            write!(
                output_string,
                "{indentation_str}{indentation_str}{:?}{comma_str}{new_line_str}",
                total_skip_amount
            )
            .unwrap();
        }
        write!(
            output_string,
            "{indentation_str}{},{new_line_str}",
            array_closing_str,
        )
        .unwrap();

        write!(
            output_string,
            "{indentation_str}values:{space_str}{}{new_line_str}",
            array_opening_str,
        )
        .unwrap();
        for (i, (_, value)) in self.entries.iter().enumerate() {
            let comma_str = match build_config.output_format {
                OutputFormat::RON => {
                    if i == self.entries.len() - 1 {
                        ""
                    } else {
                        ","
                    }
                }
                _ => ",",
            };
            let entry_str = match build_config.value_formatting {
                ValueFormatting::Display => &format!(
                    "{indentation_str}{indentation_str}{}{comma_str}{new_line_str}",
                    *value
                ),
                ValueFormatting::Debug => &format!(
                    "{indentation_str}{indentation_str}{:?}{comma_str}{new_line_str}",
                    *value
                ),
                ValueFormatting::DisplayAlternate => &format!(
                    "{indentation_str}{indentation_str}{:#}{comma_str}{new_line_str}",
                    *value
                ),
                ValueFormatting::DebugAlternate => &format!(
                    "{indentation_str}{indentation_str}{:#?}{comma_str}{new_line_str}",
                    *value
                ),
            };
            write!(output_string, "{}", entry_str).unwrap();
        }
        let comma_str = match build_config.output_format {
            OutputFormat::RON => "",
            _ => ",",
        };
        write!(
            output_string,
            "{indentation_str}{}{comma_str}{new_line_str}",
            array_closing_str
        )
        .unwrap();
        write!(output_string, "{}", struct_closing_str).unwrap();
        output_string
    }
}

mod recipe;
pub use recipe::Recipe;

mod rlvl;
pub use rlvl::RecipeLevel;

mod item;
pub use item::Item;

pub trait SheetData: Sized {
    const SHEET: &'static str;
    const REQUIRED_FIELDS: &[&str];
    fn row_id(&self) -> u32;
    fn from_json(value: &json::JsonValue) -> Option<Self>;
}

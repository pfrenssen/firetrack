use db::category::Categories;

// Holds the data needed to render a single category in the categories dropdown.
#[derive(Debug, Serialize)]
pub struct CategoryDropdownItem {
    pub id: Option<i32>,
    pub level: u8,
    pub name: String,
}

// A collection of category dropdown items.
#[derive(Debug, Serialize)]
pub struct CategoryDropdownItems {
    pub items: Vec<CategoryDropdownItem>,
}

// Converts the given Categories tree into a list of items suitable for rendering the categories
// dropdown. The root category will be given the name "No category".
impl From<Categories> for CategoryDropdownItems {
    fn from(categories: Categories) -> Self {
        let items = vec![CategoryDropdownItem {
            id: None,
            level: 1,
            name: "No category".to_string(),
        }];

        let items = get_dropdown_items(categories, items, 0);

        CategoryDropdownItems { items }
    }
}

// Recursive function which performs a depth-first transformation of a category tree into a flat
// list of categories.
fn get_dropdown_items(
    categories: Categories,
    mut items: Vec<CategoryDropdownItem>,
    mut level: u8,
) -> Vec<CategoryDropdownItem> {
    level += 1;

    for cat in categories.children {
        items.push(CategoryDropdownItem {
            id: cat.category.clone().map(|c| c.id),
            level,
            name: cat
                .category
                .clone()
                .map(|c| c.name)
                // Apart from the root category there should not be any nameless categories, so we
                // should not see this "Unnamed" category in practice.
                .unwrap_or_else(|| "Unnamed".to_string()),
        });
        items = get_dropdown_items(cat, items, level);
    }

    items
}

use std::env;
use std::process;

use syntect::highlighting::ThemeSet;
use syntect::html::{css_for_theme_with_class_style, ClassStyle};

const CLASS_STYLE: ClassStyle = ClassStyle::SpacedPrefixed { prefix: "cb_" };

fn main() {
    let theme_set = ThemeSet::load_defaults();
    let theme_name = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("No theme specified");
        eprint_available_themes(&theme_set);
        process::exit(1)
    });
    let theme = theme_set.themes.get(&theme_name).unwrap_or_else(|| {
        eprintln!("Theme not found: {}", theme_name);
        eprint_available_themes(&theme_set);
        process::exit(1)
    });
    let css = css_for_theme_with_class_style(theme, CLASS_STYLE);
    println!("{}", css);
}

fn eprint_available_themes(theme_set: &ThemeSet) {
    eprintln!("Available themes:");
    for key in theme_set.themes.keys() {
        eprintln!("  {}", key);
    }
}

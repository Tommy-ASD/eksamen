use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

fn get_file() -> Value {
    let file = std::fs::read_to_string("ønskeliste.json").unwrap();
    let file: serde_json::Value = serde_json::from_str(&file).unwrap();
    file
}

fn set_file(file: &serde_json::Value) {
    let file = serde_json::to_string_pretty(&file).unwrap();
    std::fs::write("ønskeliste.json", file).unwrap();
}

#[derive(Serialize, Deserialize, Debug)]
enum Bought {
    Hidden,
    Visible(bool),
}

impl Default for Bought {
    fn default() -> Self {
        Self::Hidden
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WishListElement {
    pub name: String,
    pub price: f64,
    pub store: String,
    pub link: Option<Url>,
    #[serde(default)]
    bought: Bought,
}

impl WishListElement {
    pub fn new(name: String, price: f64, store: String, link: Option<Url>) -> Self {
        Self {
            name,
            price,
            store,
            link,
            bought: Bought::Hidden,
        }
    }
    pub fn new_from_cli() -> Self {
        let name = crate::input!("Navn: ");
        let mut price = None;
        // this loop will run until price is Some(f64)
        // in other words, until the user inputs a valid f64
        while price.is_none() {
            let mut price_str = crate::input!("Pris: ");
            if !price_str.contains('.') {
                price_str.push_str(".0");
            }
            price = price_str.parse::<f64>().ok();
            // if price is Ok, break the loop
            if let Some(_) = price {
                break;
            } else {
                println!("Pris må være et tall!");
            }
        }
        let price = price.unwrap();
        let store = crate::input!("Butikk: ");
        let add_link = crate::input!("Legg til link? (y/n): ");
        let link = if add_link == "y" {
            let mut link = crate::input!("Link: ");
            if !link.starts_with("http") {
                link = format!("https://{}", link);
            }
            Url::parse(&link).ok()
        } else {
            None
        };
        Self::new(name, price, store, link)
    }
}

impl Display for WishListElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let link = if let Some(link) = &self.link {
            link.to_string()
        } else {
            String::from("Ingen link")
        };
        write!(
            f,
            "
Navn: {}
    Pris: {}
    Butikk: {}
    Link: {}",
            self.name, self.price, self.store, link
        )
    }
}

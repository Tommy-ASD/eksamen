use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::{Database, Namespace, Root, Scope},
    sql::{self, Object, Query, Thing},
    Surreal,
};
#[derive(Debug, Deserialize)]
struct Record {
    #[allow(dead_code)]
    id: Thing,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SrdbWishListElement {
    // id: sql::Thing,
    #[serde(rename = "in")]
    in_: sql::Thing,
    out: sql::Thing,
    name: String,
    price: f64,
    store: String,
    time: i32,
}

use crate::per::WishListElement;

pub mod per;
pub mod utils;

pub static DB: Surreal<Client> = Surreal::init();
pub static NS_STR: &'static str = "test";
pub static DB_STR: &'static str = "test";
pub static SC_STR: &'static str = "account";

#[derive(Debug, Serialize)]
struct AuthParams<'a> {
    name: &'a str,
    pass: &'a str,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // start db as a child process
    // let db = std::process::Command::new("./db.exe")
    //     .arg("start")
    //     .arg("--log")
    //     .arg("debug")
    //     .arg("--user")
    //     .arg("root")
    //     .arg("--pass")
    //     .arg("root")
    //     .arg("file://./db")
    //     .spawn()
    //     .expect("Failed to start database");
    DB.connect::<Ws>("localhost:8000")
        .await
        .expect("Failed to connect to database at localhost:8000");
    DB.use_ns(NS_STR).use_db(DB_STR).await?;
    test().await?;
    return Ok(());
    std::env::set_var("RUST_BACKTRACE", "1");
    loop {
        let input = crate::input!("Sign in or sign up? (in/up): ");
        if input == "in" {
            match signin().await {
                Ok(_) => break,
                Err(e) => println!("Error: {}", e),
            }
        } else if input == "up" {
            match signup().await {
                Ok(_) => break,
                Err(e) => println!("Error: {}", e),
            }
        } else {
            println!("Invalid input");
        }
    }
    after_login().await?;
    Ok(())
}

async fn signup() -> Result<(), Box<dyn std::error::Error>> {
    let name = crate::input!("Username: ");
    let pass = crate::input!("Password: ");
    // Select the namespace/database to use

    // Sign a user in
    DB.signup(Scope {
        namespace: NS_STR,
        database: DB_STR,
        scope: SC_STR,
        params: AuthParams {
            name: &name,
            pass: &pass,
        },
    })
    .await?;
    println!("Signed up as {}", name);
    Ok(())
}

async fn signin() -> Result<(), Box<dyn std::error::Error>> {
    let name = crate::input!("Username: ");
    let pass = crate::input!("Password: ");
    // Select the namespace/database to use

    // Sign a user in
    DB.signin(Scope {
        namespace: NS_STR,
        database: DB_STR,
        scope: SC_STR,
        params: AuthParams {
            name: &name,
            pass: &pass,
        },
    })
    .await?;
    println!("Signed in as {}", name);
    Ok(())
}

async fn after_login() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let input = crate::input!(
            "What do you want to do?
1. Add a wish
2. Read your wishlist
3. Read another user's wishlist
4. Exit"
        );
        match input.as_str() {
            "1" => match write_wishlist().await {
                Ok(_) => println!("Wishes added"),
                Err(e) => println!("Error: {}", e),
            },
            "2" => match read_wishlist().await {
                Ok(_) => println!("Wishes read"),
                Err(e) => println!("Error: {}", e),
            },
            "3" => read_other_wishlist().await?,
            "4" => break,
            _ => println!("Invalid input"),
        }
    }
    Ok(())
}

async fn write_wishlist() -> Result<(), Box<dyn std::error::Error>> {
    let auth_query = format!("SELECT id FROM $auth");
    let auth_query = DB.query(auth_query);
    let mut auth_result = auth_query.await?;
    dbg!(&auth_result);
    let auth_id = auth_result
        .take::<Vec<HashMap<String, Thing>>>(0)?
        .pop()
        .unwrap()
        .get("id")
        .unwrap()
        .clone();
    dbg!(&auth_id);
    let auth_id = format!("{}:{}", auth_id.tb, auth_id.id);
    println!("Add wishes: ");
    loop {
        let element = per::WishListElement::new_from_cli();
        println!("{}", element);
        // add to db
        let created: Option<Record> = match DB.create("item").content(element).await {
            Ok(r) => r,
            Err(e) => None,
        };
        println!("{:?}", created);
        let item_id = match created {
            Some(r) => format!("{}:{}", r.id.tb, r.id.id),
            None => continue,
        };
        println!("item_id: {}", item_id);
        let query = format!("RELATE {auth_id}->wishes_for->{item_id};");
        let query = DB.query(query);
        let result = query.await?;
        dbg!(result);

        let input = crate::input!("Add more wishes? (y/n): ");
        if input == "n" {
            break;
        }
    }
    // let stringified = serde_json::to_string(&wish_list).unwrap();
    // let bytes = stringified.as_bytes();
    // let (encrypted_bytes, padding) = utils::encrypt(key, bytes);
    // fs::write(file_path, encrypted_bytes).unwrap();
    // padding
    Ok(())
}

async fn read_wishlist() -> Result<(), Box<dyn std::error::Error>> {
    let items: Vec<WishListElement> = DB.select("item").await?;
    dbg!(items);
    Ok(())
}

async fn read_other_wishlist() -> Result<(), Box<dyn std::error::Error>> {
    let name = crate::input!("Write the username of the persin who's wishlist you want to see: ");
    let wishes: Vec<SrdbWishListElement> = DB.select("wishes_for").await?;
    dbg!(wishes);
    // let query = "SELECT ->wishes_for->item AS wishes FROM user WHERE name = $name FETCH wishes";
    // let mut result = DB.query(query).bind(("name", &name)).await?;
    // dbg!(&result);
    // let items = result
    //     .take::<Vec<Object>>(0)?
    //     .pop()
    //     .unwrap()
    //     // .get("wishes")
    //     // .unwrap()
    //     .clone();
    // dbg!(&items);

    Ok(())
}

async fn test() -> Result<(), Box<dyn std::error::Error>> {
    DB.signin(Scope {
        namespace: NS_STR,
        database: DB_STR,
        scope: SC_STR,
        params: AuthParams {
            name: "Petter",
            pass: "Pass",
        },
    })
    .await?;
    let wishes: Vec<SrdbWishListElement> = DB.select("item").await?;
    dbg!(wishes);
    // let query = "SELECT ->wishes_for->item AS wishes FROM user WHERE name = \"Per\" FETCH wishes";
    // let mut result = DB.query(query).await?;
    // dbg!(&result);
    // let items = result
    //     .take::<Vec<Object>>(0)?
    //     // .get("wishes")
    //     // .unwrap()
    //     .clone();
    // dbg!(&items);

    Ok(())
}

fn write_padding(key: &str, padding: u8) {
    // read padding_manager.json
    let padding_manager = match std::fs::read_to_string("padding_manager.json") {
        Ok(padding_manager) => padding_manager,
        Err(_) => {
            let padding_manager = serde_json::Value::Object(serde_json::Map::new());
            serde_json::to_string_pretty(&padding_manager).unwrap()
        }
    };
    let mut padding_manager: serde_json::Value = serde_json::from_str(&padding_manager).unwrap();
    // add padding to padding_manager
    padding_manager[key] = serde_json::Value::from(padding);
    // write padding_manager.json
    let padding_manager = serde_json::to_string_pretty(&padding_manager).unwrap();
    std::fs::write("padding_manager.json", padding_manager).unwrap();
}

fn read_padding(key: &str) -> u8 {
    // read padding_manager.json
    let padding_manager = std::fs::read_to_string("padding_manager.json").unwrap();
    let padding_manager: serde_json::Value = serde_json::from_str(&padding_manager).unwrap();
    // get padding from padding_manager
    let padding = match padding_manager.get(key) {
        Some(serde_json::Value::Number(padding)) => padding.as_u64().unwrap() as u8,
        _ => 0,
    };
    padding as u8
}

fn password<'a>() -> Vec<u8> {
    let mut key = crate::input!("Password (16 characters): ")
        .as_bytes()
        .to_vec();
    if key.len() < 16 {
        utils::pad(&mut key);
    }
    let key_ref = key[0..16].to_vec();
    key_ref
}

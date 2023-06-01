use per::EncryptedWishListElement;
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
struct SrdbUserElement {
    // id: sql::Thing,
    // #[serde(rename = "in")]
    // in_: sql::Thing,
    // out: sql::Thing,
    name: String,
    wishes: Vec<WishListElement>, // pass: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
struct SrdbWishList {
    wishes: Vec<WishListElement>,
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
    let test_wish = WishListElement::new("test".to_string(), 100.0, "test".to_string(), None);
    let encr = EncryptedWishListElement::from_unencrypted(test_wish, b"0123456789abcdef");
    println!("{:?}", encr);
    let decr = encr.decrypt(b"0123456789abcdef");
    println!("{:?}", decr);
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
    // test().await?;
    // return Ok(());
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
    let name = crate::input!("Write the username of the person who's wishlist you want to see: ");
    let mut response = DB
        .query(&format!(
            "SELECT ->wishes_for->item AS wishes FROM user WHERE name = \"{name}\" FETCH wishes;"
        ))
        .bind(("name", &name))
        .await?;
    let wishes: Vec<SrdbWishList> = response.take(0)?;
    // get the first element
    let mut wishes: Vec<WishListElement> = wishes
        .first()
        .ok_or("No user with that name")?
        .wishes
        .clone();
    let max_price = crate::input!("What is your max price? (0 for no limit): ");
    let max_price: f64 = max_price.parse()?;
    if max_price != 0.0 {
        wishes = wishes
            .into_iter()
            .filter(|w| w.price <= max_price)
            .collect::<Vec<_>>();
    }
    wishes.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
    wishes.iter().for_each(|w| println!("{}", w));
    Ok(())
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

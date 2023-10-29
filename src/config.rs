// barebones server.properties reader

use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

pub struct Config {
    config: HashMap<String, String>
}

impl Config {
    pub fn parse() -> Self {
        let re = Regex::new(r"[\r\n]+").unwrap();
        let mut text = String::new();
        let mut file = File::open("server.properties").expect("Couldn't find server.properties");
        file.read_to_string(&mut text).expect("server.properties malformed");

        let config: HashMap<String, String> = re.find_iter(&text).filter_map(
            |x| {
                let item = x.as_str();
                if item.starts_with("#") || !item.contains("=") {
                    return None
                }
                let result: Vec<&str> = item.split("=").map(|y| y.trim()).collect();
                Some((result[0].to_string(), result[1].to_string()))
            }
        )
        .collect();

        Self {
            config: config
        }
    }

    pub fn get_property(&self, name: &str) -> &String {
        match self.config.get(&name.to_string()) {
            Some(value) => value,
            None => panic!("Item not found in config: {}", name)
        }
    }
}
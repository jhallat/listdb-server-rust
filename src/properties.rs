use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

pub struct Properties {
    property_map: HashMap<String, String>,
}

impl Properties {
    pub fn new() -> Properties {
        Properties {
            property_map: HashMap::new(),
        }
    }

    pub fn load(&mut self, prop_file_name: &str, args: Vec<String>) {
        self.property_map.clear();
        let file =
            File::open(prop_file_name).expect("I tried, but I can't open the property file.");
        let buffer = BufReader::new(file);

        for property_line in buffer.lines() {
            let contents = property_line.unwrap();
            let property_values: Vec<&str> = contents.split('=').collect();
            self.property_map.insert(
                property_values[0].to_string().trim().to_string(),
                property_values[1].to_string().trim().to_string(),
            );
        }
        for arg in args {
            if arg.trim_start().starts_with("-p") && arg.contains("=") {
                let tokens: Vec<&str> = arg.split("=").collect();
                let property = tokens[0][2..].to_string();
                let value = tokens[1].to_string();
                self.property_map.insert(property, value);
            }
        }
    }

    pub fn get(&self, key: &str) -> String {
        match self.property_map.get(&key.to_string()) {
            Some(value) => value.to_string(),
            None => "".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn load_and_read_properties() {
        let mut properties = Properties::new();
        let args: Vec<String> = Vec::new();
        properties.load("test/resources/test.properties", args);
        let test_one = properties.get("test.one");
        let test_two = properties.get("test.two");
        assert_eq!(test_one, "testone");
        assert_eq!(test_two, "testtwo");
    }

    #[test]
    fn load_properties_from_args() {
        let mut properties = Properties::new();
        let mut args: Vec<String> = Vec::new();
        args.push("-ptest.one=argone".to_string());
        args.push("-ptest.three=argthree".to_string());
        properties.load("test/resources/test.properties", args);
        let test_one = properties.get("test.one");
        let test_two = properties.get("test.two");
        let test_three = properties.get("test.three");
        assert_eq!(test_one, "argone");
        assert_eq!(test_two, "testtwo");
        assert_eq!(test_three, "argthree");
    }
}

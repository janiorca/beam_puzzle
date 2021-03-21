use std::{io::{Read, Write}, path::PathBuf};
use std::fs::File;
use serde::{Deserialize, Serialize};
use super::page_manager;



#[derive(Serialize, Deserialize)]
pub struct Config{
    width: u32,
    height: u32,
    fullscreen: bool,
    max_level: u32
}

impl Config {
    pub fn width( &self ) -> u32 {
        return self.width;
    }
    pub fn height( &self ) -> u32 {
        return self.height;
    }

    pub fn set_full_screen( &mut self, fullscreen: bool ) {
        self.fullscreen = fullscreen;
        self.save();
    }
    pub fn fullscreen( &self ) -> bool {
        return self.fullscreen;
    }

    pub fn increase_max_level( &mut self, level: u32 ) {
        self.max_level = self.max_level.max(level);
        self.save();
    }
    pub fn max_level( &self ) -> u32 {
        return self.max_level;
    }

}

impl Default for Config {
    fn default() -> Self { Config{
        width: page_manager::LOGICAL_WIDTH, height: page_manager::LOGICAL_HEIGHT, fullscreen: false, max_level: 1
    }}
}

impl Config {
    pub fn get_config_file_path() -> PathBuf {
        let app_dirs = platform_dirs::AppDirs::new(Some("beam_puzzle"), true).unwrap();
        let config_file_path = app_dirs.config_dir.join("save_data.json");
        return  config_file_path;
    } 

    pub fn new( ) -> Config {
        println!( "config path: {:?}", Config::get_config_file_path() );

        // Check if it exists and load if it does, otherwise construct one with default values
        let config_path = Config::get_config_file_path();

        let mut config = Config::default();
        if config_path.exists() {
            let res = File::open(config_path);
            if res.is_err() {
                println!( "Failed to open save file {:?} ", res.err() );            
            } else {
                let mut file = res.unwrap();
                let mut buf = String::new();
                let data_result = file.read_to_string(&mut buf);
                if data_result.is_err() {
                    println!( "Failed to read save file");
                }
                let config_result = serde_json::from_str(&buf);
                if config_result.is_err() {
                    println!( "Failed to parse save file");
                }
                config = config_result.unwrap();
            }
        }
        
        return config;
    }

    pub fn save(&self) {
        let config_path = Config::get_config_file_path();
        println!( "saving config to : {:?}", config_path );

        // Does the directory exist
        let app_dirs = platform_dirs::AppDirs::new(Some("beam_puzzle"), true).unwrap();
        if !app_dirs.config_dir.exists() {
            // Create the path
            println!( "Config path does not exist: {:?}\nCreating it", config_path );
            if std::fs::create_dir(app_dirs.config_dir).is_err() {
                println!( "Failed to create config dir" );
            }
        }

        // Serialize it to a JSON string.
        let json_config = serde_json::to_string(&self).unwrap();
        let res = File::create(config_path);
        if res.is_err() {
            println!( "Failed to create save file {:?} ", res.err() );            
        } else {
            let mut file = res.unwrap();
            let write_result = file.write_all(json_config.as_bytes());
            if write_result.is_err() {
                println!( "Failed to write save file");
            }
        }
        
        println!("{}", json_config);        
    }
}

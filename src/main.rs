use std::{error::Error, option::Option};
use schemars::schema::Schema;
use serde::de::DeserializeOwned;
use std::path::Path;
use clap::Parser;

mod deploy;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// deploy configuration
    #[arg(short, long, default_value = "config.yaml")]
    config: Option<String>,

    /// server configuration
    #[arg(short, long)]
    server: Option<String>,

    #[arg(long)]
    test: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let config_file = args.config.unwrap();
    println!("{}", config_file);
    let config_file = config_file.as_str();
    let path = Path::new(config_file);
    assert!(path.exists());
    let mut configure = load_config::<deploy::Configure>(config_file).unwrap();

    let server = args.server;
    match server {
        Some(server) => {
            let server = load_config::<deploy::Server>(server.as_str()).unwrap();
            configure.server = Some(server);
        },
        None => {
            assert!(configure.server.is_some());
        },
    }
    if args.test {
        println!("{:#?}", configure);
    } else {
        deploy::deploy(configure);
    }
    Ok(())
}

fn load_config<T>(path: &str) -> Option<T> where T: DeserializeOwned {
    match serde_yaml::from_str::<Schema>(&std::fs::read_to_string(path).expect(&format!("failure to read file {}", path))) {
        Ok(root_schema) => {
            let data = serde_json::to_string_pretty(&root_schema).expect(&format!("failure to parse yaml from {}", path));
            // println!("{}", data);
            let result = serde_json::from_str::<T>(&data).expect(&format!("failure to format json string {}", data));
            Some(result)
        }
        Err(err) => {
            println!("{}", err);
            None
        }
    }
}


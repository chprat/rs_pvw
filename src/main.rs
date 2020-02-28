use std::env;

fn slider() {
    println!("Slider")
}

fn move_files() {
    println!("Move")
}

fn update_database() {
    println!("Update")
}

fn database_stats() {
    println!("Stats")
}

fn program_settings() {
    println!("Settings")
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        slider();
    } else if args.len() == 2 {
        match args[1].as_str() {
            "update" => update_database(),
            "move" => move_files(),
            "stats" => database_stats(),
            "settings" => program_settings(),
            _ => println!("Unsupported argument"),
        }
    } else {
        println!("Unsupported argument length {}", args.len());
    }
}

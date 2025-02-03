use std::env::args;

fn main() -> Result<(), iqair::Error> {
    if matches!(
        args().nth(1).as_deref(),
        Some("--help" | "-h" | "help" | "h")
    ) {
        println!("{} (KEY) (raw)", args().next().unwrap());
        println!("provides iqair data for nearest city");
        println!("data refreshes hourly");
        return Ok(());
    }
    let Some(key) = args().nth(1).or(std::env::var("AIR_KEY").ok()) else {
        println!("\x1b[31;1mno key provided!\x1b[0m");
        println!("get one at https://dashboard.iqair.com/personal/api-keys for free");
        println!("then set AIR_KEY or pass it to this binary");
        std::process::exit(0);
    };
    let data = iqair::nearest(&key)?;
    if args().nth(1).as_deref() == Some("raw") || args().nth(2).as_deref() == Some("raw") {
        println!("{}", serde_json::to_string_pretty(&data).unwrap());
    } else {
        println!("{}", data.current.pollution.aqi_us)
    }
    Ok(())
}

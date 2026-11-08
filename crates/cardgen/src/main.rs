mod app;
mod card;
mod generate;
mod model;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Headless round-trip path: `cardgen --generate "prompt"` runs the full
    // prompt → model → card loop without a GPU/display and prints the card.
    if args.len() >= 3 && args[1] == "--generate" {
        match crate::generate::generate_and_print(&args[2]) {
            Ok(json) => println!("{json}"),
            Err(error) => {
                eprintln!("error: {error:#}");
                std::process::exit(1);
            }
        }
        return;
    }

    app::run();
}

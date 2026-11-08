mod app;
mod generate;
mod model;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Headless round-trip path: `cardgen --generate "prompt"` runs the full
    // prompt → model → card loop without a GPU/display and prints the card.
    // `--spec=v3` emits the canonical V3 envelope; default is V2 (import-compat).
    if args.len() >= 3 && args[1] == "--generate" {
        let v3 = args.len() >= 4 && args[3] == "--spec=v3";
        match crate::generate::generate_and_print(&args[2], v3) {
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

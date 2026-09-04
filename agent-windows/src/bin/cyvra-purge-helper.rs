//! Elevated Mode S helper. No network. No shell. Plan in, result out.

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut plan = None;
    let mut result = None;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--plan" => {
                plan = args.get(index + 1).cloned();
                index += 2;
            }
            "--result" => {
                result = args.get(index + 1).cloned();
                index += 2;
            }
            _ => index += 1,
        }
    }
    let Some(plan) = plan else {
        eprintln!("CYVRA Purge helper requires --plan");
        std::process::exit(2);
    };
    let Some(result) = result else {
        eprintln!("CYVRA Purge helper requires --result");
        std::process::exit(2);
    };
    let code =
        cyvra_core::purge::helper_main(std::path::Path::new(&plan), std::path::Path::new(&result));
    std::process::exit(code);
}

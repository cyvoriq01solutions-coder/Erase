fn main() {
    let scan = cyvra_core::run_scan();
    println!("{}", scan.render_json());
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let exit_code = su_profile_switcher_lib::cli::run_cli(&args);
    std::process::exit(exit_code);
}

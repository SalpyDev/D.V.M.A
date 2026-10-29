use std::process::ExitCode;

fn main() -> ExitCode {
    if let Err(err) = dvma::ui::run() {
        eprintln!("❌ Error: {}", err);
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    name: Option<String>,
}

#[tokio::main]
async fn main() {
    let _cli = Cli::parse();
    println!("Hello from mpca!");
}

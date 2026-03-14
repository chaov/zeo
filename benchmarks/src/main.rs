mod lib;
mod reporting;
mod monitoring;

use std::fs;
use std::path::Path;
use lib::BenchmarkRunner;
use reporting::ReportGenerator;

fn main() -> Result<(), String> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║          Zeo Performance Benchmark Framework v0.1.0         ║");
    println!("║                    Performance vs Bun                       ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    let config_path = Path::new("config.toml");
    if !config_path.exists() {
        return Err("config.toml not found. Please create a configuration file.".to_string());
    }

    let config_content = fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let config: lib::BenchmarkConfig = toml::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;

    let mut runner = BenchmarkRunner::new(config.clone());

    println!("Starting benchmark suite...");
    println!("Target: 50%+ performance improvement over Bun");
    println!();

    runner.run_all()?;

    println!("Benchmark suite completed!");
    println!("Generating reports...");
    println!();

    let comparisons = runner.compare_with_bun();

    let report_generator = ReportGenerator::new(config.reporting.output_dir.clone());

    if config.reporting.format.contains(&"json".to_string()) {
        report_generator.generate_json_report(&runner, &comparisons)?;
    }

    if config.reporting.format.contains(&"markdown".to_string()) {
        report_generator.generate_markdown_report(&runner, &comparisons)?;
    }

    if config.reporting.format.contains(&"html".to_string()) {
        report_generator.generate_html_report(&runner, &comparisons)?;
    }

    println!();
    println!("═════════════════════════════════════════════════════════════");
    println!("                    Benchmark Summary");
    println!("═════════════════════════════════════════════════════════════");

    let goals_met = comparisons.iter().filter(|c| c.goal_met).count();
    let total_goals = comparisons.len();

    println!("Goals Met: {}/{}", goals_met, total_goals);

    for comp in &comparisons {
        let improvement_pct = (comp.bun_value / comp.zeo_value - 1.0) * 100.0;
        let status = if comp.goal_met { "✅" } else { "❌" };
        println!("{} {}: {:.1}% faster", status, comp.scenario, improvement_pct);
    }

    println!();
    println!("Reports generated in: {}", config.reporting.output_dir);
    println!("═════════════════════════════════════════════════════════════");

    Ok(())
}
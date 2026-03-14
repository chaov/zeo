use std::fs;
use std::path::Path;
use serde_json;
use crate::{BenchmarkRunner, ComparisonResult};

pub struct ReportGenerator {
    output_dir: String,
}

impl ReportGenerator {
    pub fn new(output_dir: String) -> Self {
        fs::create_dir_all(&output_dir).unwrap();
        ReportGenerator { output_dir }
    }

    pub fn generate_json_report(&self, runner: &BenchmarkRunner, comparisons: &[ComparisonResult]) -> Result<(), String> {
        let report = serde_json::json!({
            "benchmark": {
                "name": "zeo-benchmark",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "results": runner.get_results(),
                "comparisons": comparisons
            }
        });

        let output_path = Path::new(&self.output_dir).join("benchmark_report.json");
        fs::write(output_path, serde_json::to_string_pretty(&report).unwrap())
            .map_err(|e| format!("Failed to write JSON report: {}", e))?;

        println!("JSON report generated: {}/benchmark_report.json", self.output_dir);
        Ok(())
    }

    pub fn generate_markdown_report(&self, runner: &BenchmarkRunner, comparisons: &[ComparisonResult]) -> Result<(), String> {
        let mut markdown = String::new();

        markdown.push_str("# Zeo Performance Benchmark Report\n\n");
        markdown.push_str(&format!("**Generated:** {}\n\n", chrono::Utc::now().to_rfc3339()));
        markdown.push_str("---\n\n");

        markdown.push_str("## Executive Summary\n\n");
        
        let goals_met = comparisons.iter().filter(|c| c.goal_met).count();
        let total_goals = comparisons.len();
        
        markdown.push_str(&format!("**Performance Goals Met:** {}/{}\n\n", goals_met, total_goals));
        
        if goals_met == total_goals {
            markdown.push_str("✅ **All performance goals achieved!** Zeo outperforms Bun by 50%+ in all tested scenarios.\n\n");
        } else {
            markdown.push_str(&format!("⚠️ **Partial success:** {}/{} scenarios meet the 50%+ improvement goal.\n\n", goals_met, total_goals));
        }

        markdown.push_str("## Performance Comparison vs Bun\n\n");
        markdown.push_str("| Scenario | Zeo | Bun | Improvement | Goal Met |\n");
        markdown.push_str("|----------|-----|-----|-------------|----------|\n");

        for comp in comparisons {
            let improvement_pct = (comp.improvement_ratio - 1.0) * 100.0;
            let goal_icon = if comp.goal_met { "✅" } else { "❌" };
            
            markdown.push_str(&format!(
                "| {} | {:.2} {} | {:.2} {} | {:.1}% | {} |\n",
                comp.scenario, comp.zeo_value, comp.unit, comp.bun_value, comp.unit, improvement_pct, goal_icon
            ));
        }

        markdown.push_str("\n## Detailed Results\n\n");

        let scenarios = vec![
            "startup_time",
            "javascript_execution", 
            "module_loading",
            "file_io",
            "network_requests",
            "ai_agent_execution",
        ];

        for scenario in &scenarios {
            markdown.push_str(&format!("### {}\n\n", self.format_scenario_name(scenario)));

            let results: Vec<_> = runner.get_results().iter()
                .filter(|r| r.scenario == *scenario)
                .collect();

            if !results.is_empty() {
                markdown.push_str("| Target | Metric | Value | Unit |\n");
                markdown.push_str("|--------|--------|-------|------|\n");

                for result in results {
                    markdown.push_str(&format!(
                        "| {} | {} | {:.2} | {} |\n",
                        result.target, result.metric, result.value, result.unit
                    ));
                }
                markdown.push_str("\n");
            }
        }

        markdown.push_str("## Performance Analysis\n\n");
        markdown.push_str("### Key Findings\n\n");

        for comp in comparisons {
            let improvement_pct = (comp.improvement_ratio - 1.0) * 100.0;
            
            if comp.goal_met {
                markdown.push_str(&format!(
                    "- **{}**: Zeo is {:.1}% faster than Bun ✅\n",
                    self.format_scenario_name(&comp.scenario), improvement_pct
                ));
            } else {
                markdown.push_str(&format!(
                    "- **{}**: Zeo is {:.1}% faster than Bun (Goal: 50%+) ⚠️\n",
                    self.format_scenario_name(&comp.scenario), improvement_pct
                ));
            }
        }

        markdown.push_str("\n### Recommendations\n\n");

        let failed_goals: Vec<_> = comparisons.iter().filter(|c| !c.goal_met).collect();
        if failed_goals.is_empty() {
            markdown.push_str("All performance targets have been met. Zeo is ready for production deployment.\n");
        } else {
            markdown.push_str("The following scenarios need optimization:\n\n");
            for comp in failed_goals {
                markdown.push_str(&format!(
                    "- {}: Current improvement {:.1}%, Target: 50%+\n",
                    self.format_scenario_name(&comp.scenario),
                    (comp.improvement_ratio - 1.0) * 100.0
                ));
            }
        }

        let output_path = Path::new(&self.output_dir).join("benchmark_report.md");
        fs::write(output_path, markdown)
            .map_err(|e| format!("Failed to write Markdown report: {}", e))?;

        println!("Markdown report generated: {}/benchmark_report.md", self.output_dir);
        Ok(())
    }

    pub fn generate_html_report(&self, runner: &BenchmarkRunner, comparisons: &[ComparisonResult]) -> Result<(), String> {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang=\"en\">\n");
        html.push_str("<head>\n");
        html.push_str("    <meta charset=\"UTF-8\">\n");
        html.push_str("    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        html.push_str("    <title>Zeo Performance Benchmark Report</title>\n");
        html.push_str("    <style>\n");
        html.push_str("        body { font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 0; padding: 20px; background: #f1f1f1; }\n");
        html.push_str("        .container { max-width: 1200px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }\n");
        html.push_str("        h1 { color: #2c3e50; border-bottom: 3px solid #3498db; padding-bottom: 10px; }\n");
        html.push_str("        h2 { color: #34495e; margin-top: 30px; }\n");
        html.push_str("        .summary { background: #ecf0f1; padding: 20px; border-radius: 5px; margin: 20px 0; }\n");
        html.push_str("        .success { color: #27ae60; font-weight: bold; }\n");
        html.push_str("        .warning { color: #e67e22; font-weight: bold; }\n");
        html.push_str("        table { width: 100%; border-collapse: collapse; margin: 20px 0; }\n");
        html.push_str("        th, td { padding: 12px; text-align: left; border-bottom: 1px solid #ddd; }\n");
        html.push_str("        th { background: #3498db; color: white; }\n");
        html.push_str("        tr:hover { background: #f5f5f5; }\n");
        html.push_str("        .goal-met { color: #27ae60; }\n");
        html.push_str("        .goal-not-met { color: #e74c3c; }\n");
        html.push_str("        .chart-container { margin: 30px 0; }\n");
        html.push_str("        .metric-card { background: #f8f9fa; padding: 15px; margin: 10px 0; border-radius: 5px; border-left: 4px solid #3498db; }\n");
        html.push_str("    </style>\n");
        html.push_str("</head>\n");
        html.push_str("<body>\n");
        html.push_str("    <div class=\"container\">\n");
        html.push_str("        <h1>🚀 Zeo Performance Benchmark Report</h1>\n");
        html.push_str(&format!("        <p><strong>Generated:</strong> {}</p>\n", chrono::Utc::now().to_rfc3339()));

        let goals_met = comparisons.iter().filter(|c| c.goal_met).count();
        let total_goals = comparisons.len();

        html.push_str("        <div class=\"summary\">\n");
        html.push_str(&format!("            <h2>Executive Summary</h2>\n"));
        html.push_str(&format!("            <p><strong>Performance Goals Met:</strong> {}/{}</p>\n", goals_met, total_goals));
        
        if goals_met == total_goals {
            html.push_str("            <p class=\"success\">✅ All performance goals achieved! Zeo outperforms Bun by 50%+ in all tested scenarios.</p>\n");
        } else {
            html.push_str(&format!("            <p class=\"warning\">⚠️ Partial success: {}/{} scenarios meet the 50%+ improvement goal.</p>\n", goals_met, total_goals));
        }
        html.push_str("        </div>\n");

        html.push_str("        <h2>Performance Comparison vs Bun</h2>\n");
        html.push_str("        <table>\n");
        html.push_str("            <thead>\n");
        html.push_str("                <tr>\n");
        html.push_str("                    <th>Scenario</th>\n");
        html.push_str("                    <th>Zeo</th>\n");
        html.push_str("                    <th>Bun</th>\n");
        html.push_str("                    <th>Improvement</th>\n");
        html.push_str("                    <th>Goal Met</th>\n");
        html.push_str("                </tr>\n");
        html.push_str("            </thead>\n");
        html.push_str("            <tbody>\n");

        for comp in comparisons {
            let improvement_pct = (comp.improvement_ratio - 1.0) * 100.0;
            let goal_class = if comp.goal_met { "goal-met" } else { "goal-not-met" };
            let goal_icon = if comp.goal_met { "✅" } else { "❌" };
            
            html.push_str(&format!(
                "                <tr>\n");
            html.push_str(&format!(
                "                    <td>{}</td>\n", self.format_scenario_name(&comp.scenario)));
            html.push_str(&format!(
                "                    <td>{:.2} {}</td>\n", comp.zeo_value, comp.unit));
            html.push_str(&format!(
                "                    <td>{:.2} {}</td>\n", comp.bun_value, comp.unit));
            html.push_str(&format!(
                "                    <td>{:.1}%</td>\n", improvement_pct));
            html.push_str(&format!(
                "                    <td class=\"{}\">{}</td>\n", goal_class, goal_icon));
            html.push_str("                </tr>\n");
        }

        html.push_str("            </tbody>\n");
        html.push_str("        </table>\n");

        html.push_str("        <h2>Performance Analysis</h2>\n");
        html.push_str("        <div class=\"chart-container\">\n");

        for comp in comparisons {
            let improvement_pct = (comp.improvement_ratio - 1.0) * 100.0;
            let bar_width = std::cmp::min(improvement_pct as i32, 100);
            let bar_color = if comp.goal_met { "#27ae60" } else { "#e74c3c" };
            
            html.push_str("            <div class=\"metric-card\">\n");
            html.push_str(&format!("                <h3>{}</h3>\n", self.format_scenario_name(&comp.scenario)));
            html.push_str(&format!("                <p><strong>Zeo:</strong> {:.2} {} | <strong>Bun:</strong> {:.2} {}</p>\n", 
                comp.zeo_value, comp.unit, comp.bun_value, comp.unit));
            html.push_str(&format!("                <p><strong>Improvement:</strong> {:.1}%</p>\n", improvement_pct));
            html.push_str(&format!("                <div style=\"background: #eee; height: 20px; border-radius: 10px; overflow: hidden;\">\n"));
            html.push_str(&format!("                    <div style=\"background: {}; width: {}%; height: 100%;\"></div>\n", bar_color, bar_width));
            html.push_str("                </div>\n");
            html.push_str("            </div>\n");
        }

        html.push_str("        </div>\n");
        html.push_str("        <h2>Recommendations</h2>\n");

        let failed_goals: Vec<_> = comparisons.iter().filter(|c| !c.goal_met).collect();
        if failed_goals.is_empty() {
            html.push_str("        <p class=\"success\">All performance targets have been met. Zeo is ready for production deployment.</p>\n");
        } else {
            html.push_str("        <p>The following scenarios need optimization:</p>\n");
            html.push_str("        <ul>\n");
            for comp in failed_goals {
                html.push_str(&format!(
                    "            <li><strong>{}</strong>: Current improvement {:.1}%, Target: 50%+</li>\n",
                    self.format_scenario_name(&comp.scenario),
                    (comp.improvement_ratio - 1.0) * 100.0
                ));
            }
            html.push_str("        </ul>\n");
        }

        html.push_str("    </div>\n");
        html.push_str("</body>\n");
        html.push_str("</html>\n");

        let output_path = Path::new(&self.output_dir).join("benchmark_report.html");
        fs::write(output_path, html)
            .map_err(|e| format!("Failed to write HTML report: {}", e))?;

        println!("HTML report generated: {}/benchmark_report.html", self.output_dir);
        Ok(())
    }

    fn format_scenario_name(&self, scenario: &str) -> String {
        match scenario {
            "startup_time" => "Startup Time".to_string(),
            "javascript_execution" => "JavaScript Execution".to_string(),
            "module_loading" => "Module Loading".to_string(),
            "file_io" => "File I/O".to_string(),
            "network_requests" => "Network Requests".to_string(),
            "ai_agent_execution" => "AI Agent Execution".to_string(),
            _ => scenario.to_string(),
        }
    }
}
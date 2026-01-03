//! ValidatereportGenerator
//!
//! for GeneratedetailedValidate and testreport

use std::fs::File;
use std::io::Write;

/// Validatereport
#[derive(Debug, Clone)]
pub struct ValidationReport {
 pub title: String,
 pub generated_at: String,
 pub sections: Vec<ReportSection>,
 pub summary: ReportSummary,
}

/// reportsection
#[derive(Debug, Clone)]
pub struct ReportSection {
 pub title: String,
 pub content: Vec<String>,
 pub subsections: Vec<ReportSection>,
}

/// reportdigest
#[derive(Debug, Clone)]
pub struct ReportSummary {
 pub total_tests: usize,
 pub passed: usize,
 pub failed: usize,
 pub success_rate: f64,
}

impl ValidationReport {
 /// Createnewreport
 pub fn new(title: String) -> Self {
 #[cfg(feature = "reporter")]
 let generated_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
 #[cfg(not(feature = "reporter"))]
 let generated_at = std::time::SystemTime::now()
.duration_since(std::time::UNIX_EPOCH)
.map(|d| format!("{}", d.as_secs()))
.unwrap_or_else(|_| "N/A".to_string());

 Self {
 title,
 generated_at,
 sections: Vec::new(),
 summary: ReportSummary {
 total_tests: 0,
 passed: 0,
 failed: 0,
 success_rate: 0.0,
 },
 }
 }

 /// Addsection
 pub fn add_section(&mut self, section: ReportSection) {
 self.sections.push(section);
 }

 /// settingsdigest
 pub fn set_summary(&mut self, total: usize, passed: usize, failed: usize) {
 self.summary = ReportSummary {
 total_tests: total,
 passed,
 failed,
 success_rate: if total > 0 {
 (passed as f64 / total as f64) * 100.0
 } else {
 0.0
 },
 };
 }

 /// Generate Markdown formatreport
 pub fn to_markdown(&self) -> String {
 let mut md = String::new();

 // title
 md.push_str(&format!("# {}\n\n", self.title));
 md.push_str(&format!("**Generate when between**: {}\n\n", self.generated_at));
 md.push_str("---\n\n");

 // digest
 md.push_str("## 📊 testdigest\n\n");
 md.push_str(&format!("- **总testcount**: {}\n", self.summary.total_tests));
 md.push_str(&format!("- **through**: {} ✅\n", self.summary.passed));
 md.push_str(&format!("- **failure**: {} ❌\n", self.summary.failed));
 md.push_str(&format!(
 "- **success率**: {:.2}%\n\n",
 self.summary.success_rate
 ));
 md.push_str("---\n\n");

 // eachsection
 for section in &self.sections {
 md.push_str(&section.to_markdown(2));
 }

 md
 }

 /// Generate纯textreport
 pub fn to_text(&self) -> String {
 let mut text = String::new();

 // title
 text.push_str(&format!("# {}\n\n", self.title));
 text.push_str(&format!("Generate when between: {}\n", self.generated_at));
 text.push_str(&"=".repeat(70));
 text.push_str("\n\n");

 // digest
 text.push_str("testdigest:\n");
 text.push_str(&format!(" 总testcount: {}\n", self.summary.total_tests));
 text.push_str(&format!(" through: {}\n", self.summary.passed));
 text.push_str(&format!(" failure: {}\n", self.summary.failed));
 text.push_str(&format!(" success率: {:.2}%\n\n", self.summary.success_rate));
 text.push_str(&"=".repeat(70));
 text.push_str("\n\n");

 // eachsection
 for section in &self.sections {
 text.push_str(&section.to_text(0));
 }

 text
 }

 /// save as file
 pub fn save_to_file(&self, filename: &str, format: ReportFormat) -> std::io::Result<()> {
 let content = match format {
 ReportFormat::Markdown => self.to_markdown(),
 ReportFormat::Text => self.to_text(),
 };

 let mut file = File::create(filename)?;
 file.write_all(content.as_bytes())?;

 Ok(())
 }
}

impl ReportSection {
 /// Createnewsection
 pub fn new(title: String) -> Self {
 Self {
 title,
 content: Vec::new(),
 subsections: Vec::new(),
 }
 }

 /// Addinside容execute
 pub fn add_line(&mut self, line: String) {
 self.content.push(line);
 }

 /// Addchildsection
 pub fn add_subsection(&mut self, subsection: ReportSection) {
 self.subsections.push(subsection);
 }

 /// convert to Markdown
 fn to_markdown(&self, level: usize) -> String {
 let mut md = String::new();

 // sectiontitle
 md.push_str(&"#".repeat(level));
 md.push_str(&format!(" {}\n\n", self.title));

 // inside容
 for line in &self.content {
 md.push_str(line);
 md.push('\n');
 }
 if !self.content.is_empty() {
 md.push('\n');
 }

 // childsection
 for subsection in &self.subsections {
 md.push_str(&subsection.to_markdown(level + 1));
 }

 md
 }

 /// convert to纯text
 fn to_text(&self, indent: usize) -> String {
 let mut text = String::new();
 let indent_str = " ".repeat(indent);

 // sectiontitle
 text.push_str(&format!("{}{}\n", indent_str, self.title));
 text.push_str(&format!("{}{}\n", indent_str, "-".repeat(self.title.len())));

 // inside容
 for line in &self.content {
 text.push_str(&format!("{} {}\n", indent_str, line));
 }
 if !self.content.is_empty() {
 text.push('\n');
 }

 // childsection
 for subsection in &self.subsections {
 text.push_str(&subsection.to_text(indent + 1));
 }

 text
 }
}

/// reportformat
#[derive(Debug, Clone, Copy)]
pub enum ReportFormat {
 Markdown,
 Text,
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_report_creation() {
 let mut report = ValidationReport::new("Test Report".to_string());
 report.set_summary(100, 95, 5);

 assert_eq!(report.summary.total_tests, 100);
 assert_eq!(report.summary.passed, 95);
 assert_eq!(report.summary.success_rate, 95.0);
 }

 #[test]
 fn test_section_creation() {
 let mut section = ReportSection::new("Test Section".to_string());
 section.add_line("Line 1".to_string());
 section.add_line("Line 2".to_string());

 assert_eq!(section.content.len(), 2);
 }

 #[test]
 fn test_markdown_generation() {
 let mut report = ValidationReport::new("Test Report".to_string());
 report.set_summary(10, 9, 1);

 let md = report.to_markdown();
 assert!(md.contains("# Test Report"));
 // Checksuccess率field exists (not mandatoryrequirepreciseformat)
 assert!(md.contains("success率") || md.contains("Success"));
 assert!(md.contains("90."));
 }
}

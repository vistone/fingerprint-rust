//! 验证报告生成器
//!
//! 用于生成详细的验证和测试报告

use std::fs::File;
use std::io::Write;

/// 验证报告
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub title: String,
    pub generated_at: String,
    pub sections: Vec<ReportSection>,
    pub summary: ReportSummary,
}

/// 报告章节
#[derive(Debug, Clone)]
pub struct ReportSection {
    pub title: String,
    pub content: Vec<String>,
    pub subsections: Vec<ReportSection>,
}

/// 报告摘要
#[derive(Debug, Clone)]
pub struct ReportSummary {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub success_rate: f64,
}

impl ValidationReport {
    /// 创建新报告
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

    /// 添加章节
    pub fn add_section(&mut self, section: ReportSection) {
        self.sections.push(section);
    }

    /// 设置摘要
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

    /// 生成 Markdown 格式报告
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        // 标题
        md.push_str(&format!("# {}\n\n", self.title));
        md.push_str(&format!("**生成时间**: {}\n\n", self.generated_at));
        md.push_str("---\n\n");

        // 摘要
        md.push_str("## 📊 测试摘要\n\n");
        md.push_str(&format!("- **总测试数**: {}\n", self.summary.total_tests));
        md.push_str(&format!("- **通过**: {} ✅\n", self.summary.passed));
        md.push_str(&format!("- **失败**: {} ❌\n", self.summary.failed));
        md.push_str(&format!(
            "- **成功率**: {:.2}%\n\n",
            self.summary.success_rate
        ));
        md.push_str("---\n\n");

        // 各个章节
        for section in &self.sections {
            md.push_str(&section.to_markdown(2));
        }

        md
    }

    /// 生成纯文本报告
    pub fn to_text(&self) -> String {
        let mut text = String::new();

        // 标题
        text.push_str(&format!("# {}\n\n", self.title));
        text.push_str(&format!("生成时间: {}\n", self.generated_at));
        text.push_str(&"=".repeat(70));
        text.push_str("\n\n");

        // 摘要
        text.push_str("测试摘要:\n");
        text.push_str(&format!("  总测试数: {}\n", self.summary.total_tests));
        text.push_str(&format!("  通过: {}\n", self.summary.passed));
        text.push_str(&format!("  失败: {}\n", self.summary.failed));
        text.push_str(&format!("  成功率: {:.2}%\n\n", self.summary.success_rate));
        text.push_str(&"=".repeat(70));
        text.push_str("\n\n");

        // 各个章节
        for section in &self.sections {
            text.push_str(&section.to_text(0));
        }

        text
    }

    /// 保存为文件
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
    /// 创建新章节
    pub fn new(title: String) -> Self {
        Self {
            title,
            content: Vec::new(),
            subsections: Vec::new(),
        }
    }

    /// 添加内容行
    pub fn add_line(&mut self, line: String) {
        self.content.push(line);
    }

    /// 添加子章节
    pub fn add_subsection(&mut self, subsection: ReportSection) {
        self.subsections.push(subsection);
    }

    /// 转换为 Markdown
    fn to_markdown(&self, level: usize) -> String {
        let mut md = String::new();

        // 章节标题
        md.push_str(&"#".repeat(level));
        md.push_str(&format!(" {}\n\n", self.title));

        // 内容
        for line in &self.content {
            md.push_str(line);
            md.push('\n');
        }
        if !self.content.is_empty() {
            md.push('\n');
        }

        // 子章节
        for subsection in &self.subsections {
            md.push_str(&subsection.to_markdown(level + 1));
        }

        md
    }

    /// 转换为纯文本
    fn to_text(&self, indent: usize) -> String {
        let mut text = String::new();
        let indent_str = "  ".repeat(indent);

        // 章节标题
        text.push_str(&format!("{}{}\n", indent_str, self.title));
        text.push_str(&format!("{}{}\n", indent_str, "-".repeat(self.title.len())));

        // 内容
        for line in &self.content {
            text.push_str(&format!("{}  {}\n", indent_str, line));
        }
        if !self.content.is_empty() {
            text.push('\n');
        }

        // 子章节
        for subsection in &self.subsections {
            text.push_str(&subsection.to_text(indent + 1));
        }

        text
    }
}

/// 报告格式
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
        // 检查成功率字段存在（不强制要求精确格式）
        assert!(md.contains("成功率") || md.contains("Success"));
        assert!(md.contains("90."));
    }
}

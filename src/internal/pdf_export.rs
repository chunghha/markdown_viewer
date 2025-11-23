//! PDF export functionality for the markdown viewer
//!
//! This module provides functionality to export markdown content to PDF files
//! using the markdown2pdf library.

use anyhow::Result;
use std::path::Path;
use tracing::{debug, info};

/// Export markdown content to a PDF file
///
/// # Arguments
/// * `markdown_content` - The raw markdown text to export
/// * `output_path` - Path where the PDF should be saved
/// * `pdf_config` - PDF export configuration (fonts, fallbacks)
///
/// # Returns
/// * `Ok(())` if the PDF was successfully created
/// * `Err` if there was an error during conversion or file writing
///
/// # Example
/// ```no_run,ignore
/// use std::path::Path;
/// use markdown_viewer::internal::pdf_export::export_to_pdf;
/// use markdown_viewer::config::PdfExportConfig;
///
/// let markdown = "# Hello World\n\nThis is a test.";
/// let pdf_config = PdfExportConfig::default();
/// export_to_pdf(markdown, Path::new("output.pdf"), &pdf_config).unwrap();
/// ```
pub fn export_to_pdf(
    markdown_content: &str,
    output_path: &Path,
    pdf_config: &crate::config::PdfExportConfig,
) -> Result<()> {
    info!("Exporting markdown to PDF: {:?}", output_path);
    debug!("Markdown content length: {} bytes", markdown_content.len());

    // Convert path to string
    let output_path_str = output_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid output path: {:?}", output_path))?;

    // Configure fonts from user configuration
    let font_config = markdown2pdf::fonts::FontConfig {
        custom_paths: vec![],
        default_font: Some(pdf_config.default_font.clone()),
        code_font: Some(pdf_config.code_font.clone()),
        fallback_fonts: pdf_config.fallback_fonts.clone(),
        enable_subsetting: pdf_config.enable_subsetting,
    };

    // Use markdown2pdf to convert markdown to PDF with custom font configuration
    markdown2pdf::parse_into_file(
        markdown_content.to_string(),
        output_path_str,
        markdown2pdf::config::ConfigSource::Default,
        Some(&font_config),
    )
    .map_err(|e| anyhow::anyhow!("PDF generation failed: {:?}", e))?;

    info!("Successfully exported PDF to {:?}", output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_export_to_pdf_creates_file() {
        let markdown = "# Test Document\n\nThis is a test.\n\n## Section\n\n* Item 1\n* Item 2";
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join("test_export.pdf");
        let pdf_config = crate::config::PdfExportConfig::default();

        // Clean up if file exists
        let _ = fs::remove_file(&output_path);

        // Export to PDF
        let result = export_to_pdf(markdown, &output_path, &pdf_config);
        assert!(result.is_ok(), "PDF export should succeed");

        // Verify file was created
        assert!(output_path.exists(), "PDF file should exist");

        // Verify it's not empty
        let metadata = fs::metadata(&output_path).unwrap();
        assert!(metadata.len() > 0, "PDF file should not be empty");

        // Clean up
        fs::remove_file(&output_path).unwrap();
    }

    #[test]
    fn test_export_to_pdf_handles_empty_markdown() {
        let markdown = "";
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join("test_empty.pdf");
        let pdf_config = crate::config::PdfExportConfig::default();

        // Clean up if file exists
        let _ = fs::remove_file(&output_path);

        // Export to PDF
        let result = export_to_pdf(markdown, &output_path, &pdf_config);

        // Should still succeed (creates empty or minimal PDF)
        assert!(result.is_ok(), "PDF export should handle empty content");

        // Clean up if file was created
        let _ = fs::remove_file(&output_path);
    }

    #[test]
    fn test_export_to_pdf_validates_path() {
        let markdown = "# Test";
        let invalid_path = Path::new("/invalid/nonexistent/directory/test.pdf");
        let pdf_config = crate::config::PdfExportConfig::default();

        let result = export_to_pdf(markdown, invalid_path, &pdf_config);
        assert!(result.is_err(), "Should fail with invalid path");
    }
}

use crate::cli::import_enhanced;
use crate::cli::tool_detector::Tool;
use anyhow::Result;

pub async fn run_import_from_tool(
    tool_name: &str,
    is_global: bool,
    force: bool,
    output_path: &str,
) -> Result<()> {
    let tool = Tool::from_name(tool_name)?;
    import_enhanced::run_import_from_tool(tool, is_global, force, output_path).await
}

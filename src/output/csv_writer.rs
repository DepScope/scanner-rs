//! CSV output writer

use std::path::Path;
use csv::Writer;

use crate::models::DependencyRecord;

/// Write dependency records to a CSV file
pub fn write_csv(records: &[DependencyRecord], output_path: impl AsRef<Path>) -> std::io::Result<()> {
    let mut writer = Writer::from_path(output_path)?;
    
    // Write header
    writer.write_record(&[
        "package",
        "version",
        "source_file",
        "dep_type",
        "ecosystem",
        "file_type",
    ])?;
    
    // Write records
    for record in records {
        writer.write_record(&[
            &record.name,
            &record.version,
            record.source_file.to_string_lossy().as_ref(),
            &record.dep_type.to_string(),
            &record.ecosystem.to_string(),
            &record.file_type.to_string(),
        ])?;
    }
    
    writer.flush()?;
    Ok(())
}

use std::io::Result;

fn main() -> Result<()> {
    // Only handle protobuf files when the feature is enabled
    #[cfg(feature = "protobuf")]
    {
        use std::path::Path;
        use std::process::Command;

        // Check if protobuf files exist, if not fetch them
        let proto_files = [
            "proto/cel/expr/syntax.proto",
            "proto/cel/expr/checked.proto",
            "proto/cel/expr/value.proto",
            "proto/cel/expr/eval.proto",
            "proto/cel/expr/explain.proto",
        ];

        let missing_files: Vec<_> = proto_files
            .iter()
            .filter(|file| !Path::new(file).exists())
            .collect();

        if !missing_files.is_empty() {
            println!("cargo:warning=Protobuf files missing, fetching from CEL specification...");

            let output = Command::new("sh")
                .arg("update_protos.sh")
                .output()
                .expect("Failed to execute update_protos.sh. Make sure curl is available.");

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("Failed to fetch protobuf files: {}", stderr);
            }
        }

        // Generate protobuf code
        prost_build::Config::new()
            .disable_comments(&["."])
            .compile_protos(&proto_files, &["proto/"])?;
    }
    Ok(())
}

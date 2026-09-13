//! Build script for Windows application manifest and icon embedding.
//!
//! Configures and embeds an application manifest targeting Windows platforms,
//! enabling UTF-8 code page support, long path awareness, segment heap, and
//! specifying minimum supported OS versions, as well as compiling the primary
//! application icon from resources.

use embed_manifest::{
    manifest::{ActiveCodePage, ExecutionLevel, HeapType, Setting, SupportedOS},
    new_manifest,
};
use embed_resource::{CompilationResult, NONE};

const APPLICATION_RESOURCES_PATH: &str = "resources/resources.rc";
const APPLICATION_MAIN_ICON_PATH: &str = "resources/launcher.ico";

fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        compile_windows_resources();
        embed_windows_application_manifest();
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={APPLICATION_RESOURCES_PATH}");
    println!("cargo:rerun-if-changed={APPLICATION_MAIN_ICON_PATH}");
}

/// Compiles the Windows resource script containing application icons.
fn compile_windows_resources() {
    match embed_resource::compile(APPLICATION_RESOURCES_PATH, NONE) {
        CompilationResult::Ok | CompilationResult::NotWindows => {}
        CompilationResult::NotAttempted(reason) => {
            panic!(
                "Failed to attempt Windows resource compilation ({APPLICATION_RESOURCES_PATH}): {reason}"
            );
        }
        CompilationResult::Failed(reason) => {
            panic!(
                "Failed to compile Windows resource file ({APPLICATION_RESOURCES_PATH}): {reason}"
            );
        }
    }
}

/// Generates and embeds the application manifest into the Windows binary.
fn embed_windows_application_manifest() {
    let manifest = new_manifest(env!("CARGO_PKG_NAME"))
        .supported_os(SupportedOS::Windows10..)
        .active_code_page(ActiveCodePage::Utf8)
        .requested_execution_level(ExecutionLevel::AsInvoker)
        .long_path_aware(Setting::Enabled)
        .heap_type(HeapType::SegmentHeap);

    embed_manifest::embed_manifest(manifest).expect("unable to embed manifest file");
}

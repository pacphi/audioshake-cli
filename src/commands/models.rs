//! Models command implementation

use comfy_table::{presets::UTF8_FULL, Table};

use crate::cli::args::ModelsArgs;
use crate::config::types::AppConfig;
use crate::error::Result;

/// Model information for display
struct ModelInfo {
    name: &'static str,
    description: &'static str,
    category: &'static str,
    credits_per_min: f64,
    supports_hq: bool,
    variants: &'static [&'static str],
}

const MODELS: &[ModelInfo] = &[
    // Stem separation
    ModelInfo {
        name: "vocals",
        description: "Isolated vocal track",
        category: "stems",
        credits_per_min: 1.0,
        supports_hq: true,
        variants: &["hq"],
    },
    ModelInfo {
        name: "instrumental",
        description: "Instrumental (no vocals)",
        category: "stems",
        credits_per_min: 1.0,
        supports_hq: true,
        variants: &["hq"],
    },
    ModelInfo {
        name: "drums",
        description: "Isolated drum track",
        category: "stems",
        credits_per_min: 1.0,
        supports_hq: false,
        variants: &[],
    },
    ModelInfo {
        name: "bass",
        description: "Isolated bass track",
        category: "stems",
        credits_per_min: 1.0,
        supports_hq: false,
        variants: &[],
    },
    ModelInfo {
        name: "guitar",
        description: "Isolated guitar track",
        category: "stems",
        credits_per_min: 1.0,
        supports_hq: false,
        variants: &[],
    },
    ModelInfo {
        name: "piano",
        description: "Isolated piano track",
        category: "stems",
        credits_per_min: 1.0,
        supports_hq: false,
        variants: &[],
    },
    ModelInfo {
        name: "strings",
        description: "Isolated string instruments",
        category: "stems",
        credits_per_min: 1.0,
        supports_hq: false,
        variants: &[],
    },
    ModelInfo {
        name: "wind",
        description: "Isolated wind instruments",
        category: "stems",
        credits_per_min: 1.0,
        supports_hq: false,
        variants: &[],
    },
    ModelInfo {
        name: "other",
        description: "Residual audio",
        category: "stems",
        credits_per_min: 1.0,
        supports_hq: false,
        variants: &[],
    },
    // DMX
    ModelInfo {
        name: "dialogue",
        description: "Isolate speech from audio",
        category: "dmx",
        credits_per_min: 1.5,
        supports_hq: false,
        variants: &[],
    },
    ModelInfo {
        name: "effects",
        description: "Sound effects and ambience",
        category: "dmx",
        credits_per_min: 1.5,
        supports_hq: false,
        variants: &[],
    },
    ModelInfo {
        name: "music_detection",
        description: "Detect music sections",
        category: "dmx",
        credits_per_min: 0.5,
        supports_hq: false,
        variants: &[],
    },
    ModelInfo {
        name: "background",
        description: "Background audio extraction",
        category: "dmx",
        credits_per_min: 1.5,
        supports_hq: false,
        variants: &[],
    },
    // Multi-voice speaker separation (enterprise feature)
    ModelInfo {
        name: "multi_voice",
        description: "Separate individual speakers (enterprise)",
        category: "speaker",
        credits_per_min: 1.0,
        supports_hq: false,
        variants: &["two_speaker", "n_speaker"],
    },
    // Transcription
    ModelInfo {
        name: "transcription",
        description: "Convert speech to text",
        category: "transcription",
        credits_per_min: 1.0,
        supports_hq: false,
        variants: &[],
    },
    ModelInfo {
        name: "alignment",
        description: "Word-level timestamps",
        category: "transcription",
        credits_per_min: 1.0,
        supports_hq: false,
        variants: &[],
    },
];

/// Execute models command
///
/// # Errors
///
/// This function is infallible but returns Result for consistency.
#[allow(clippy::unnecessary_wraps)]
pub fn execute(args: &ModelsArgs, _config: &AppConfig) -> Result<()> {
    let models: Vec<&ModelInfo> = args.category.as_ref().map_or_else(
        || MODELS.iter().collect(),
        |category| {
            MODELS
                .iter()
                .filter(|m| m.category == category.as_str())
                .collect()
        },
    );

    if args.detailed {
        print_detailed_table(&models);
    } else {
        print_simple_list(&models);
    }

    Ok(())
}

fn print_simple_list(models: &[&ModelInfo]) {
    let mut current_category = "";

    for model in models {
        if model.category != current_category {
            if !current_category.is_empty() {
                println!();
            }
            let title = match model.category {
                "stems" => "Stem Separation Models",
                "dmx" => "Dialogue/Music/Effects Models",
                "speaker" => "Speaker Separation Models",
                "transcription" => "Transcription Models",
                _ => model.category,
            };
            println!("{}", console::style(title).bold());
            println!("{}", "-".repeat(title.len()));
            current_category = model.category;
        }

        let suffix = if model.supports_hq {
            " (HQ available)".to_string()
        } else if !model.variants.is_empty() {
            format!(" (variants: {})", model.variants.join(", "))
        } else {
            String::new()
        };

        println!(
            "  {:<16} {}{}",
            console::style(model.name).cyan(),
            model.description,
            console::style(&suffix).dim()
        );
    }
}

fn print_detailed_table(models: &[&ModelInfo]) {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        "Model",
        "Description",
        "Category",
        "Credits/Min",
        "Variants",
    ]);

    for model in models {
        let variants = if model.variants.is_empty() {
            "-".to_string()
        } else {
            model.variants.join(", ")
        };

        table.add_row(vec![
            model.name.to_string(),
            model.description.to_string(),
            model.category.to_string(),
            format!("{:.1}", model.credits_per_min),
            variants,
        ]);
    }

    println!("{table}");
}

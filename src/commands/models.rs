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
}

const MODELS: &[ModelInfo] = &[
    // Stem separation
    ModelInfo {
        name: "vocals",
        description: "Isolated vocal track",
        category: "stems",
        credits_per_min: 1.0,
        supports_hq: true,
    },
    ModelInfo {
        name: "instrumental",
        description: "Instrumental (no vocals)",
        category: "stems",
        credits_per_min: 1.0,
        supports_hq: true,
    },
    ModelInfo {
        name: "drums",
        description: "Isolated drum track",
        category: "stems",
        credits_per_min: 1.0,
        supports_hq: false,
    },
    ModelInfo {
        name: "bass",
        description: "Isolated bass track",
        category: "stems",
        credits_per_min: 1.0,
        supports_hq: false,
    },
    ModelInfo {
        name: "guitar",
        description: "Isolated guitar track",
        category: "stems",
        credits_per_min: 1.0,
        supports_hq: false,
    },
    ModelInfo {
        name: "piano",
        description: "Isolated piano track",
        category: "stems",
        credits_per_min: 1.0,
        supports_hq: false,
    },
    ModelInfo {
        name: "strings",
        description: "Isolated string instruments",
        category: "stems",
        credits_per_min: 1.0,
        supports_hq: false,
    },
    ModelInfo {
        name: "wind",
        description: "Isolated wind instruments",
        category: "stems",
        credits_per_min: 1.0,
        supports_hq: false,
    },
    ModelInfo {
        name: "other",
        description: "Residual audio",
        category: "stems",
        credits_per_min: 1.0,
        supports_hq: false,
    },
    // DMX
    ModelInfo {
        name: "dialogue",
        description: "Isolate speech from audio",
        category: "dmx",
        credits_per_min: 1.5,
        supports_hq: false,
    },
    ModelInfo {
        name: "effects",
        description: "Sound effects and ambience",
        category: "dmx",
        credits_per_min: 1.5,
        supports_hq: false,
    },
    ModelInfo {
        name: "music_detection",
        description: "Detect music sections",
        category: "dmx",
        credits_per_min: 0.5,
        supports_hq: false,
    },
    ModelInfo {
        name: "background",
        description: "Background audio extraction",
        category: "dmx",
        credits_per_min: 1.5,
        supports_hq: false,
    },
    // Transcription
    ModelInfo {
        name: "transcription",
        description: "Convert speech to text",
        category: "transcription",
        credits_per_min: 1.0,
        supports_hq: false,
    },
    ModelInfo {
        name: "alignment",
        description: "Word-level timestamps",
        category: "transcription",
        credits_per_min: 1.0,
        supports_hq: false,
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
                "transcription" => "Transcription Models",
                _ => model.category,
            };
            println!("{}", console::style(title).bold());
            println!("{}", "-".repeat(title.len()));
            current_category = model.category;
        }

        let hq = if model.supports_hq {
            " (HQ available)"
        } else {
            ""
        };
        println!(
            "  {:<16} {}{}",
            console::style(model.name).cyan(),
            model.description,
            console::style(hq).dim()
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
        "HQ",
    ]);

    for model in models {
        table.add_row(vec![
            model.name.to_string(),
            model.description.to_string(),
            model.category.to_string(),
            format!("{:.1}", model.credits_per_min),
            if model.supports_hq { "Yes" } else { "-" }.to_string(),
        ]);
    }

    println!("{table}");
}

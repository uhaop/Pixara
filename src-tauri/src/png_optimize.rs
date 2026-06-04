use std::path::Path;

use oxipng::{Deflater, InFile, OutFile, Options, StripChunks, ZopfliOptions, optimize};

use crate::cancel::is_batch_cancelled;
use crate::types::{GvError, Preset};

/// Lossless PNG recompression after encode + metadata strip. Slower; smaller files.
pub fn optimize_png_file(path: &Path, preset: Preset) -> Result<(), GvError> {
    if is_batch_cancelled() {
        return Err(GvError::Message("cancelled".into()));
    }

    let opts = options_for_preset(preset);
    let input = InFile::Path(path.to_path_buf());
    let output = OutFile::Path {
        path: None,
        preserve_attrs: true,
    };
    optimize(&input, &output, &opts).map_err(|e| {
        GvError::Message(format!("PNG optimize failed for {}: {e}", path.display()))
    })?;
    Ok(())
}

fn options_for_preset(preset: Preset) -> Options {
    let mut opts = Options::default();
    opts.strip = StripChunks::None;
    opts.fix_errors = false;

    match preset {
        Preset::Web => {
            opts.bit_depth_reduction = false;
            opts.color_type_reduction = false;
            opts.palette_reduction = false;
            opts.grayscale_reduction = false;
            opts.idat_recoding = true;
            opts.optimize_alpha = false;
            opts.deflater = Deflater::Libdeflater { compression: 6 };
        }
        Preset::High => {
            opts.bit_depth_reduction = true;
            opts.color_type_reduction = true;
            opts.palette_reduction = true;
            opts.grayscale_reduction = true;
            opts.idat_recoding = true;
            opts.optimize_alpha = true;
            opts.deflater = Deflater::Libdeflater { compression: 9 };
        }
        Preset::Smallest => {
            opts.bit_depth_reduction = true;
            opts.color_type_reduction = true;
            opts.palette_reduction = true;
            opts.grayscale_reduction = true;
            opts.idat_recoding = true;
            opts.optimize_alpha = true;
            opts.fast_evaluation = false;
            opts.deflater = Deflater::Zopfli(ZopfliOptions::default());
        }
    }

    opts
}

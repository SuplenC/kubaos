//! The three ONNX models, run through ONNX Runtime on the CPU: Depth Anything V2 Small's depth,
//! BiRefNet's single object and SAM's edge.
//!
//! ONNX Runtime is loaded when the models are, from `ORT_DYLIB_PATH` when it is set, else from
//! the path the Nix build bakes in (`KUBAOS_DEPTH_ORT` at build time, nixpkgs' `onnxruntime`),
//! else from the library path. Nothing links against it, so the workspace builds and tests
//! without it, and a missing runtime is an error, never a panic.
//!
//! # Owns
//!
//! - The model files' names in the models folder ([`Models`]), loading the runtime and the sessions
//!   ([`Sessions`]), and each model's input and output: [`Sessions::depth`], [`Sessions::object`],
//!   [`Sessions::edge`].
//!
//! # Not here
//!
//! - What their outputs mean for the cut-out: `crate::choose` and `crate::edge`.

use std::path::Path;
use std::path::PathBuf;

use image::RgbImage;
use image::imageops;
use image::imageops::FilterType;
use ort::environment::Environment;
use ort::session::Session;
use ort::session::builder::GraphOptimizationLevel;
use ort::value::Tensor;

use crate::grid::Grid;

/// Depth Anything V2 Small's file in the models folder.
pub const DEPTH_FILE: &str = "depth-anything-v2-small.onnx";
/// BiRefNet lite's.
pub const OBJECT_FILE: &str = "birefnet-lite.onnx";
/// SAM ViT-B's image encoder's.
pub const SAM_ENCODER_FILE: &str = "sam-vit-b-encoder.onnx";
/// SAM ViT-B's mask decoder's.
pub const SAM_DECODER_FILE: &str = "sam-vit-b-decoder.onnx";

/// The models' files, in one folder.
#[derive(Debug, Clone)]
pub struct Models {
    pub depth: PathBuf,
    pub object: PathBuf,
    pub sam_encoder: PathBuf,
    pub sam_decoder: PathBuf,
}

impl Models {
    /// The models in `dir`, by their names there.
    ///
    /// # Errors
    ///
    /// [`ModelsError`] naming the first file that isn't there.
    pub fn in_dir(dir: &Path) -> Result<Self, ModelsError> {
        let file = |name: &str| {
            let path = dir.join(name);
            if path.is_file() {
                Ok(path)
            } else {
                Err(ModelsError(path))
            }
        };
        Ok(Self {
            depth: file(DEPTH_FILE)?,
            object: file(OBJECT_FILE)?,
            sam_encoder: file(SAM_ENCODER_FILE)?,
            sam_decoder: file(SAM_DECODER_FILE)?,
        })
    }
}

/// A model's file isn't in the models folder.
#[derive(Debug, thiserror::Error)]
#[error("no model at {}", .0.display())]
pub struct ModelsError(PathBuf);

/// Loading ONNX Runtime or a model, or running one, failed.
#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error("loading ONNX Runtime from {}", .0.display())]
    Library(PathBuf, #[source] ort::LoadDynamicError),
    #[error("starting ONNX Runtime")]
    Runtime(#[source] ort::Error),
    #[error("loading {}", .0.display())]
    Load(PathBuf, #[source] ort::Error),
    #[error("running {0}")]
    Run(&'static str, #[source] ort::Error),
    #[error("{0} answered in a shape it doesn't have: {1:?}")]
    Shape(&'static str, Vec<i64>),
}

/// The runtime's library, as the module doc says.
fn runtime_path() -> PathBuf {
    if let Some(path) = std::env::var_os("ORT_DYLIB_PATH").filter(|path| !path.is_empty()) {
        return path.into();
    }
    option_env!("KUBAOS_DEPTH_ORT").unwrap_or("libonnxruntime.so").into()
}

/// The models, loaded into ONNX Runtime.
#[derive(Debug)]
pub struct Sessions {
    depth: Session,
    object: Session,
    sam_encoder: Session,
    sam_decoder: Session,
    /// Keeps the runtime's environment alive for the sessions.
    _environment: Environment,
}

/// The ImageNet mean and deviation Depth Anything and BiRefNet were trained on.
const MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const DEVIATION: [f32; 3] = [0.229, 0.224, 0.225];

/// Depth Anything's input width: its training size, a multiple of its 14-pixel patches.
const DEPTH_WIDTH: u32 = 518;
/// BiRefNet's input, square.
const OBJECT_SIZE: u32 = 1024;
/// SAM's encoder input, the picture scaled to fit its top left: the size its ONNX export takes.
const SAM_INPUT: (u32, u32) = (1024, 684);

impl Sessions {
    /// Loads ONNX Runtime and the models, each to run on `threads` of the CPU.
    ///
    /// # Errors
    ///
    /// [`RunError::Library`] or [`RunError::Runtime`] when ONNX Runtime can't be loaded or
    /// started, [`RunError::Load`] when a model can't be loaded.
    pub fn load(models: &Models, threads: usize) -> Result<Self, RunError> {
        let runtime = runtime_path();
        log::debug!("loading ONNX Runtime from {}", runtime.display());
        let environment = ort::init_from(&runtime)
            .map_err(|e| RunError::Library(runtime.clone(), e))?
            .with_name("kubaos-depth")
            .build()
            .map_err(RunError::Runtime)?;
        let load = |path: &Path| -> Result<Session, RunError> {
            let error = |e: ort::Error| RunError::Load(path.to_owned(), e);
            Session::builder(&environment)
                .map_err(error)?
                .with_optimization_level(GraphOptimizationLevel::Level3)
                .map_err(|e| error(e.into()))?
                .with_intra_threads(threads)
                .map_err(|e| error(e.into()))?
                .commit_from_file(path)
                .map_err(error)
        };
        Ok(Self {
            depth: load(&models.depth)?,
            object: load(&models.object)?,
            sam_encoder: load(&models.sam_encoder)?,
            sam_decoder: load(&models.sam_decoder)?,
            _environment: environment,
        })
    }

    /// Depth Anything's relative depth of `picture`, normalised to `[0, 1]` with 1 nearest, at
    /// its own size: 518 wide.
    ///
    /// # Errors
    ///
    /// [`RunError::Run`] when the model fails, [`RunError::Shape`] when it answers in a shape it
    /// has no business answering in.
    pub fn depth(&mut self, picture: &RgbImage) -> Result<Grid, RunError> {
        const NAME: &str = "Depth Anything";
        let height = (picture.height() as f32 * DEPTH_WIDTH as f32 / picture.width() as f32 / 14.0)
            .round()
            .max(1.0) as u32
            * 14;
        let input = normalised(&imageops::resize(picture, DEPTH_WIDTH, height, FilterType::CatmullRom));
        let tensor = Tensor::from_array(([1, 3, height as usize, DEPTH_WIDTH as usize], input))
            .map_err(|e| RunError::Run(NAME, e))?;
        let outputs = self
            .depth
            .run(ort::inputs![tensor])
            .map_err(|e| RunError::Run(NAME, e))?;
        let (shape, values) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| RunError::Run(NAME, e))?;
        let [_, rows, columns] = shape[..] else {
            return Err(RunError::Shape(NAME, shape.to_vec()));
        };
        let (low, high) = values
            .iter()
            .fold((f32::MAX, f32::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
        let span = (high - low).max(f32::EPSILON);
        Ok(Grid::new(
            columns as usize,
            rows as usize,
            values.iter().map(|v| (v - low) / span).collect(),
        ))
    }

    /// BiRefNet's chance each pixel of `picture` belongs to its single object, 1024 square: the
    /// picture squeezed into it.
    ///
    /// # Errors
    ///
    /// As [`Sessions::depth`].
    pub fn object(&mut self, picture: &RgbImage) -> Result<Grid, RunError> {
        const NAME: &str = "BiRefNet";
        let input = normalised(&imageops::resize(
            picture,
            OBJECT_SIZE,
            OBJECT_SIZE,
            FilterType::CatmullRom,
        ));
        let tensor = Tensor::from_array(([1, 3, OBJECT_SIZE as usize, OBJECT_SIZE as usize], input))
            .map_err(|e| RunError::Run(NAME, e))?;
        let outputs = self
            .object
            .run(ort::inputs![tensor])
            .map_err(|e| RunError::Run(NAME, e))?;
        let (shape, values) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| RunError::Run(NAME, e))?;
        let [_, _, rows, columns] = shape[..] else {
            return Err(RunError::Shape(NAME, shape.to_vec()));
        };
        Ok(Grid::new(
            columns as usize,
            rows as usize,
            values.iter().map(|&v| sigmoid(v)).collect(),
        ))
    }

    /// SAM's mask of the region `prompt` points at in `picture`: its chance per pixel at the
    /// picture's size in the encoder's input (1024 wide for a landscape), and SAM's own estimate of
    /// how well it fits.
    ///
    /// # Errors
    ///
    /// As [`Sessions::depth`].
    pub fn edge(&mut self, picture: &RgbImage, prompt: &Prompt) -> Result<(Grid, f32), RunError> {
        const ENCODER: &str = "SAM's encoder";
        const DECODER: &str = "SAM's decoder";
        let (input_width, input_height) = SAM_INPUT;
        let scale = (input_width as f32 / picture.width() as f32).min(input_height as f32 / picture.height() as f32);
        let width = ((picture.width() as f32 * scale).round() as u32).clamp(1, input_width);
        let height = ((picture.height() as f32 * scale).round() as u32).clamp(1, input_height);
        let scaled = imageops::resize(picture, width, height, FilterType::Triangle);
        let mut canvas = vec![0.0f32; (input_width * input_height * 3) as usize];
        for (x, y, pixel) in scaled.enumerate_pixels() {
            let at = ((y * input_width + x) * 3) as usize;
            for (channel, &value) in pixel.0.iter().enumerate() {
                canvas[at + channel] = f32::from(value);
            }
        }
        let image = Tensor::from_array(([input_height as usize, input_width as usize, 3], canvas))
            .map_err(|e| RunError::Run(ENCODER, e))?;
        let embedding = {
            let outputs = self
                .sam_encoder
                .run(ort::inputs![image])
                .map_err(|e| RunError::Run(ENCODER, e))?;
            let (shape, values) = outputs[0]
                .try_extract_tensor::<f32>()
                .map_err(|e| RunError::Run(ENCODER, e))?;
            Tensor::from_array((shape.to_vec(), values.to_vec())).map_err(|e| RunError::Run(ENCODER, e))?
        };

        // SAM's decoder takes a padding point labelled -1 after the prompt's own.
        let mut coords: Vec<f32> = Vec::new();
        let mut labels: Vec<f32> = Vec::new();
        for &(x, y, label) in &prompt.points {
            coords.extend([x * width as f32, y * height as f32]);
            labels.push(label);
        }
        coords.extend([0.0, 0.0]);
        labels.push(-1.0);
        let count = labels.len();
        let decoder = |e| RunError::Run(DECODER, e);
        let inputs = ort::inputs![
            "image_embeddings" => embedding,
            "point_coords" => Tensor::from_array(([1, count, 2], coords)).map_err(decoder)?,
            "point_labels" => Tensor::from_array(([1, count], labels)).map_err(decoder)?,
            "mask_input" => Tensor::from_array(([1, 1, 256, 256], vec![0.0f32; 256 * 256])).map_err(decoder)?,
            "has_mask_input" => Tensor::from_array(([1], vec![0.0f32])).map_err(decoder)?,
            "orig_im_size" => Tensor::from_array(([2], vec![input_height as f32, input_width as f32])).map_err(decoder)?,
        ];
        let outputs = self.sam_decoder.run(inputs).map_err(decoder)?;
        let (shape, masks) = outputs["masks"].try_extract_tensor::<f32>().map_err(decoder)?;
        let [_, count, rows, columns] = shape[..] else {
            return Err(RunError::Shape(DECODER, shape.to_vec()));
        };
        let (_, fits) = outputs["iou_predictions"]
            .try_extract_tensor::<f32>()
            .map_err(decoder)?;
        let best = (0..count as usize)
            .max_by(|&a, &b| fits[a].total_cmp(&fits[b]))
            .unwrap_or(0);
        let (rows, columns) = (rows as usize, columns as usize);
        let plane = &masks[best * rows * columns..(best + 1) * rows * columns];
        let mask = Grid::from_fn(width as usize, height as usize, |x, y| {
            sigmoid(plane[y.min(rows - 1) * columns + x.min(columns - 1)])
        });
        Ok((mask, fits.get(best).copied().unwrap_or(0.0)))
    }
}

/// What SAM is pointed at: points as shares of the picture, each labelled 1 inside the region, 0
/// outside it, or 2 and 3 for a box's top left and bottom right corners.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Prompt {
    pub points: Vec<(f32, f32, f32)>,
}

/// `picture` as the models take it: planes of red, green and blue, each normalised by ImageNet's
/// mean and deviation.
fn normalised(picture: &RgbImage) -> Vec<f32> {
    let plane = (picture.width() * picture.height()) as usize;
    let mut values = vec![0.0; plane * 3];
    for (at, pixel) in picture.pixels().enumerate() {
        for channel in 0..3 {
            values[channel * plane + at] = (f32::from(pixel.0[channel]) / 255.0 - MEAN[channel]) / DEVIATION[channel];
        }
    }
    values
}

fn sigmoid(v: f32) -> f32 {
    1.0 / (1.0 + (-v).exp())
}

use std::env;
use std::path::PathBuf;
use gio::prelude::FileExt;
use rten_imageproc::{Point, Rect};
use ocrs::{ImageSource, OcrEngine, OcrEngineParams, TextItem, TextWord};
use rten::Model;

#[derive(Debug)]
pub enum LineOrientation {
    Horizontal,
    Vertical,
}
#[derive(Debug)]
pub struct OcrWord {
    pub text: String,
    pub start_dist: i32, // starting position of the char relative to the left or top edge of its parent rect
    pub end_dist: i32, // ending position of the char relative to the left or top edge of its parent rect
}
#[derive(Debug)]
pub struct OcrLine {
    pub rect: Rect<i32>,
    pub words: Vec<OcrWord>,
    pub orientation: LineOrientation
}
// only works for horizontal lines
pub fn recognize_text_from_image(file_path: &PathBuf) -> anyhow::Result<Vec<OcrLine>>{

    // Use the `download-models.sh` script to download the models.
    let detection_model_path = env::var("DETECTION_MODEL_PATH")?;
    let rec_model_path = env::var("RECOGNITION_MODEL_PATH")?;

    let detection_model = Model::load_file(detection_model_path)?;
    let recognition_model = Model::load_file(rec_model_path)?;

    let engine = OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        ..Default::default()
    })?;
    let img = image::open(file_path).map(|image| image.into_rgb8())?;
    
    // Apply standard image pre-processing expected by this library (convert
    // to greyscale, map range to [-0.5, 0.5]).
    let img_source = ImageSource::from_bytes(img.as_raw(), img.dimensions())?;
    let ocr_input = engine.prepare_input(img_source)?;

    // Detect and recognize text. If you only need the text and don't need any
    // layout information, you can also use `engine.get_text(&ocr_input)`,
    // which returns all the text in an image as a single string.

    // Get oriented bounding boxes of text words in input image.
    let word_rects = engine.detect_words(&ocr_input)?;

    // Group words into lines. Each line is represented by a list of word
    // bounding boxes.
    let line_rects = engine.find_text_lines(&ocr_input, &word_rects);
    // Recognize the characters in each line.
    let line_texts = engine.recognize_text(&ocr_input, &line_rects)?;
    
    let mut lines: Vec<OcrLine> = Vec::new();
    for line in line_texts
        .iter()
        .flatten()
        .filter(|l| l.to_string().len() > 1) 
    {
        //left edge of a horizontal line
        let line_bounding_rect =line.bounding_rect();
        
        let words: Vec<TextWord> = line.words().collect();
        let left_edge = line_bounding_rect.left_edge();
        let mut ocr_words: Vec<OcrWord> = Vec::new();
        for word in words {
            ocr_words.push(OcrWord {
                text: word.to_string(),
                start_dist: left_edge.distance(word.bounding_rect().top_left()) as i32,
                end_dist: left_edge.distance(word.bounding_rect().top_right()) as i32,
            })
        }
        lines.push(OcrLine {
            rect: line_bounding_rect,
            words: ocr_words,
            orientation: LineOrientation::Horizontal,
        })
        
    }
    
    Ok(lines)
}
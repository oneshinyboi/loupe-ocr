# Loupe OCR - GNOME Image Viewer with Text Recognition

## Project Overview
Loupe is the GNOME image viewer that provides a clean, modern interface for viewing images. This project extends Loupe with OCR (Optical Character Recognition) functionality, allowing users to extract and highlight text directly from images.

## Current OCR Implementation Status
The OCR feature is currently in development with the following components:

### Implemented Components
- **OCR Engine** (`src/ocr_engine.rs`): Core text recognition functionality using RTEN models
- **Text Overlay Widget** (`src/widgets/text_overlay.rs`): UI component for displaying and interacting with detected text
- **"Recognise Text" Action**: Button in the image window for triggering OCR processing
- **OCR Models**: Pre-trained models for text detection and recognition stored in `ocr-models/`

### Key Files
- `src/ocr_engine.rs` - OCR processing logic
- `src/widgets/text_overlay.rs` - Text overlay widget implementation 
- `src/widgets/text_overlay.ui` - UI definition for text overlay
- `src/widgets/image_view.rs` - Main image view integration
- `src/widgets/image_window/actions.rs` - Action handling for OCR button
- `ocr-models/` - Directory containing RTEN model files for text detection/recognition

### Build and Test Commands
```bash
# Build the project
meson compile -C builddir

# Run the application
./builddir/src/loupe

# Download OCR models (if needed)
cd ocr-models && ./download-models.sh
```

### Development Notes
- The project uses Rust with GTK4 and libadwaita for the UI
- OCR functionality uses the `ocrs` crate with RTEN models
- Text detection and recognition are handled separately with dedicated model files
- The text overlay allows users to interact with detected text regions in the image

### Current Features
- Text detection and recognition from images
- Visual overlay showing detected text regions
- Integration with existing image viewer interface
- Support for copying detected text

### Architecture
The OCR system is integrated into the existing Loupe architecture:
1. User clicks "Recognise Text" button
2. OCR engine processes the current image
3. Text overlay widget displays detected text regions
4. Users can interact with highlighted text areas

This implementation maintains Loupe's clean, user-friendly design while adding powerful text recognition capabilities.
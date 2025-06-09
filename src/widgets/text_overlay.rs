// Copyright (c) 2025 Diamond
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! A widget that overlays selectable text on top of images
//!
//! This widget creates clickable, selectable text regions based on OCR data
//! that can be highlighted and copied by the user.

use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gio::File;
use gtk::CompositeTemplate;

use crate::deps::*;
use crate::ocr_engine::{recognize_text_from_image, OcrLine};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "text_overlay.ui")]
    pub struct LpTextOverlay {
        #[template_child]
        pub overlay: TemplateChild<gtk::Overlay>,
        #[template_child]
        pub revealer: TemplateChild<gtk::Revealer>,
        #[template_child]
        pub text_container: TemplateChild<gtk::Fixed>,
        #[template_child]
        pub selection_container: TemplateChild<gtk::Fixed>,
        
        /// OCR data for displaying text
        pub ocr_data: RefCell<Option<Vec<OcrLine>>>,
        /// Store text labels mapped by their data
        pub text_labels: RefCell<HashMap<String, gtk::Label>>,
        /// Store selection rectangles
        pub selection_boxes: RefCell<Vec<gtk::DrawingArea>>,
        /// Current selection state
        pub selection_active: RefCell<bool>,
        /// Selection start and end coordinates
        pub selection_start: RefCell<Option<(f64, f64)>>,
        pub selection_end: RefCell<Option<(f64, f64)>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LpTextOverlay {
        const NAME: &'static str = "LpTextOverlay";
        type Type = super::LpTextOverlay;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_css_name("lptextoverlay");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LpTextOverlay {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            
            // Set up revealer
            self.revealer.set_can_target(false);
            self.revealer
                .set_transition_type(gtk::RevealerTransitionType::Crossfade);
            self.revealer.set_reveal_child(false);
            
        }
    }

    impl WidgetImpl for LpTextOverlay {}
    impl BinImpl for LpTextOverlay {}
}

glib::wrapper! {
    pub struct LpTextOverlay(ObjectSubclass<imp::LpTextOverlay>)
        @extends gtk::Widget, adw::Bin;
}

impl LpTextOverlay {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
    pub async fn start_ocr(&self, file: Option<File>) {
        if let Some(file) = file{
            let path = file.path().unwrap();
            
            // show loading wheel
            let spinner = gtk::Spinner::new();
            spinner.start();
            self.imp().overlay.add_overlay(&spinner);
            
            // Run OCR in background thread to avoid blocking UI
            let (sender, receiver) = async_channel::bounded(1);
            let path_clone = path.clone();
            
            std::thread::spawn(move || {
                let result = recognize_text_from_image(&path_clone);
                let _ = sender.send_blocking(result);
            });
            
            match receiver.recv().await {
                Ok(Ok(overlay_data)) => {
                    log::info!("OCR completed with {:?} text regions", overlay_data);
                    self.show_with_data(overlay_data);
                }
                Ok(Err(e)) => {
                    log::error!("OCR failed: {}", e);
                }
                Err(e) => {
                    log::error!("OCR channel error: {}", e);
                }
            }
            
            self.imp().overlay.remove_overlay(&spinner);
        } else {
            log::error!("No current file for OCR");
        }
    }

    pub fn show_with_data(&self, ocr_data: Vec<OcrLine>) {
        let imp = self.imp();
        
        // Clear existing text labels and selection boxes
        self.clear_overlay();
        
        // Create text labels from OCR data
        //self.create_text_labels(&ocr_data);
        
        // Store OCR data after using it
        imp.ocr_data.replace(Some(ocr_data));
        
        // Show the overlay
        imp.revealer.set_reveal_child(true);
        
        // Make sure the overlay captures events
        imp.overlay.set_can_target(true);
        imp.text_container.set_can_target(true);
        
        self.grab_focus();
    }

    pub fn hide(&self) {
        let imp = self.imp();
        imp.revealer.set_reveal_child(false);
        
        // Disable event capturing when hidden
        imp.overlay.set_can_target(false);
        imp.text_container.set_can_target(false);
        
        self.clear_selection();
    }

    fn clear_overlay(&self) {
        let imp = self.imp();
        
        // Remove all existing text labels
        let mut child = imp.text_container.first_child();
        while let Some(widget) = child {
            let next = widget.next_sibling();
            imp.text_container.remove(&widget);
            child = next;
        }

        // Remove all existing selection boxes
        let mut child = imp.selection_container.first_child();
        while let Some(widget) = child {
            let next = widget.next_sibling();
            imp.selection_container.remove(&widget);
            child = next;
        }

        imp.text_labels.borrow_mut().clear();
        imp.selection_boxes.borrow_mut().clear();
    }

    /*fn create_text_labels(&self, ocr_data: &Vec<OcrLine>) {
        let imp = self.imp();
        
        for data in &ocr_data.data {
            if data.text.trim().is_empty() || data.conf < 30.0 || data.level != 5 {
                continue; // Skip low confidence or empty text
            }
            
            // Create a transparent label for the text
            let label = gtk::Label::new(Some(&data.text));
            label.set_selectable(true);
            label.set_can_focus(true);
            //label.set_opacity(0.0); // Make invisible but still selectable
            label.set_size_request(data.width, data.height);
            
            imp.text_container.put(&label, data.left as f64, data.top as f64);
            
            // Store the label with a unique key
            let key = format!("{}_{}_{}_{}", data.page_num, data.block_num, data.par_num, data.word_num);
            imp.text_labels.borrow_mut().insert(key, label.clone());

            // Add hover effect to show where text is
            let motion_controller = gtk::EventControllerMotion::new();
            motion_controller.connect_enter(glib::clone!(
                #[weak] label,
                move |_, _, _| {
                    label.add_css_class("text-hover");
                }
            ));
            motion_controller.connect_leave(glib::clone!(
                #[weak] label,
                move |_| {
                    label.remove_css_class("text-hover");
                }
            ));
            
            label.add_controller(motion_controller);
        }
    }*/

    fn start_selection(&self, x: f64, y: f64) {
        let imp = self.imp();
        imp.selection_start.replace(Some((x, y)));
        imp.selection_active.replace(true);
        self.clear_selection();
    }

    fn update_selection(&self, offset_x: f64, offset_y: f64) {
        let imp = self.imp();
        
        if let Some((start_x, start_y)) = *imp.selection_start.borrow() {
            let end_x = start_x + offset_x;
            let end_y = start_y + offset_y;
            
            imp.selection_end.replace(Some((end_x, end_y)));
            
            //self.update_selection_visual(start_x, start_y, end_x, end_y);
        }
    }

    /*fn update_selection_visual(&self, start_x: f64, start_y: f64, end_x: f64, end_y: f64) {
        let imp = self.imp();
        
        // Clear previous selection visuals
        self.clear_selection_visual();
        
        let min_x = start_x.min(end_x);
        let max_x = start_x.max(end_x);
        let min_y = start_y.min(end_y);
        let max_y = start_y.max(end_y);
        
        // Find all text labels that intersect with selection
        let ocr_data = imp.ocr_data.borrow();
        if let Some(ref data) = *ocr_data {
            for text_data in &data.data {
                if text_data.text.trim().is_empty() || text_data.conf < 30.0 {
                    continue;
                }
                
                let text_left = text_data.left as f64;
                let text_top = text_data.top as f64;
                let text_right = text_left + text_data.width as f64;
                let text_bottom = text_top + text_data.height as f64;
                
                // Check if this text intersects with selection rectangle
                if text_left < max_x && text_right > min_x && text_top < max_y && text_bottom > min_y {
                    self.create_selection_highlight(text_left, text_top, text_data.width as f64, text_data.height as f64);
                }
            }
        }
    }*/

    fn create_selection_highlight(&self, x: f64, y: f64, width: f64, height: f64) {
        let imp = self.imp();
        
        let highlight = gtk::DrawingArea::new();
        highlight.set_size_request(width as i32, height as i32);
        highlight.add_css_class("text-selection");
        
        highlight.set_draw_func(move |_, cr, _, _| {
            cr.set_source_rgba(0.2, 0.4, 0.8, 0.3); // Blue highlight with transparency
            cr.rectangle(0.0, 0.0, width, height);
            cr.fill().unwrap();
        });
        
        imp.selection_container.put(&highlight, x, y);
        imp.selection_boxes.borrow_mut().push(highlight);
    }

    fn clear_selection_visual(&self) {
        let imp = self.imp();
        
        for selection_box in imp.selection_boxes.borrow().iter() {
            imp.selection_container.remove(selection_box);
        }
        imp.selection_boxes.borrow_mut().clear();
    }

    fn end_selection(&self) {
        let imp = self.imp();
        imp.selection_active.replace(false);
    }

    fn clear_selection(&self) {
        let imp = self.imp();
        imp.selection_start.replace(None);
        imp.selection_end.replace(None);
        imp.selection_active.replace(false);
        self.clear_selection_visual();
    }

    /*fn copy_selected_text(&self) {
        let imp = self.imp();
        
        if let (Some((start_x, start_y)), Some((end_x, end_y))) = 
            (*imp.selection_start.borrow(), *imp.selection_end.borrow()) {
            
            let min_x = start_x.min(end_x);
            let max_x = start_x.max(end_x);
            let min_y = start_y.min(end_y);
            let max_y = start_y.max(end_y);
            
            let mut selected_text = Vec::new();
            
            // Collect all text within selection
            let ocr_data = imp.ocr_data.borrow();
            if let Some(ref data) = *ocr_data {
                for text_data in &data.data {
                    if text_data.text.trim().is_empty() || text_data.conf < 30.0 {
                        continue;
                    }
                    
                    let text_left = text_data.left as f64;
                    let text_top = text_data.top as f64;
                    let text_right = text_left + text_data.width as f64;
                    let text_bottom = text_top + text_data.height as f64;
                    
                    // Check if this text intersects with selection rectangle
                    if text_left < max_x && text_right > min_x && text_top < max_y && text_bottom > min_y {
                        selected_text.push((text_data.top, text_data.left, text_data.text.clone()));
                    }
                }
            }
            
            if !selected_text.is_empty() {
                // Sort by line (top coordinate) then by position (left coordinate)
                selected_text.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
                
                // Group by lines and join words
                let mut lines: Vec<Vec<String>> = Vec::new();
                let mut current_line = Vec::new();
                let mut current_top = selected_text[0].0;
                
                for (top, _, text) in selected_text {
                    if (top - current_top).abs() < 10 { // Same line threshold
                        current_line.push(text);
                    } else {
                        if !current_line.is_empty() {
                            lines.push(current_line);
                        }
                        current_line = vec![text];
                        current_top = top;
                    }
                }
                
                if !current_line.is_empty() {
                    lines.push(current_line);
                }
                
                // Join lines with newlines and words with spaces
                let final_text = lines.iter()
                    .map(|line| line.join(" "))
                    .collect::<Vec<_>>()
                    .join("\n");
                
                // Copy to clipboard
                let clipboard = self.clipboard();
                clipboard.set_text(&final_text);
                
                // Show a toast notification
                if let Some(_window) = self.root().and_then(|r| r.downcast::<gtk::Window>().ok()) {
                    // This would need to be adapted to work with the specific window type in Loupe
                    log::info!("Copied text to clipboard: {}", final_text);
                }
            }
        }
    }*/
}

impl Default for LpTextOverlay {
    fn default() -> Self {
        Self::new()
    }
}
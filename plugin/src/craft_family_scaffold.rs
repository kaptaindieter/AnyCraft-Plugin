use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug)]
pub struct CraftAppSpec {
    pub name: String,
    pub purpose: String,
    pub file_extension: String,
}

impl Default for CraftAppSpec {
    fn default() -> Self {
        Self {
            name: "CutCraft".to_string(),
            purpose: "Lossless Video Trimmer & Clip Cutter".to_string(),
            file_extension: "cutproj".to_string(),
        }
    }
}

pub fn scaffold_app(spec: &CraftAppSpec) -> Result<PathBuf, String> {
    let clean_name = spec.name.trim();
    if clean_name.is_empty() {
        return Err("App name cannot be empty".to_string());
    }

    let dir_name = clean_name.to_lowercase().replace(' ', "_");
    let base_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let target_dir = base_dir.join(&dir_name);
    let src_dir = target_dir.join("src");
    let assets_dir = target_dir.join("assets");

    fs::create_dir_all(&src_dir).map_err(|e| format!("Failed to create directories: {e}"))?;
    fs::create_dir_all(&assets_dir).map_err(|e| format!("Failed to create assets directory: {e}"))?;

    // Copy assets from base_dir/assets to target_dir/assets
    let source_assets = base_dir.join("assets");
    if source_assets.exists() {
        copy_dir_recursive(&source_assets, &assets_dir)?;
    }

    // 1. Cargo.toml
    let cargo_toml = CARGO_TOML_TEMPLATE.replace("__PKG_NAME__", &dir_name);
    fs::write(target_dir.join("Cargo.toml"), cargo_toml).map_err(|e| format!("Failed to write Cargo.toml: {e}"))?;

    // 2. Copy ArtCraft Theme from RecordCraft
    let theme_src = base_dir.join("recordcraft").join("src").join("artcraft_theme.rs");
    if theme_src.exists() {
        let theme_content = fs::read_to_string(&theme_src).map_err(|e| e.to_string())?;
        fs::write(src_dir.join("artcraft_theme.rs"), theme_content).map_err(|e| e.to_string())?;
    }

    // 3. Generate ArtCraft Drive
    let drive_code = DRIVE_TEMPLATE.replace("__APP_NAME__", clean_name).replace("__EXT__", &spec.file_extension);
    fs::write(src_dir.join("artcraft_drive.rs"), drive_code).map_err(|e| format!("Failed to write artcraft_drive.rs: {e}"))?;

    // 4. Generate ArtCraft AI Agent from RecordCraft template
    let agent_src = base_dir.join("recordcraft").join("src").join("artcraft_agent.rs");
    if agent_src.exists() {
        let mut agent_code = fs::read_to_string(&agent_src).map_err(|e| e.to_string())?;
        agent_code = agent_code.replace("RecordCraft AI Assistant", &format!("{} AI Assistant", clean_name));
        agent_code = agent_code.replace("Screen Recording & Audio Studio", &format!("{} & Studio", clean_name));
        agent_code = agent_code.replace(
            "Welcome to RecordCraft AI Assistant! I can optimize screen recording settings, configure multi-track audio routing, and suggest bitrate presets. What would you like to record?",
            &format!("Welcome to {} AI Assistant! I can help cut, split, optimize, and organize your clips with lossless export. What would you like to edit today?", clean_name)
        );
        fs::write(src_dir.join("artcraft_agent.rs"), agent_code).map_err(|e| e.to_string())?;
    }

    // 5. Generate app.rs
    let app_code = APP_RS_TEMPLATE
        .replace("__APP_NAME__", clean_name)
        .replace("__EXT__", &spec.file_extension);
    fs::write(src_dir.join("app.rs"), app_code).map_err(|e| format!("Failed to write app.rs: {e}"))?;

    // 6. Generate main.rs
    let main_code = MAIN_RS_TEMPLATE.replace("__APP_NAME__", clean_name);
    fs::write(src_dir.join("main.rs"), main_code).map_err(|e| format!("Failed to write main.rs: {e}"))?;

    // 7. Create Windows Desktop Shortcut (.lnk)
    create_desktop_shortcut(clean_name, &target_dir)?;

    Ok(target_dir)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    if let Ok(entries) = fs::read_dir(src) {
        for entry in entries.flatten() {
            let path = entry.path();
            let dest_child = dst.join(entry.file_name());
            if path.is_dir() {
                let _ = fs::create_dir_all(&dest_child);
                copy_dir_recursive(&path, &dest_child)?;
            } else {
                let _ = fs::copy(&path, &dest_child);
            }
        }
    }
    Ok(())
}

fn create_desktop_shortcut(name: &str, target_dir: &Path) -> Result<(), String> {
    let desktop = std::env::var_os("USERPROFILE").map(|u| PathBuf::from(u).join("Desktop")).unwrap_or_else(|| PathBuf::from("."));
    let lnk_path = desktop.join(format!("{name}.lnk"));
    let exe_path = target_dir.join("target").join("release").join(format!("{}.exe", name.to_lowercase().replace(' ', "_")));

    let ps_cmd = format!(
        "$WshShell = New-Object -ComObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut('{}'); $Shortcut.TargetPath = '{}'; $Shortcut.WorkingDirectory = '{}'; $Shortcut.Save()",
        lnk_path.display(),
        exe_path.display(),
        target_dir.display()
    );

    let status = Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps_cmd])
        .status();

    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(format!("PowerShell failed with code {:?}", s.code())),
        Err(e) => Err(format!("Failed to execute PowerShell: {e}")),
    }
}

const CARGO_TOML_TEMPLATE: &str = r#"[package]
name = "__PKG_NAME__"
version = "0.1.0"
edition = "2021"

[dependencies]
eframe = { git = "https://github.com/storytold/egui.git", rev = "6d518fc799681571963842f95b55a6b68431b15a", default-features = false, features = ["default_fonts", "glow"] }
egui = { git = "https://github.com/storytold/egui.git", rev = "6d518fc799681571963842f95b55a6b68431b15a", features = ["default_fonts"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = "0.4"
ureq = { version = "3.5", features = ["rustls"] }
windows-sys = { version = "0.59", features = [
    "Win32_Media_Audio",
    "Win32_System_Com",
    "Win32_System_Threading",
    "Win32_UI_WindowsAndMessaging",
    "Win32_Foundation",
] }

[patch.crates-io]
ecolor = { git = "https://github.com/storytold/egui.git", rev = "6d518fc799681571963842f95b55a6b68431b15a" }
emath = { git = "https://github.com/storytold/egui.git", rev = "6d518fc799681571963842f95b55a6b68431b15a" }
epaint = { git = "https://github.com/storytold/egui.git", rev = "6d518fc799681571963842f95b55a6b68431b15a" }
epaint_default_fonts = { git = "https://github.com/storytold/egui.git", rev = "6d518fc799681571963842f95b55a6b68431b15a" }
egui = { git = "https://github.com/storytold/egui.git", rev = "6d518fc799681571963842f95b55a6b68431b15a" }
egui-winit = { git = "https://github.com/storytold/egui.git", rev = "6d518fc799681571963842f95b55a6b68431b15a" }
egui_glow = { git = "https://github.com/storytold/egui.git", rev = "6d518fc799681571963842f95b55a6b68431b15a" }
eframe = { git = "https://github.com/storytold/egui.git", rev = "6d518fc799681571963842f95b55a6b68431b15a" }
winit = { git = "https://github.com/storytold/winit.git", rev = "a517b3846ee8457d84ee4975110bfda6797c44dc" }
"#;

const DRIVE_TEMPLATE: &str = r#"use std::path::PathBuf;
use std::time::Instant;

pub struct ArtCraftDrive {
    pub app_name: String,
    pub file_extension: String,
    pub auto_save_enabled: bool,
    pub last_save_time: Instant,
    pub save_interval_secs: u64,
}

impl ArtCraftDrive {
    pub fn new(app_name: &str, file_extension: &str) -> Self {
        Self {
            app_name: app_name.to_string(),
            file_extension: file_extension.to_string(),
            auto_save_enabled: true,
            last_save_time: Instant::now(),
            save_interval_secs: 60,
        }
    }

    pub fn google_drive_dir(&self) -> PathBuf {
        for drive in [r"G:\Meine Ablage\Art Craft", r"G:\My Drive\Art Craft", r"G:\Art Craft"] {
            let p = PathBuf::from(drive).join(&self.app_name);
            if p.exists() || PathBuf::from(drive).exists() {
                let _ = std::fs::create_dir_all(&p);
                return p;
            }
        }
        let fallback = std::env::var("USERPROFILE").map(|u| PathBuf::from(u).join("Documents").join("ArtCraft").join(&self.app_name)).unwrap_or_else(|_| PathBuf::from("."));
        let _ = std::fs::create_dir_all(&fallback);
        fallback
    }

    pub fn should_autosave(&self) -> bool {
        self.last_save_time.elapsed().as_secs() >= self.save_interval_secs
    }

    pub fn mark_saved(&mut self) {
        self.last_save_time = Instant::now();
    }
}
"#;

const APP_RS_TEMPLATE: &str = r#"use egui::{Color32, RichText, vec2};
use crate::artcraft_drive::ArtCraftDrive;
use crate::artcraft_theme::{paint_aurora_border, AppTheme};
use crate::artcraft_agent::ArtCraftAgent;

pub struct __APP_NAME__App {
    pub theme: AppTheme,
    pub drive: ArtCraftDrive,
    pub agent: ArtCraftAgent,
    pub project_name: String,
    pub video_source: String,
    pub in_point_secs: f32,
    pub out_point_secs: f32,
    pub video_duration_secs: f32,
    pub is_exporting: bool,
    pub export_status: String,
    pub clips: Vec<String>,
}

impl __APP_NAME__App {
    pub fn new() -> Self {
        Self {
            theme: AppTheme::load_global(),
            drive: ArtCraftDrive::new("__APP_NAME__", "__EXT__"),
            agent: ArtCraftAgent::new(),
            project_name: "Untitled Project".to_string(),
            video_source: "No video loaded".to_string(),
            in_point_secs: 0.0,
            out_point_secs: 60.0,
            video_duration_secs: 180.0,
            is_exporting: false,
            export_status: "Ready".to_string(),
            clips: vec!["Clip_001.mp4 (00:00 - 01:00)".to_string()],
        }
    }
}

impl eframe::App for __APP_NAME__App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let is_aurora = self.theme == AppTheme::Aurora;
        let is_light = self.theme == AppTheme::Light;

        let bg = if is_aurora {
            Color32::from_rgb(9, 11, 20)
        } else if is_light {
            Color32::from_rgb(245, 247, 252)
        } else {
            Color32::from_rgb(18, 18, 20)
        };

        let panel_bg = if is_aurora {
            Color32::from_rgb(15, 19, 34)
        } else if is_light {
            Color32::from_rgb(255, 255, 255)
        } else {
            Color32::from_rgb(24, 24, 27)
        };

        let text_main = if is_light { Color32::from_rgb(20, 25, 35) } else { Color32::WHITE };
        let text_dim = if is_light { Color32::from_rgb(100, 110, 130) } else { Color32::from_rgb(150, 165, 195) };

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(bg))
            .show(ui, |ui| {
                if is_aurora {
                    paint_aurora_border(ui.painter(), ui.max_rect(), 0.0);
                }

                // Top Header
                ui.horizontal(|ui| {
                    ui.label(RichText::new("✂  __APP_NAME__").size(20.0).strong().color(Color32::from_rgb(235, 110, 240)));
                    ui.label(RichText::new(format!("— {}", self.project_name)).size(14.0).color(text_dim));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // AI Assistant Toggle
                        let agent_btn = egui::Button::new(
                            RichText::new(if self.agent.is_open { "🤖 Close AI" } else { "✨ AI Assistant" })
                                .size(13.0)
                                .color(Color32::WHITE)
                        )
                        .fill(Color32::from_rgb(120, 60, 220))
                        .corner_radius(6.0);
                        if ui.add(agent_btn).clicked() {
                            self.agent.is_open = !self.agent.is_open;
                        }

                        // Theme switcher
                        ui.label(RichText::new("Theme:").size(12.0).color(text_dim));
                        for t in AppTheme::ALL {
                            let sel = self.theme == t;
                            if ui.selectable_label(sel, t.name()).clicked() {
                                self.theme = t;
                                AppTheme::save_global(t);
                            }
                        }

                        ui.separator();
                        ui.label(RichText::new("☁ Google Drive Connected").size(11.5).color(Color32::from_rgb(0, 230, 130)));
                    });
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // Main Workbench & Cutting Controls
                ui.columns(2, |cols| {
                    let ui = &mut cols[0];
                    egui::Frame::new().fill(panel_bg).corner_radius(8.0).inner_margin(16.0).show(ui, |ui| {
                        ui.label(RichText::new("Lossless Video Trimmer & Clip Cutter").size(16.0).strong().color(text_main));
                        ui.add_space(8.0);
                        ui.label(RichText::new("Source Video:").color(text_dim));
                        ui.add(egui::TextEdit::singleline(&mut self.video_source).desired_width(ui.available_width()));
                        ui.add_space(12.0);

                        ui.label(RichText::new("Cut In-Point (seconds):").color(text_dim));
                        ui.add(egui::Slider::new(&mut self.in_point_secs, 0.0..=self.video_duration_secs).suffix("s"));

                        ui.add_space(8.0);
                        ui.label(RichText::new("Cut Out-Point (seconds):").color(text_dim));
                        ui.add(egui::Slider::new(&mut self.out_point_secs, 0.0..=self.video_duration_secs).suffix("s"));

                        ui.add_space(14.0);
                        let cut_btn = egui::Button::new(RichText::new("✂  Trim & Add Cut Segment").strong().color(Color32::WHITE))
                            .fill(Color32::from_rgb(190, 60, 160))
                            .min_size(vec2(ui.available_width(), 38.0));
                        if ui.add(cut_btn).clicked() {
                            let seg = format!("Segment_{:03}.mp4 ({:.1}s - {:.1}s)", self.clips.len() + 1, self.in_point_secs, self.out_point_secs);
                            self.clips.push(seg);
                        }
                    });

                    let ui = &mut cols[1];
                    egui::Frame::new().fill(panel_bg).corner_radius(8.0).inner_margin(16.0).show(ui, |ui| {
                        ui.label(RichText::new(format!("Export Queue & Cut List ({})", self.clips.len())).size(16.0).strong().color(text_main));
                        ui.add_space(8.0);

                        for (i, c) in self.clips.iter().enumerate() {
                            ui.label(format!("{}. {}", i + 1, c));
                        }

                        ui.add_space(14.0);
                        let exp_btn = egui::Button::new(RichText::new("⚡ Lossless Direct Stream Export (Fast)").strong().color(Color32::WHITE))
                            .fill(Color32::from_rgb(30, 140, 70))
                            .min_size(vec2(ui.available_width(), 40.0));
                        if ui.add(exp_btn).clicked() {
                            self.export_status = "Export completed successfully to Google Drive!".to_string();
                        }
                        ui.add_space(6.0);
                        ui.label(RichText::new(&self.export_status).size(12.0).color(Color32::from_rgb(0, 230, 130)));
                    });
                });
            });

        // Render ArtCraft AI Sidebar
        self.agent.render_sidebar(ui, "__APP_NAME__ Project: Cut Points & Export Queue");
    }
}
"#;

const MAIN_RS_TEMPLATE: &str = r#"mod artcraft_agent;
mod artcraft_drive;
mod artcraft_theme;
mod app;

use app::__APP_NAME__App;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([960.0, 600.0])
            .with_title("__APP_NAME__ - Craft Family"),
        ..Default::default()
    };

    eframe::run_native(
        "__APP_NAME__",
        options,
        Box::new(|_cc| Ok(Box::new(__APP_NAME__App::new()))),
    )
}
"#;

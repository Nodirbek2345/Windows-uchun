use std::sync::Arc;
use eframe::egui;
use std::sync::atomic::Ordering;
use crate::state::AppState;
use crate::detector::DetectionType;
use egui_plot::{Line, Plot, PlotPoints};
use tray_icon::{TrayIconEvent, menu::MenuEvent};

pub struct AIFilterApp {
    state: Arc<AppState>,
    selected_tab: String,
    // Filterlar sahifasi uchun
    filter_phone: bool,
    filter_email: bool,
    filter_card: bool,
    filter_passport: bool,
    filter_pinfl: bool,
    filter_name: bool,
    filter_inn: bool,
    filter_date: bool,
    // Sozlamalar
    proxy_port: String,
    mask_style: String,
    // Test maydoni
    test_input: String,
    test_output: String,
    
    // Tray integration
    tray_show_id: tray_icon::menu::MenuId,
    tray_quit_id: tray_icon::menu::MenuId,
    window_visible: bool,
    should_quit: bool,
}

impl AIFilterApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        state: Arc<AppState>,
        tray_show_id: tray_icon::menu::MenuId,
        tray_quit_id: tray_icon::menu::MenuId,
        start_minimized: bool,
    ) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = egui::Color32::from_rgb(10, 12, 18);
        visuals.window_fill = egui::Color32::from_rgb(10, 12, 18);
        visuals.override_text_color = Some(egui::Color32::from_rgb(200, 210, 230));
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(18, 22, 35);
        cc.egui_ctx.set_visuals(visuals);

        let window_visible = !start_minimized;

        if start_minimized {
            cc.egui_ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }

        let ctx_clone = cc.egui_ctx.clone();
        let show_id = tray_show_id.clone();
        let quit_id = tray_quit_id.clone();
        let state_clone = state.clone();

        std::thread::spawn(move || {
            let menu_channel = tray_icon::menu::MenuEvent::receiver();
            let tray_channel = tray_icon::TrayIconEvent::receiver();
            loop {
                if let Ok(event) = menu_channel.try_recv() {
                    if event.id == show_id {
                        ctx_clone.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                        ctx_clone.send_viewport_cmd(egui::ViewportCommand::Focus);
                        ctx_clone.request_repaint();
                    } else if event.id == quit_id {
                        state_clone.sync_system_proxy(false);
                        std::process::exit(0);
                    }
                }
                
                if let Ok(event) = tray_channel.try_recv() {
                    if let tray_icon::TrayIconEvent::DoubleClick { .. } = event {
                        ctx_clone.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                        ctx_clone.send_viewport_cmd(egui::ViewportCommand::Focus);
                        ctx_clone.request_repaint();
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        });

        Self {
            state,
            selected_tab: "Bosh sahifa".to_string(),
            filter_phone: true,
            filter_email: true,
            filter_card: true,
            filter_passport: true,
            filter_pinfl: true,
            filter_name: true,
            filter_inn: true,
            filter_date: false,
            proxy_port: "8080".to_string(),
            mask_style: "label".to_string(),
            test_input: String::new(),
            test_output: String::new(),
            
            tray_show_id,
            tray_quit_id,
            window_visible,
            should_quit: false,
        }
    }
}

// === CONSTANTS ===
const BG_DARK: egui::Color32 = egui::Color32::from_rgb(11, 13, 23);
const BG_CARD: egui::Color32 = egui::Color32::from_rgb(18, 22, 35);
const BG_SIDEBAR: egui::Color32 = egui::Color32::from_rgb(14, 16, 26);
const NEON: egui::Color32 = egui::Color32::from_rgb(240, 245, 255);
const DIM: egui::Color32 = egui::Color32::from_rgb(120, 130, 150);
const PURPLE: egui::Color32 = egui::Color32::from_rgb(125, 45, 250);
const BLUE: egui::Color32 = egui::Color32::from_rgb(0, 120, 255);
const CYAN: egui::Color32 = egui::Color32::from_rgb(0, 210, 210);
const ORANGE: egui::Color32 = egui::Color32::from_rgb(255, 140, 0);
const GREEN: egui::Color32 = egui::Color32::from_rgb(0, 230, 100);
const RED: egui::Color32 = egui::Color32::from_rgb(255, 60, 90);
const BORDER: egui::Color32 = egui::Color32::from_rgb(30, 35, 50);

fn card_frame(border: egui::Color32) -> egui::Frame {
    egui::Frame::none()
        .fill(BG_CARD)
        .rounding(10.0)
        .inner_margin(10.0)
        .stroke(egui::Stroke::new(1.0, border))
}

impl eframe::App for AIFilterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        
        // Tray events are now handled in the background thread.

        // Intercept standard window Close ('X' button)
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.should_quit {
                // User explicitly clicked "Quit" from tray, allow it to close
            } else {
                // Cancel close and hide instead
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                self.window_visible = false;
            }
        }

        // If should quit, close the app
        if self.should_quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        
        // === TOP HEADER ===
        egui::TopBottomPanel::top("header")
            .frame(egui::Frame::none().fill(BG_DARK).inner_margin(egui::Margin::symmetric(12.0, 8.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(24.0, 24.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 6.0, egui::Color32::from_rgb(30, 15, 60));
                    ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "🛡", egui::FontId::proportional(14.0), PURPLE);
                    
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("PrivacyProxy").size(16.0).strong().color(egui::Color32::WHITE));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("⋮").size(18.0).color(DIM));
                        ui.add_space(6.0);
                        
                        let active = self.state.is_active();
                        let dot = if active { GREEN } else { RED };
                        let bg = if active { egui::Color32::from_rgb(15, 45, 25) } else { egui::Color32::from_rgb(45, 15, 25) };
                        let text = if active { "Yoqilgan" } else { "O'chirilgan" };
                        
                        let frame = egui::Frame::none().fill(bg).rounding(12.0).inner_margin(egui::Margin::symmetric(10.0, 4.0));
                        let resp = frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let (r, _) = ui.allocate_exact_size(egui::Vec2::new(8.0, 8.0), egui::Sense::hover());
                                ui.painter().circle_filled(r.center(), 4.0, dot);
                                ui.add_space(5.0);
                                ui.label(egui::RichText::new(text).color(dot).size(11.0));
                            });
                        }).response;
                        
                        let interact = ui.interact(resp.rect, ui.id().with("toggle_top"), egui::Sense::click());
                        if interact.clicked() { self.state.set_active(!active); }
                        if interact.hovered() { ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand); }
                    });
                });
            });

        // === ICON-SIDEBAR ===
        egui::SidePanel::left("sidebar")
            .exact_width(65.0)
            .frame(
                egui::Frame::none()
                    .fill(BG_SIDEBAR)
                    .inner_margin(egui::Margin::symmetric(0.0, 10.0))
            )
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(4.0);
                    
                    let tabs = [
                        ("Bosh sahifa", "🏠", PURPLE),
                        ("AI", "🤖", DIM),
                        ("Statistika", "📊", DIM),
                        ("Filterlar", "🔒", DIM),
                        ("Sozlamalar", "⚙", DIM),
                        ("Logs", "📋", DIM),
                        ("Yordam", "❓", DIM),
                        ("Haqida", "ℹ", DIM),
                    ];
                    
                    for (name, icon, _accent) in tabs {
                        let is_selected = self.selected_tab == name;
                        
                        let (rect, resp) = ui.allocate_exact_size(egui::Vec2::new(65.0, 48.0), egui::Sense::click());
                        let color = if is_selected { PURPLE } else { DIM };
                        let text_color = if is_selected { NEON } else { DIM };
                        
                        if is_selected {
                            let line_rect = egui::Rect::from_min_size(rect.min, egui::Vec2::new(2.5, rect.height()));
                            ui.painter().rect_filled(line_rect, 0.0, PURPLE);
                            let bg_rect = egui::Rect::from_min_size(rect.min + egui::Vec2::new(2.5, 0.0), egui::Vec2::new(rect.width() - 2.5, rect.height()));
                            ui.painter().rect_filled(bg_rect, 0.0, egui::Color32::from_rgb(25, 20, 45));
                        }
                        
                        if resp.hovered() {
                            ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                            if !is_selected {
                                ui.painter().rect_filled(rect, 0.0, egui::Color32::from_rgb(20, 24, 38));
                            }
                        }
                        
                        ui.painter().text(rect.center() - egui::Vec2::new(0.0, 8.0), egui::Align2::CENTER_CENTER, icon, egui::FontId::proportional(18.0), color);
                        ui.painter().text(rect.center() + egui::Vec2::new(0.0, 13.0), egui::Align2::CENTER_CENTER, name, egui::FontId::proportional(9.0), text_color);
                        
                        if resp.clicked() {
                            if name == "AI" { self.selected_tab = "AI Xizmatlari".to_string(); }
                            else { self.selected_tab = name.to_string(); }
                        }
                        ui.add_space(2.0);
                    }
                });
                
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    let active = self.state.is_active();
                    let dot = if active { GREEN } else { RED };
                    ui.horizontal(|ui| {
                        let (dot_rect, _) = ui.allocate_exact_size(egui::Vec2::new(6.0, 6.0), egui::Sense::hover());
                        ui.painter().circle_filled(dot_rect.center(), 3.0, dot);
                        ui.add_space(3.0);
                        ui.label(egui::RichText::new("Online").size(9.0).color(NEON));
                    });
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("v1.2.0").size(9.0).color(DIM));
                });
            });

        // === CENTRAL PANEL ===
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BG_DARK).inner_margin(10.0))
            .show(ctx, |ui| {
                if self.state.update_available.load(std::sync::atomic::Ordering::Relaxed) {
                    card_frame(egui::Color32::from_rgb(200, 100, 0)).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("🚀 YANGI VERSIYA: ").strong().color(egui::Color32::from_rgb(200, 100, 0)));
                            ui.label(egui::RichText::new("Dastur muvaffaqiyatli yangilandi! \nO'zgarishlar kuchga kirishi uchun dasturni yoping va qayta ishga tushiring.").color(egui::Color32::WHITE).size(11.0));
                        });
                    });
                    ui.add_space(8.0);
                }

                match self.selected_tab.as_str() {
                    "Bosh sahifa" => self.page_dashboard(ui),
                    "AI Xizmatlari" => self.page_ai_services(ui),
                    "Statistika" => self.page_statistika(ui),
                    "Filterlar" => self.page_filterlar(ui),
                    "Sozlamalar" => self.page_sozlamalar(ui),
                    "Logs" => self.page_logs(ui),
                    "Yordam" => self.page_yordam(ui),
                    "Haqida" => self.page_haqida(ui),
                    _ => {}
                }
            });

        ctx.request_repaint();
    }
}

// === PAGE IMPLEMENTATIONS ===
impl AIFilterApp {

    // ==================== AI XIZMATLARI ====================
    fn page_ai_services(&self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(egui::Vec2::new(24.0, 24.0), egui::Sense::hover());
            ui.painter().rect_filled(r, 6.0, egui::Color32::from_rgb(20, 15, 40));
            ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, "AI", egui::FontId::proportional(14.0), NEON);
            ui.add_space(6.0);
            ui.label(egui::RichText::new("AI Xizmatlari").size(18.0).strong().color(NEON));
        });
        
        ui.add_space(3.0);
        ui.label(egui::RichText::new("Qaysi AI platformalarga filtr qo'llanilmoqda").size(11.0).color(DIM));
        ui.add_space(10.0);

        let active = self.state.is_active();
        card_frame(if active { GREEN } else { RED }).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let dot_color = if active { GREEN } else { RED };
                let (r, _) = ui.allocate_exact_size(egui::Vec2::new(10.0, 10.0), egui::Sense::hover());
                ui.painter().circle_filled(r.center(), 5.0, dot_color);
                ui.add_space(5.0);
                ui.label(egui::RichText::new(if active { "Filter FAOL" } else { "Filter O'CHIRILGAN" }).size(12.0).strong().color(NEON));
            });
        });

        ui.add_space(10.0);

        let services = [
            ("ChatGPT", "chat.openai.com", "O", "OpenAI", CYAN, egui::Color32::from_rgb(0, 200, 100)),
            ("Claude", "claude.ai", "A", "Anthropic", PURPLE, egui::Color32::from_rgb(180, 100, 255)),
            ("Gemini", "gemini.google.com", "G", "Google", BLUE, egui::Color32::from_rgb(80, 150, 255)),
            ("Copilot", "copilot.microsoft.com", "M", "Microsoft", ORANGE, egui::Color32::from_rgb(255, 120, 50)),
            ("Grok", "grok.x.ai", "X", "xAI", DIM, egui::Color32::from_rgb(200, 200, 200)),
            ("Perplexity", "perplexity.ai", "P", "Perplexity AI", GREEN, egui::Color32::from_rgb(50, 200, 200)),
        ];

        let target_sites = self.state.config.read().target_sites.clone();

        for (name, domain, initial, company, border_col, dot_col) in services {
            let is_monitored = target_sites.iter().any(|s| domain.contains(s) || s.contains(domain));
            let border = if is_monitored { border_col } else { egui::Color32::from_rgb(30, 35, 50) };
            card_frame(border).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(egui::Vec2::new(28.0, 28.0), egui::Sense::hover());
                    ui.painter().circle_stroke(r.center(), 14.0, egui::Stroke::new(1.5, dot_col));
                    ui.painter().circle_filled(r.center(), 11.0, egui::Color32::from_rgb(25, 25, 35));
                    ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, initial, egui::FontId::proportional(14.0), border_col);
                    
                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(name).size(13.0).strong().color(NEON));
                        ui.label(egui::RichText::new(company).size(10.0).color(DIM));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if is_monitored {
                            egui::Frame::none()
                                .fill(egui::Color32::from_rgb(20, 50, 30))
                                .rounding(8.0)
                                .inner_margin(egui::Margin::symmetric(6.0, 3.0))
                                .stroke(egui::Stroke::new(1.0, GREEN))
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new("● FAOL").size(9.0).color(GREEN));
                                });
                        } else {
                            egui::Frame::none()
                                .fill(egui::Color32::from_rgb(30, 30, 35))
                                .rounding(8.0)
                                .inner_margin(egui::Margin::symmetric(6.0, 3.0))
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new("○ Nofaol").size(9.0).color(DIM));
                                });
                        }
                    });
                });
            });
            ui.add_space(4.0);
        }

        ui.add_space(10.0);

        card_frame(egui::Color32::from_rgb(30, 40, 70)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(egui::Vec2::new(18.0, 18.0), egui::Sense::hover());
                ui.painter().circle_stroke(r.center(), 9.0, egui::Stroke::new(1.5, CYAN));
                ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, "i", egui::FontId::proportional(11.0), CYAN);
                
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Qanday ishlaydi?").size(12.0).strong().color(NEON));
                    ui.label(egui::RichText::new(
                        "AI xizmatlariga kirsangiz, dastur trafikni tekshiradi va shaxsiy ma'lumotlarni maskalaydi."
                    ).size(10.0).color(egui::Color32::from_rgb(180, 190, 220)));
                });
            });
        });
        }); // end ScrollArea
    }

    // ==================== BOSH SAHIFA ====================
    fn page_dashboard(&self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(3.0);

        // 1. Stat cards top row
        ui.columns(4, |cols| {
            let items = [
                ("Matn/Pil", self.state.stats_text_filtered.load(Ordering::Relaxed), PURPLE, "💬"),
                ("Telefon", self.state.stats_phone_filtered.load(Ordering::Relaxed), BLUE, "📞"),
                ("Email", self.state.stats_email_filtered.load(Ordering::Relaxed), CYAN, "✉"),
                ("Rasm/Yuz", self.state.stats_image_filtered.load(Ordering::Relaxed), ORANGE, "👤"),
            ];
            for (i, (title, val, color, icon)) in items.iter().enumerate() {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(15, 18, 28))
                    .rounding(10.0)
                    .inner_margin(egui::Margin::same(8.0))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(20, 25, 40)))
                    .show(&mut cols[i], |ui| {
                        ui.set_width(ui.available_width());
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new(*icon).size(18.0).color(*color));
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new(val.to_string()).size(20.0).strong().color(egui::Color32::WHITE));
                            ui.add_space(2.0);
                            ui.label(egui::RichText::new(*title).size(9.0).color(DIM));
                            ui.add_space(6.0);
                            let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(ui.available_width(), 2.0), egui::Sense::hover());
                            ui.painter().rect_filled(rect, 1.0, *color);
                        });
                    });
            }
        });

        ui.add_space(10.0);

        // 2. Chart
        egui::Frame::none().fill(BG_CARD).rounding(10.0).inner_margin(10.0).stroke(egui::Stroke::new(1.0, BORDER)).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Faoliyat (bugun)").size(13.0).strong().color(egui::Color32::WHITE));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::Frame::none().fill(egui::Color32::from_rgb(20, 22, 35)).rounding(5.0).inner_margin(egui::Margin::symmetric(6.0, 3.0)).show(ui, |ui| ui.label(egui::RichText::new("30k").size(9.0).color(DIM)));
                    ui.add_space(3.0);
                    egui::Frame::none().fill(egui::Color32::from_rgb(20, 22, 35)).rounding(5.0).inner_margin(egui::Margin::symmetric(6.0, 3.0)).show(ui, |ui| ui.label(egui::RichText::new("7k").size(9.0).color(DIM)));
                    ui.add_space(3.0);
                    egui::Frame::none().fill(PURPLE).rounding(5.0).inner_margin(egui::Margin::symmetric(6.0, 3.0)).show(ui, |ui| ui.label(egui::RichText::new("24s").color(egui::Color32::WHITE).size(9.0)));
                });
            });
            ui.add_space(6.0);

            let line = Line::new(PlotPoints::from_iter((0..280).map(|i| {
                let x = i as f64 * 0.1;
                [x, (x*0.5).sin() * 2.0 + (x * 1.2).cos() + x*0.1 + 3.0]
            })))
            .color(PURPLE)
            .width(2.0)
            .fill(0.0);
            
            Plot::new("activity_plot")
                .view_aspect(3.0)
                .show_axes([true, true])
                .show_grid(false)
                .show_x(true)
                .show_y(true)
                .allow_drag(false)
                .allow_zoom(false)
                .show(ui, |p| p.line(line));
        });

        ui.add_space(10.0);

        // 3. Filter holati
        egui::Frame::none().fill(BG_CARD).rounding(10.0).inner_margin(10.0).stroke(egui::Stroke::new(1.0, BORDER)).show(ui, |ui| {
            ui.label(egui::RichText::new("Filter holati").size(13.0).strong().color(egui::Color32::WHITE));
            ui.add_space(8.0);
            
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                let circle_radius = 40.0;
                let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(circle_radius * 2.0, circle_radius * 2.0), egui::Sense::hover());
                
                let total = self.state.stats_text_filtered.load(Ordering::Relaxed)
                    + self.state.stats_phone_filtered.load(Ordering::Relaxed)
                    + self.state.stats_email_filtered.load(Ordering::Relaxed)
                    + self.state.stats_image_filtered.load(Ordering::Relaxed);
                
                let background_color = egui::Color32::from_rgb(25, 20, 50);
                ui.painter().circle_stroke(rect.center(), circle_radius, egui::Stroke::new(12.0, background_color));
                
                if total > 0 {
                    ui.painter().circle_stroke(rect.center(), circle_radius, egui::Stroke::new(6.0, PURPLE));
                }
                
                let percent_str = if total > 0 { "100%" } else { "0%" };
                ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, percent_str, egui::FontId::proportional(18.0), egui::Color32::WHITE);
                
                ui.add_space(20.0);
                
                ui.vertical(|ui| {
                    ui.add_space(6.0);
                    let stats = [
                        ("Matn/Pil", PURPLE, self.state.stats_text_filtered.load(Ordering::Relaxed)),
                        ("Telefon", BLUE, self.state.stats_phone_filtered.load(Ordering::Relaxed)),
                        ("Email", CYAN, self.state.stats_email_filtered.load(Ordering::Relaxed)),
                        ("Rasm/Yuz", ORANGE, self.state.stats_image_filtered.load(Ordering::Relaxed)),
                    ];
                    
                    for (name, color, val) in stats {
                        ui.horizontal(|ui| {
                            let (r, _) = ui.allocate_exact_size(egui::Vec2::new(8.0, 8.0), egui::Sense::hover());
                            ui.painter().circle_filled(r.center(), 4.0, color);
                            ui.add_space(5.0);
                            ui.label(egui::RichText::new(name).size(11.0).color(DIM));
                            ui.add_space(8.0);
                            ui.label(egui::RichText::new(val.to_string()).size(12.0).color(egui::Color32::WHITE));
                        });
                        ui.add_space(4.0);
                    }
                });
            });
        });

        ui.add_space(10.0);
        
        // 4. AI Filter Banner
        egui::Frame::none().fill(BG_CARD).rounding(10.0).inner_margin(10.0).stroke(egui::Stroke::new(1.0, BORDER)).show(ui, |ui| {
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(egui::Vec2::new(48.0, 48.0), egui::Sense::hover());
                ui.painter().rect_filled(r, 12.0, egui::Color32::from_rgb(20, 12, 40));
                ui.painter().rect_stroke(r, 12.0, egui::Stroke::new(1.0, PURPLE));
                ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, "AI", egui::FontId::proportional(20.0), NEON);
                
                ui.add_space(10.0);
                
                ui.vertical(|ui| {
                    ui.add_space(3.0);
                    ui.label(egui::RichText::new("Filter").size(14.0).strong().color(egui::Color32::WHITE));
                    ui.label(egui::RichText::new("AI assistent orqali\nfilter qoidalari\ndoim donda faol.").size(10.0).color(DIM));
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(">").size(18.0).color(DIM));
                });
            });
        });
        }); // end ScrollArea
    }

    // ==================== STATISTIKA ====================
    fn page_statistika(&self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
        ui.label(egui::RichText::new("📊 Statistika").size(18.0).strong().color(NEON));
        ui.label(egui::RichText::new("Shaxsiy ma'lumotlar tahlili").size(11.0).color(DIM));
        ui.add_space(10.0);

        let total_req = self.state.stats_total_requests.load(Ordering::Relaxed);
        let text = self.state.stats_text_filtered.load(Ordering::Relaxed);
        let phone = self.state.stats_phone_filtered.load(Ordering::Relaxed);
        let email = self.state.stats_email_filtered.load(Ordering::Relaxed);
        let image = self.state.stats_image_filtered.load(Ordering::Relaxed);
        let total_pii = text + phone + email + image;

        ui.columns(3, |cols| {
            card_frame(BLUE).show(&mut cols[0], |ui| {
                ui.set_width(ui.available_width());
                ui.label(egui::RichText::new("So'rovlar").size(10.0).color(DIM));
                ui.label(egui::RichText::new(total_req.to_string()).size(22.0).strong().color(NEON));
            });
            card_frame(PURPLE).show(&mut cols[1], |ui| {
                ui.set_width(ui.available_width());
                ui.label(egui::RichText::new("PII blok").size(10.0).color(DIM));
                ui.label(egui::RichText::new(total_pii.to_string()).size(22.0).strong().color(NEON));
            });
            card_frame(GREEN).show(&mut cols[2], |ui| {
                ui.set_width(ui.available_width());
                ui.label(egui::RichText::new("Xavfsizlik").size(10.0).color(DIM));
                ui.label(egui::RichText::new(if total_pii > 0 { "Faol" } else { "Kutish" }).size(14.0).strong().color(GREEN));
            });
        });

        ui.add_space(10.0);

        card_frame(egui::Color32::from_rgb(40, 50, 80)).show(ui, |ui| {
            ui.label(egui::RichText::new("Kategoriya taqsimoti").size(13.0).strong().color(NEON));
            ui.add_space(8.0);
            
            let categories = [
                ("💬 Matn/PII", text, PURPLE),
                ("📞 Telefon", phone, BLUE),
                ("✉  Email", email, CYAN),
                ("👤 Rasm/Yuz", image, ORANGE),
            ];
            
            for (label, count, color) in categories {
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(egui::Vec2::new(8.0, 8.0), egui::Sense::hover());
                    ui.painter().circle_filled(r.center(), 4.0, color);
                    ui.label(egui::RichText::new(label).size(11.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(count.to_string()).size(13.0).strong().color(NEON));
                    });
                });
                let max_width = ui.available_width();
                let bar_width = if total_pii > 0 { (count as f32 / total_pii as f32) * max_width } else { 0.0 };
                let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(max_width, 4.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 2.0, egui::Color32::from_rgb(30, 35, 50));
                let bar_rect = egui::Rect::from_min_size(rect.min, egui::Vec2::new(bar_width.max(0.0), 4.0));
                ui.painter().rect_filled(bar_rect, 2.0, color);
                ui.add_space(5.0);
            }
        });
        }); // end ScrollArea
    }

    // ==================== FILTERLAR ====================
    fn page_filterlar(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
        ui.label(egui::RichText::new("🔒 Filterlar").size(18.0).strong().color(NEON));
        ui.label(egui::RichText::new("Qaysi ma'lumotlar filtrlansin").size(11.0).color(DIM));
        ui.add_space(10.0);

        card_frame(PURPLE).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new("PII Qoidalari").size(13.0).strong().color(NEON));
            ui.add_space(6.0);
            
            ui.checkbox(&mut self.filter_phone, egui::RichText::new("📞 Telefon (+998..)").size(11.0));
            ui.add_space(2.0);
            ui.checkbox(&mut self.filter_email, egui::RichText::new("✉  Email").size(11.0));
            ui.add_space(2.0);
            ui.checkbox(&mut self.filter_card, egui::RichText::new("💳 Karta raqami").size(11.0));
            ui.add_space(2.0);
            ui.checkbox(&mut self.filter_passport, egui::RichText::new("🛂 Passport").size(11.0));
            ui.add_space(2.0);
            ui.checkbox(&mut self.filter_pinfl, egui::RichText::new("🔢 PINFL").size(11.0));
            ui.add_space(2.0);
            ui.checkbox(&mut self.filter_name, egui::RichText::new("👤 Ism-familiya").size(11.0));
            ui.add_space(2.0);
            ui.checkbox(&mut self.filter_inn, egui::RichText::new("🏢 INN").size(11.0));
            ui.add_space(2.0);
            ui.checkbox(&mut self.filter_date, egui::RichText::new("📅 Tug'ilgan sana").size(11.0));
        });

        ui.add_space(10.0);

        card_frame(BLUE).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new("🧪 Filterni sinab ko'ring").size(13.0).strong().color(NEON));
            ui.add_space(6.0);
            ui.label(egui::RichText::new("Matn kiriting:").size(10.0).color(DIM));
            ui.add_space(4.0);
            
            ui.add(egui::TextEdit::multiline(&mut self.test_input)
                .hint_text("Masalan: tel: +998901234567")
                .desired_width(f32::INFINITY)
                .desired_rows(2)
                .font(egui::TextStyle::Monospace));
            
            ui.add_space(5.0);
            if ui.button(egui::RichText::new(" 🔍 Filtrlash ").size(12.0)).clicked() {
                let detector = crate::detector::Detector::new();
                let mapping = crate::filter::MappingStore::new();
                let filter = crate::filter::MultiFilter::new(mapping, "label".to_string());
                let detections = detector.detect_text(&self.test_input, &[]);
                if detections.is_empty() {
                    self.test_output = "✅ PII topilmadi.".to_string();
                } else {
                    let masked = filter.mask_text(&self.test_input, &detections);
                    self.test_output = format!("⚠ {} ta PII!\n{}", detections.len(), masked);
                }
            }
            
            if !self.test_output.is_empty() {
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(3.0);
                ui.label(egui::RichText::new(&self.test_output).size(11.0).color(
                    if self.test_output.starts_with("✅") { GREEN } else { ORANGE }
                ));
            }
        });
        }); // end ScrollArea
    }

    // ==================== SOZLAMALAR ====================
    fn page_sozlamalar(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
        ui.label(egui::RichText::new("⚙ Sozlamalar").size(18.0).strong().color(NEON));
        ui.label(egui::RichText::new("Proxy va tarmoq parametrlari").size(11.0).color(DIM));
        ui.add_space(10.0);

        card_frame(PURPLE).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new("Proxy Server").size(13.0).strong().color(NEON));
            ui.add_space(6.0);
            
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Port:").size(11.0));
                ui.add(egui::TextEdit::singleline(&mut self.proxy_port).desired_width(60.0));
            });
            
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Maska:").size(11.0));
                egui::ComboBox::from_id_salt("mask_combo")
                    .selected_text(&self.mask_style)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.mask_style, "label".to_string(), "Label");
                        ui.selectable_value(&mut self.mask_style, "partial".to_string(), "Qisman");
                        ui.selectable_value(&mut self.mask_style, "pseudonym".to_string(), "Psevdonim");
                    });
            });
        });

        ui.add_space(10.0);
        
        card_frame(BLUE).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new("Tizim sozlamalari").size(13.0).strong().color(NEON));
            ui.add_space(6.0);
            ui.label(egui::RichText::new("PAC skripti yordamida avtomatik boshqariladi.").size(10.0).color(DIM));
            ui.add_space(5.0);
            
            card_frame(egui::Color32::from_rgb(40, 50, 80)).show(ui, |ui| {
                ui.label(egui::RichText::new(format!("PAC: 127.0.0.1:{}/pac", self.proxy_port)).size(10.0).color(GREEN).monospace());
                ui.label(egui::RichText::new("Holati: Sozlangan").size(10.0).color(GREEN));
            });
        });
        }); // end ScrollArea
    }

    // ==================== LOGS ====================
    fn page_logs(&self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("📋 Logs").size(18.0).strong().color(NEON));
        ui.label(egui::RichText::new("Faoliyat tarixi").size(11.0).color(DIM));
        ui.add_space(10.0);

        let logs = self.state.logs.read();
        
        if logs.is_empty() {
            card_frame(egui::Color32::from_rgb(40, 50, 80)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_min_height(120.0);
                ui.vertical_centered(|ui| {
                    ui.add_space(25.0);
                    ui.label(egui::RichText::new("📭").size(32.0));
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Faoliyat yo'q").size(13.0).color(DIM));
                    ui.add_space(5.0);
                    ui.label(egui::RichText::new("So'rov yuboring").size(10.0).color(egui::Color32::from_rgb(90, 100, 120)));
                });
            });
        } else {
            card_frame(egui::Color32::from_rgb(40, 50, 80)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Vaqt").size(10.0).strong().color(DIM));
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("Tur").size(10.0).strong().color(DIM));
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("Asl").size(10.0).strong().color(DIM));
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("Niqob").size(10.0).strong().color(DIM));
                });
                ui.separator();
                
                egui::ScrollArea::vertical().max_height(500.0).show(ui, |ui| {
                    for entry in logs.iter().rev() {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(&entry.timestamp).size(9.0).color(DIM).monospace());
                            ui.add_space(4.0);
                            let cat_color = match entry.category.as_str() {
                                "Telefon" => BLUE,
                                "Email" => CYAN,
                                _ => PURPLE,
                            };
                            ui.label(egui::RichText::new(&entry.category).size(9.0).color(cat_color));
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new(&entry.original).size(9.0).color(RED).monospace());
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new(&entry.masked).size(9.0).color(GREEN).monospace());
                        });
                        ui.separator();
                    }
                });
            });
        }
    }

    // ==================== YORDAM ====================
    fn page_yordam(&self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
        ui.label(egui::RichText::new("❓ Yordam").size(18.0).strong().color(NEON));
        ui.add_space(10.0);

        let faqs = [
            ("AI filter nima?",
             "Brauzer va AI xizmatlari orasida proksi. Shaxsiy ma'lumotlarni aniqlaydi va maskalaydi."),
            ("Qanday ishlaydi?",
             "1. Dasturni ishga tushiring\n2. AI xizmatlaridan foydalaning\n3. Dastur avtomatik filtrlaydi"),
            ("Qaysi ma'lumotlar?",
             "Telefon, email, karta, passport, PINFL, INN, tug'ilgan sana."),
            ("Saqlanadimi?",
             "Yo'q! Faqat RAM da. Hech narsa tashqariga yuborilmaydi."),
        ];

        for (question, answer) in faqs {
            card_frame(egui::Color32::from_rgb(30, 40, 70)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(egui::RichText::new(question).size(12.0).strong().color(BLUE));
                ui.add_space(3.0);
                ui.label(egui::RichText::new(answer).size(10.0).color(egui::Color32::from_rgb(180, 190, 220)));
            });
            ui.add_space(5.0);
        }
        }); // end ScrollArea
    }

    // ==================== HAQIDA ====================
    fn page_haqida(&self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("ℹ Haqida").size(18.0).strong().color(NEON));
        ui.add_space(10.0);

        card_frame(PURPLE).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.label(egui::RichText::new("🛡").size(40.0));
                ui.add_space(6.0);
                ui.label(egui::RichText::new("PrivacyProxy").size(20.0).strong().color(NEON));
                ui.label(egui::RichText::new("v1.1.2").size(12.0).color(DIM));
                ui.add_space(10.0);
                ui.label(egui::RichText::new("Shaxsiy ma'lumotlarni\nAI xizmatlaridan himoya\nqilish uchun lokal proksi").size(11.0).color(egui::Color32::from_rgb(180, 190, 220)));
                ui.add_space(10.0);
            });
        });
    }
}

use std::sync::Arc;
use eframe::egui;
use std::sync::atomic::Ordering;
use crate::state::AppState;
use crate::detector::DetectionType;
use egui_plot::{Line, Plot, PlotPoints};
use tray_icon::{TrayIconEvent, menu::MenuEvent};

pub struct PrivacyProxyApp {
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

impl PrivacyProxyApp {
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
                    } else if event.id == quit_id {
                        state_clone.sync_system_proxy(false);
                        std::process::exit(0);
                    }
                }
                
                if let Ok(event) = tray_channel.try_recv() {
                    if let tray_icon::TrayIconEvent::DoubleClick { .. } = event {
                        ctx_clone.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                        ctx_clone.send_viewport_cmd(egui::ViewportCommand::Focus);
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
const BG_DARK: egui::Color32 = egui::Color32::from_rgb(10, 12, 18);
const BG_CARD: egui::Color32 = egui::Color32::from_rgb(15, 18, 30);
const BG_SIDEBAR: egui::Color32 = egui::Color32::from_rgb(12, 14, 22);
const NEON: egui::Color32 = egui::Color32::from_rgb(230, 240, 255);
const DIM: egui::Color32 = egui::Color32::from_rgb(120, 130, 150);
const PURPLE: egui::Color32 = egui::Color32::from_rgb(130, 80, 255);
const BLUE: egui::Color32 = egui::Color32::from_rgb(80, 150, 255);
const CYAN: egui::Color32 = egui::Color32::from_rgb(80, 200, 255);
const ORANGE: egui::Color32 = egui::Color32::from_rgb(255, 150, 80);
const GREEN: egui::Color32 = egui::Color32::from_rgb(80, 255, 120);
const RED: egui::Color32 = egui::Color32::from_rgb(255, 80, 100);

fn card_frame(border: egui::Color32) -> egui::Frame {
    egui::Frame::none()
        .fill(BG_CARD)
        .rounding(15.0)
        .inner_margin(20.0)
        .stroke(egui::Stroke::new(1.0, border))
}

impl eframe::App for PrivacyProxyApp {
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
        
        // === SIDEBAR ===
        egui::SidePanel::left("sidebar")
            .exact_width(220.0)
            .frame(
                egui::Frame::none()
                    .fill(BG_SIDEBAR)
                    .inner_margin(egui::Margin::symmetric(20.0, 30.0))
            )
            .show(ctx, |ui| {
                ui.add_space(10.0);
                ui.label(egui::RichText::new("🛡 PrivacyProxy").size(24.0).color(NEON).strong());
                ui.add_space(40.0);
                
                let tabs = [
                    ("Bosh sahifa", "🏠"),
                    ("Statistika", "📊"),
                    ("Filterlar", "🔒"),
                    ("Sozlamalar", "⚙"),
                    ("Logs", "📋"),
                    ("Yordam", "❓"),
                    ("Haqida", "ℹ"),
                ];
                
                for (name, icon) in tabs {
                    let is_selected = self.selected_tab == name;
                    
                    let bg = if is_selected {
                        egui::Color32::from_rgba_unmultiplied(80, 60, 200, 150)
                    } else {
                        egui::Color32::TRANSPARENT
                    };
                    let tc = if is_selected {
                        egui::Color32::from_rgb(200, 200, 255)
                    } else {
                        DIM
                    };
                    
                    let frame = egui::Frame::none().fill(bg).rounding(8.0).inner_margin(egui::Margin::symmetric(15.0, 12.0));
                    let resp = frame.show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(egui::RichText::new(format!("{}   {}", icon, name)).size(16.0).color(tc));
                    }).response;
                    
                    let interact = ui.interact(resp.rect, ui.id().with(name), egui::Sense::click());
                    if interact.clicked() {
                        self.selected_tab = name.to_string();
                    }
                    if interact.hovered() && !is_selected {
                        ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    ui.add_space(5.0);
                }
                
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(20.0);
                    card_frame(egui::Color32::from_rgb(30, 40, 70)).show(ui, |ui| {
                        ui.label(egui::RichText::new("🛡 v1.0.0\nPrivacy Proxy").size(14.0));
                    });
                });
            });

        // === CENTRAL PANEL ===
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BG_DARK).inner_margin(30.0))
            .show(ctx, |ui| {
                match self.selected_tab.as_str() {
                    "Bosh sahifa" => self.page_dashboard(ui),
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
impl PrivacyProxyApp {

    // ==================== BOSH SAHIFA ====================
    fn page_dashboard(&self, ui: &mut egui::Ui) {
        // Header
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("🛡 PrivacyProxy Dashboard").size(32.0).strong().color(NEON));
                ui.label(egui::RichText::new("Sizning maxfiyligingiz — bizning ustuvor vazifamiz").size(16.0).color(DIM));
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let active = self.state.is_active();
                let dot = if active { GREEN } else { RED };
                let bg = if active { egui::Color32::from_rgb(20, 40, 25) } else { egui::Color32::from_rgb(40, 20, 25) };
                let border = if active { egui::Color32::from_rgb(50, 150, 80) } else { egui::Color32::from_rgb(150, 50, 80) };
                
                let resp = egui::Frame::none().fill(bg).rounding(20.0)
                    .inner_margin(egui::Margin::symmetric(15.0, 8.0))
                    .stroke(egui::Stroke::new(1.0, border))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(12.0, 12.0), egui::Sense::hover());
                            ui.painter().circle_filled(rect.center(), 6.0, dot);
                            ui.add_space(5.0);
                            ui.label(egui::RichText::new(if active { "Yoqilgan" } else { "O'chirilgan" }).color(dot));
                        });
                    }).response;
                let interact = ui.interact(resp.rect, ui.id().with("toggle"), egui::Sense::click());
                if interact.clicked() { self.state.set_active(!active); }
                if interact.hovered() { ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand); }
            });
        });

        ui.add_space(25.0);

        // Stat cards
        ui.columns(4, |cols| {
            let items = [
                ("Matn/PII", self.state.stats_text_filtered.load(Ordering::Relaxed), PURPLE, "💬"),
                ("Telefon", self.state.stats_phone_filtered.load(Ordering::Relaxed), BLUE, "📞"),
                ("Email", self.state.stats_email_filtered.load(Ordering::Relaxed), CYAN, "✉"),
                ("Rasm/Yuz", self.state.stats_image_filtered.load(Ordering::Relaxed), ORANGE, "👤"),
            ];
            for (i, (title, val, color, icon)) in items.iter().enumerate() {
                card_frame(*color).show(&mut cols[i], |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(*icon).size(24.0));
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(*title).size(14.0).color(DIM));
                            ui.label(egui::RichText::new(val.to_string()).size(32.0).strong().color(NEON));
                            ui.label(egui::RichText::new("bloklangan").size(12.0).color(egui::Color32::from_rgb(100, 110, 130)));
                        });
                    });
                });
            }
        });

        ui.add_space(25.0);

        // Chart
        card_frame(egui::Color32::from_rgb(40, 50, 80)).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Faoliyat statistikasi (bugun)").size(18.0).strong().color(NEON));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::Frame::none().fill(PURPLE).rounding(10.0).inner_margin(egui::Margin::symmetric(10.0, 5.0)).show(ui, |ui| {
                        ui.label(egui::RichText::new("24 soat").color(egui::Color32::WHITE).size(12.0));
                    });
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("7 kun").size(12.0).color(DIM));
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("30 kun").size(12.0).color(DIM));
                });
            });
            ui.add_space(10.0);

            let line = Line::new(PlotPoints::from_iter((0..100).map(|i| {
                let x = i as f64 * 0.1;
                [x, (x.sin() * 2.0) + (x * 1.5).cos() + 4.0]
            }))).color(BLUE).width(2.0);
            Plot::new("stats_plot").view_aspect(4.0).show_axes([true, true]).show_grid(true)
                .show(ui, |p| p.line(line));
        });

        ui.add_space(25.0);

        // Bottom 2 cards
        ui.columns(2, |cols| {
            // Filter holati
            card_frame(egui::Color32::from_rgb(30, 40, 70)).show(&mut cols[0], |ui| {
                ui.set_width(ui.available_width());
                ui.set_min_height(130.0);
                ui.label(egui::RichText::new("Filter holati").size(16.0).strong().color(NEON));
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(80.0, 80.0), egui::Sense::hover());
                    let total = self.state.stats_text_filtered.load(Ordering::Relaxed)
                        + self.state.stats_phone_filtered.load(Ordering::Relaxed)
                        + self.state.stats_email_filtered.load(Ordering::Relaxed);
                    ui.painter().circle_stroke(rect.center(), 35.0, egui::Stroke::new(8.0, egui::Color32::from_rgb(50, 60, 100)));
                    if total > 0 {
                        ui.painter().circle_stroke(rect.center(), 35.0, egui::Stroke::new(4.0, GREEN));
                    }
                    ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, &format!("{}", total), egui::FontId::proportional(20.0), NEON);
                    ui.add_space(25.0);
                    ui.vertical(|ui| {
                        for (name, color, val) in [
                            ("Matn/PII", PURPLE, self.state.stats_text_filtered.load(Ordering::Relaxed)),
                            ("Telefon", BLUE, self.state.stats_phone_filtered.load(Ordering::Relaxed)),
                            ("Email", CYAN, self.state.stats_email_filtered.load(Ordering::Relaxed)),
                            ("Rasm/Yuz", ORANGE, self.state.stats_image_filtered.load(Ordering::Relaxed)),
                        ] {
                            ui.horizontal(|ui| {
                                let (r, _) = ui.allocate_exact_size(egui::Vec2::new(8.0, 8.0), egui::Sense::hover());
                                ui.painter().circle_filled(r.center(), 4.0, color);
                                ui.label(egui::RichText::new(name).size(13.0));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(egui::RichText::new(val.to_string()).size(13.0).color(NEON));
                                });
                            });
                        }
                    });
                });
            });
            // Eslatma
            card_frame(egui::Color32::from_rgb(30, 40, 70)).show(&mut cols[1], |ui| {
                ui.set_width(ui.available_width());
                ui.set_min_height(130.0);
                ui.label(egui::RichText::new("Eslatma").size(16.0).strong().color(NEON));
                ui.add_space(15.0);
                ui.horizontal(|ui| {
                    egui::Frame::none().fill(egui::Color32::from_rgb(40, 30, 80)).rounding(30.0).inner_margin(20.0)
                        .stroke(egui::Stroke::new(2.0, PURPLE))
                        .show(ui, |ui| { ui.label(egui::RichText::new("🛡").size(30.0)); });
                    ui.add_space(20.0);
                    ui.label(egui::RichText::new("Rust orqali filter qoidalari\ndoim fonda faol.").size(16.0).color(egui::Color32::from_rgb(180, 190, 220)));
                });
            });
        });
    }

    // ==================== STATISTIKA ====================
    fn page_statistika(&self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("📊 Batafsil Statistika").size(32.0).strong().color(NEON));
        ui.label(egui::RichText::new("Barcha ushlab qolingan shaxsiy ma'lumotlar tahlili").size(16.0).color(DIM));
        ui.add_space(25.0);

        let total_req = self.state.stats_total_requests.load(Ordering::Relaxed);
        let text = self.state.stats_text_filtered.load(Ordering::Relaxed);
        let phone = self.state.stats_phone_filtered.load(Ordering::Relaxed);
        let email = self.state.stats_email_filtered.load(Ordering::Relaxed);
        let image = self.state.stats_image_filtered.load(Ordering::Relaxed);
        let total_pii = text + phone + email + image;

        ui.columns(3, |cols| {
            card_frame(BLUE).show(&mut cols[0], |ui| {
                ui.set_width(ui.available_width());
                ui.label(egui::RichText::new("Jami so'rovlar").size(14.0).color(DIM));
                ui.label(egui::RichText::new(total_req.to_string()).size(36.0).strong().color(NEON));
            });
            card_frame(PURPLE).show(&mut cols[1], |ui| {
                ui.set_width(ui.available_width());
                ui.label(egui::RichText::new("Jami PII bloklangan").size(14.0).color(DIM));
                ui.label(egui::RichText::new(total_pii.to_string()).size(36.0).strong().color(NEON));
            });
            card_frame(GREEN).show(&mut cols[2], |ui| {
                ui.set_width(ui.available_width());
                ui.label(egui::RichText::new("Xavfsizlik darajasi").size(14.0).color(DIM));
                ui.label(egui::RichText::new(if total_pii > 0 { "Faol himoya" } else { "Kutilmoqda" }).size(20.0).strong().color(GREEN));
            });
        });

        ui.add_space(25.0);

        card_frame(egui::Color32::from_rgb(40, 50, 80)).show(ui, |ui| {
            ui.label(egui::RichText::new("Kategoriya bo'yicha taqsimot").size(18.0).strong().color(NEON));
            ui.add_space(15.0);
            
            let categories = [
                ("💬 Matn/PII (ism, passport, PINFL, karta, INN, sana)", text, PURPLE),
                ("📞 Telefon raqamlari", phone, BLUE),
                ("✉  Email manzillari", email, CYAN),
                ("👤 Rasm/Yuz", image, ORANGE),
            ];
            
            for (label, count, color) in categories {
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(egui::Vec2::new(10.0, 10.0), egui::Sense::hover());
                    ui.painter().circle_filled(r.center(), 5.0, color);
                    ui.label(egui::RichText::new(label).size(15.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(count.to_string()).size(18.0).strong().color(NEON));
                    });
                });
                // Simple bar
                let max_width = ui.available_width();
                let bar_width = if total_pii > 0 { (count as f32 / total_pii as f32) * max_width } else { 0.0 };
                let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(max_width, 6.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 3.0, egui::Color32::from_rgb(30, 35, 50));
                let bar_rect = egui::Rect::from_min_size(rect.min, egui::Vec2::new(bar_width.max(0.0), 6.0));
                ui.painter().rect_filled(bar_rect, 3.0, color);
                ui.add_space(8.0);
            }
        });
    }

    // ==================== FILTERLAR ====================
    fn page_filterlar(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("🔒 Filter Sozlamalari").size(32.0).strong().color(NEON));
        ui.label(egui::RichText::new("Qaysi turdagi shaxsiy ma'lumotlar filtrlansin").size(16.0).color(DIM));
        ui.add_space(25.0);

        card_frame(PURPLE).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new("PII Aniqlash Qoidalari").size(18.0).strong().color(NEON));
            ui.add_space(10.0);
            
            ui.checkbox(&mut self.filter_phone, egui::RichText::new("📞 Telefon raqamlari (+998 XX XXX XX XX)").size(15.0));
            ui.add_space(4.0);
            ui.checkbox(&mut self.filter_email, egui::RichText::new("✉  Email manzillari (user@example.com)").size(15.0));
            ui.add_space(4.0);
            ui.checkbox(&mut self.filter_card, egui::RichText::new("💳 Karta raqamlari (8600 XXXX XXXX XXXX)").size(15.0));
            ui.add_space(4.0);
            ui.checkbox(&mut self.filter_passport, egui::RichText::new("🛂 Passport seriyasi (AB1234567)").size(15.0));
            ui.add_space(4.0);
            ui.checkbox(&mut self.filter_pinfl, egui::RichText::new("🔢 PINFL (14 raqamli identifikator)").size(15.0));
            ui.add_space(4.0);
            ui.checkbox(&mut self.filter_name, egui::RichText::new("👤 Ism-familiya (Bosh harfli so'zlar)").size(15.0));
            ui.add_space(4.0);
            ui.checkbox(&mut self.filter_inn, egui::RichText::new("🏢 INN (9 raqamli soliq raqami)").size(15.0));
            ui.add_space(4.0);
            ui.checkbox(&mut self.filter_date, egui::RichText::new("📅 Tug'ilgan sana (DD.MM.YYYY)").size(15.0));
        });

        ui.add_space(25.0);

        // Test maydoni
        card_frame(BLUE).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new("🧪 Filterni sinab ko'ring").size(18.0).strong().color(NEON));
            ui.add_space(10.0);
            ui.label(egui::RichText::new("Matn kiriting va filtr qanday ishlashini ko'ring:").size(14.0).color(DIM));
            ui.add_space(8.0);
            
            ui.add(egui::TextEdit::multiline(&mut self.test_input)
                .hint_text("Masalan: Mening ismim Anvar Karimov, tel: +998901234567, email: anvar@mail.com")
                .desired_width(f32::INFINITY)
                .desired_rows(3)
                .font(egui::TextStyle::Monospace));
            
            ui.add_space(8.0);
            if ui.button(egui::RichText::new("  🔍 Filtrlash  ").size(16.0)).clicked() {
                let detector = crate::detector::Detector::new();
                let mapping = crate::filter::MappingStore::new();
                let filter = crate::filter::MultiFilter::new(mapping, "label".to_string());
                let detections = detector.detect_text(&self.test_input, &[]);
                if detections.is_empty() {
                    self.test_output = "✅ Hech qanday shaxsiy ma'lumot topilmadi.".to_string();
                } else {
                    let masked = filter.mask_text(&self.test_input, &detections);
                    self.test_output = format!("⚠ {} ta PII topildi!\n\nMaskalangan natija:\n{}", detections.len(), masked);
                }
            }
            
            if !self.test_output.is_empty() {
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(5.0);
                ui.label(egui::RichText::new(&self.test_output).size(14.0).color(
                    if self.test_output.starts_with("✅") { GREEN } else { ORANGE }
                ));
            }
        });
    }

    // ==================== SOZLAMALAR ====================
    fn page_sozlamalar(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("⚙ Dastur Sozlamalari").size(32.0).strong().color(NEON));
        ui.label(egui::RichText::new("Proxy va tarmoq parametrlarini boshqarish").size(16.0).color(DIM));
        ui.add_space(25.0);

        card_frame(PURPLE).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new("Proxy Server").size(18.0).strong().color(NEON));
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Port:").size(15.0));
                ui.add(egui::TextEdit::singleline(&mut self.proxy_port).desired_width(80.0));
                ui.label(egui::RichText::new(format!("(hozir: 127.0.0.1:{})", self.proxy_port)).size(13.0).color(DIM));
            });
            
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Maskalash usuli:").size(15.0));
                egui::ComboBox::from_id_salt("mask_combo")
                    .selected_text(&self.mask_style)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.mask_style, "label".to_string(), "Label ([TELEFON], [EMAIL])");
                        ui.selectable_value(&mut self.mask_style, "partial".to_string(), "Qisman (+9****)");
                        ui.selectable_value(&mut self.mask_style, "pseudonym".to_string(), "Psevdonim (ID-a1b2c3)");
                    });
            });
        });

        ui.add_space(20.0);
        
        card_frame(BLUE).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new("Avtomatik tizim sozlamalari").size(18.0).strong().color(NEON));
            ui.add_space(10.0);
            ui.label(egui::RichText::new("Dastur Windows proksi parametrlarini PAC skripti yordamida avtomatik boshqaradi.").size(14.0).color(DIM));
            ui.add_space(8.0);
            
            card_frame(egui::Color32::from_rgb(40, 50, 80)).show(ui, |ui| {
                ui.label(egui::RichText::new(format!("PAC Manzil: http://127.0.0.1:{}/pac", self.proxy_port)).size(15.0).color(GREEN).monospace());
                ui.label(egui::RichText::new("Holati: Avtomatik sozlangan").size(15.0).color(GREEN));
            });

            ui.add_space(8.0);
            ui.label(egui::RichText::new("Windows sozlamalarining 'Proxy' bo'limida natijani ko'rishingiz mumkin.").size(13.0).color(DIM));
        });
    }

    // ==================== LOGS ====================
    fn page_logs(&self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("📋 Faoliyat Tarixi (Logs)").size(32.0).strong().color(NEON));
        ui.label(egui::RichText::new("Ushlab qolingan barcha shaxsiy ma'lumotlar ro'yxati").size(16.0).color(DIM));
        ui.add_space(25.0);

        let logs = self.state.logs.read();
        
        if logs.is_empty() {
            card_frame(egui::Color32::from_rgb(40, 50, 80)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_min_height(200.0);
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.label(egui::RichText::new("📭").size(48.0));
                    ui.add_space(15.0);
                    ui.label(egui::RichText::new("Hali hech qanday faoliyat yo'q").size(18.0).color(DIM));
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("Proxy orqali so'rov yuboring va natijalar shu yerda ko'rinadi").size(14.0).color(egui::Color32::from_rgb(90, 100, 120)));
                });
            });
        } else {
            card_frame(egui::Color32::from_rgb(40, 50, 80)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                // Table header
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Vaqt").size(13.0).strong().color(DIM));
                    ui.add_space(30.0);
                    ui.label(egui::RichText::new("Kategoriya").size(13.0).strong().color(DIM));
                    ui.add_space(30.0);
                    ui.label(egui::RichText::new("Asl qiymat").size(13.0).strong().color(DIM));
                    ui.add_space(30.0);
                    ui.label(egui::RichText::new("Niqob").size(13.0).strong().color(DIM));
                });
                ui.separator();
                
                egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                    for entry in logs.iter().rev() {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(&entry.timestamp).size(13.0).color(DIM).monospace());
                            ui.add_space(15.0);
                            let cat_color = match entry.category.as_str() {
                                "Telefon" => BLUE,
                                "Email" => CYAN,
                                _ => PURPLE,
                            };
                            ui.label(egui::RichText::new(&entry.category).size(13.0).color(cat_color));
                            ui.add_space(15.0);
                            ui.label(egui::RichText::new(&entry.original).size(13.0).color(RED).monospace());
                            ui.add_space(15.0);
                            ui.label(egui::RichText::new(&entry.masked).size(13.0).color(GREEN).monospace());
                        });
                        ui.separator();
                    }
                });
            });
        }
    }

    // ==================== YORDAM ====================
    fn page_yordam(&self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("❓ Yordam va FAQ").size(32.0).strong().color(NEON));
        ui.add_space(25.0);

        let faqs = [
            ("PrivacyProxy nima?",
             "Bu dastur sizning brauzeringiz va AI xizmatlari (ChatGPT, Claude, Gemini) o'rtasida proksi vazifasini bajaradi. U shaxsiy ma'lumotlarni avtomatik aniqlaydi va maskalaydi."),
            ("Qanday ishlaydi?",
             "1. Dasturni ishga tushiring (u avtomat tarzda tizim proxy sini sozlaydi)\n2. AI xizmatlaridan odatdagidek foydalaning\n3. Dastur fon rejimida shaxsiy ma'lumotlarni avtomatik filtrlaydi"),
            ("Qaysi ma'lumotlar filtrlanadi?",
             "Telefon raqamlari, email manzillari, bank karta raqamlari, passport seriyalari, PINFL, INN, tug'ilgan sanalar va qora ro'yxatdagi so'zlar."),
            ("Ma'lumotlarim saqlanadiimi?",
             "Yo'q! Barcha qayta ishlash faqat kompyuteringizda (RAM da) amalga oshiriladi. Hech qanday ma'lumot tashqariga yuborilmaydi."),
        ];

        for (question, answer) in faqs {
            card_frame(egui::Color32::from_rgb(30, 40, 70)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(egui::RichText::new(question).size(16.0).strong().color(BLUE));
                ui.add_space(5.0);
                ui.label(egui::RichText::new(answer).size(14.0).color(egui::Color32::from_rgb(180, 190, 220)));
            });
            ui.add_space(10.0);
        }
    }

    // ==================== HAQIDA ====================
    fn page_haqida(&self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("ℹ PrivacyProxy Haqida").size(32.0).strong().color(NEON));
        ui.add_space(25.0);

        card_frame(PURPLE).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical_centered(|ui| {
                ui.add_space(15.0);
                ui.label(egui::RichText::new("🛡").size(64.0));
                ui.add_space(10.0);
                ui.label(egui::RichText::new("PrivacyProxy").size(28.0).strong().color(NEON));
                ui.label(egui::RichText::new("v1.0.0").size(16.0).color(DIM));
                ui.add_space(20.0);
                ui.label(egui::RichText::new("Shaxsiy ma'lumotlarni AI xizmatlaridan himoya qilish uchun\nlokal proksi dasturi").size(16.0).color(egui::Color32::from_rgb(180, 190, 220)));
                ui.add_space(15.0);
            });
        });
        

    }
}

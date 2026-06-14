use std::sync::Arc;
use eframe::egui;
use std::sync::atomic::Ordering;
use crate::state::AppState;
use crate::detector::DetectionType;
use tray_icon::menu::{Menu, MenuItem};
use tray_icon::TrayIconEvent;

fn load_icon() -> tray_icon::Icon {
    let width = 16;
    let height = 16;
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            if x == 0 || y == 0 || x == width - 1 || y == height - 1 {
                rgba.extend_from_slice(&[0, 0, 0, 255]);
            } else {
                rgba.extend_from_slice(&[41, 128, 185, 255]);
            }
        }
    }
    tray_icon::Icon::from_rgba(rgba, width as u32, height as u32).unwrap()
}

use egui_plot::{Line, Plot, PlotPoints};

// === THEME DEFINITION ===
pub struct Theme {
 pub is_dark: bool,
 pub bg_app: egui::Color32,
 pub bg_card: egui::Color32,
 pub bg_sidebar: egui::Color32,
 
 pub text_main: egui::Color32,
 pub text_dim: egui::Color32,
 
 pub sidebar_text_main: egui::Color32,
 pub sidebar_text_dim: egui::Color32,
 pub sidebar_accent: egui::Color32,
 pub sidebar_bg_hover: egui::Color32,
 pub sidebar_bg_selected: egui::Color32,
 
 pub accent_blue: egui::Color32,
 pub accent_purple: egui::Color32,
 pub accent_green: egui::Color32,
 pub accent_red: egui::Color32,
 pub accent_orange: egui::Color32,
 pub accent_cyan: egui::Color32,
 
 pub border: egui::Color32,
}

impl Theme {
 pub fn get(is_dark: bool) -> Self {
 if is_dark {
 Self {
 is_dark: true,
 bg_app: egui::Color32::from_rgb(15, 15, 18),
 bg_card: egui::Color32::from_rgb(24, 24, 28),
 bg_sidebar: egui::Color32::from_rgb(20, 20, 24),
 text_main: egui::Color32::from_rgb(255, 255, 255),
 text_dim: egui::Color32::from_rgb(160, 160, 175),
 sidebar_text_main: egui::Color32::from_rgb(255, 255, 255),
 sidebar_text_dim: egui::Color32::from_rgb(160, 160, 175),
 sidebar_accent: egui::Color32::from_rgb(139, 92, 246),
 sidebar_bg_hover: egui::Color32::from_rgb(30, 30, 35),
 sidebar_bg_selected: egui::Color32::from_rgb(28, 28, 38),
 accent_blue: egui::Color32::from_rgb(59, 130, 246),
 accent_purple: egui::Color32::from_rgb(139, 92, 246),
 accent_green: egui::Color32::from_rgb(34, 197, 94),
 accent_red: egui::Color32::from_rgb(239, 68, 68),
 accent_orange: egui::Color32::from_rgb(249, 115, 22),
 accent_cyan: egui::Color32::from_rgb(14, 165, 233),
 border: egui::Color32::from_rgb(39, 39, 42),
 }
 } else {
 Self {
 is_dark: false,
 bg_app: egui::Color32::from_rgb(240, 244, 250),
 bg_card: egui::Color32::from_rgb(255, 255, 255),
 bg_sidebar: egui::Color32::from_rgb(20, 110, 255),
 text_main: egui::Color32::from_rgb(20, 25, 40),
 text_dim: egui::Color32::from_rgb(110, 120, 140),
 sidebar_text_main: egui::Color32::from_rgb(255, 255, 255),
 sidebar_text_dim: egui::Color32::from_rgb(180, 210, 255),
 sidebar_accent: egui::Color32::from_rgb(255, 255, 255),
 sidebar_bg_hover: egui::Color32::from_rgb(10, 95, 230),
 sidebar_bg_selected: egui::Color32::from_rgb(0, 80, 210),
 accent_blue: egui::Color32::from_rgb(20, 110, 255),
 accent_purple: egui::Color32::from_rgb(109, 40, 217),
 accent_green: egui::Color32::from_rgb(16, 185, 129),
 accent_red: egui::Color32::from_rgb(239, 68, 68),
 accent_orange: egui::Color32::from_rgb(245, 158, 11),
 accent_cyan: egui::Color32::from_rgb(6, 182, 212),
 border: egui::Color32::from_rgb(220, 225, 235),
 }
 }
 }
}


#[derive(PartialEq, Clone, Copy)]
pub enum Lang { Uz, En }

impl Lang {
 pub fn tr<'a>(&self, key: &'a str) -> &'a str {
 if *self == Lang::Uz { return key; }
 match key {
 "Bosh sahifa" => "Dashboard",

            "Jonli Faoliyat (Real-time)" => "Live Activity (Real-time)",
            "Topilgan PII" => "Found PII",
            "Telefonlar" => "Phones",
            "Emaillar" => "Emails",
            "So'rovlar" => "Requests",
            "O'chirilgan" => "Disabled",
            "Yoqilgan" => "Enabled",
            "Kunduzgi" => "Light",
            "Tungi" => "Dark",
            "Activity History" => "Activity History",

 "AI Xizmatlari" => "AI Services",
 "Statistika" => "Statistics",
 "Filterlar" => "Filters",
 "Sozlamalar" => "Settings",
 "Logs" => "Logs",
 "Yordam" => "Help",
 "Haqida" => "About",
 " Tungi" => " Dark",
 " Kunduzgi" => " Light",
 " Yoqilgan" => " Enabled",
 " O'chirilgan" => " Disabled",
 "Online" => "Online",
 " YANGI VERSIYA: " => " NEW VERSION: ",
 "Dastur muvaffaqiyatli yangilandi! \nO'zgarishlar kuchga kirishi uchun dasturni yoping va qayta ishga tushiring." => "Update successful! \nRestart the app to apply changes.",
 "Hozirda qaysi AI platformaga kirsangiz, o'sha avtomatik faollashadi." => "Whichever AI platform you open will be activated automatically.",
 "Filter FAOL (Avtomatik kuzatuv)" => "Filter ACTIVE (Auto monitoring)",
 "Filter O'CHIRILGAN" => "Filter DISABLED",
 " FAOL" => " ACTIVE",
 "Kutmoqda..." => "Waiting...",
 "Qanday ishlaydi?" => "How it works?",
 "AI xizmatlariga kirsangiz, dastur trafikni tekshiradi va qaysi AIga kirganingizni aniqlab faollashadi." => "When you use AI services, the app monitors traffic and activates the corresponding AI.",
 "Xush kelibsiz!" => "Welcome!",
 "Tizimingiz to'liq himoyalangan va AI orqali ma'lumot sizishidan xavfsiz." => "Your system is fully protected from data leaks via AI.",
 " HIMOYA FAOL" => " PROTECTION ON",
 " Jonli Kuzatuv" => " Live Monitoring",
 " HOZIR ISHLATILYAPTI:" => " CURRENTLY IN USE:",
 "Barcha yuborilayotgan xabarlar xavfsiz filtrdan o'tkazilmoqda..." => "All outgoing messages are safely filtered...",
 " KUTMOQDA..." => " WAITING...",
 "Brauzerda AI ochilmagan" => "No AI opened in browser",
 "Qachonki ChatGPT yoki boshqa AI ga kirsangiz, bu yerda ko'rinadi." => "When you open ChatGPT or other AI, it will appear here.",
 " Umumiy Statistika" => " Overall Statistics",
 "Topilgan PII" => "Found PII",
 "Telefonlar" => "Phones",
 "Emaillar" => "Emails",
 "So'rovlar" => "Requests",
 " Tahlil va Statistika" => " Analysis & Statistics",
 "Dastur ishga tushgandan beri qancha shaxsiy ma'lumot himoyalanganini ko'rishingiz mumkin." => "See how much personal data has been protected since the app started.",
 "Umumiy So'rovlar" => "Total Requests",
 "Himoyalangan sir" => "Protected Secrets",
 "Xavfsizlik Holati" => "Security Status",
 "Faol Himoya" => "Active Protection",
 "Kutish holati" => "Standby",
 "Qanday turdagi ma'lumotlar yashirildi?" => "What types of data were hidden?",
 " Matn yoki Ismlar" => " Text or Names",
 " Telefon raqamlar" => " Phone Numbers",
 " Elektron pochtalar" => " Email Addresses",
 " Rasm yoki Yuzlar" => " Images or Faces",
 " Himoya Qoidalari" => " Protection Rules",
 "Barcha nozik ma'lumotlar avtomatik tarzda himoyalangan" => "All sensitive data is automatically protected",
 " Faol Himoya" => " Active Protection",
 "Telefon raqamlar" => "Phone numbers",
 "Email manzillar" => "Email addresses",
 "Karta raqamlari (Uzcard, Humo, Visa...)" => "Card numbers (Uzcard, Humo, Visa...)",
 "Passport seriyalari" => "Passport series",
 "JSHSHIR (PINFL)" => "Personal ID (PINFL)",
 "Ism va familiyalar" => "Names and Surnames",
 "INN" => "TIN (INN)",
 " Filterni sinab ko'ring" => " Try the Filter",
 "Matn kiriting (natija avtomatik chiqadi):" => "Enter text (result is automatic):",
 "Masalan: mening raqamim +998901234567" => "For example: my number is +998901234567",
 "Xavfsiz matn (PII topilmadi)" => "Safe text (No PII found)",
 "Tizim parametrlari avtomatik moslashtirilgan" => "System parameters are automatically adjusted",
 "Tarmoq holati" => "Network Status",
 " Xavfsiz ulanish avtomatik sozlandi va ishga tushdi." => " Secure connection is auto-configured and running.",
 "Dastur sizning kompyuteringizga moslashib, bo'sh portni o'zi topdi va tizim proksisini o'rnatdi. Sizdan hech qanday qo'shimcha sozlash talab etilmaydi!" => "The app found an empty port and set up system proxy automatically. No extra configuration required!",
 "Texnik ma'lumot" => "Technical Info",
 "Ajratilgan Port: " => "Allocated Port: ",
 "Maska turi: Avtomatik (Label)" => "Mask type: Auto (Label)",
 "Holati: Barqaror ishlamoqda" => "Status: Running stable",
 "Faoliyat tarixi" => "Activity History",
 "Faoliyat yo'q" => "No activity",
 "So'rov yuboring" => "Send a request",
 "Vaqt" => "Time",
 "Tur" => "Type",
 "Asl" => "Original",
 "Niqob" => "Masked",
 "AI filter nima?" => "What is AI filter?",
 "Brauzer va AI xizmatlari orasida proksi. Shaxsiy ma'lumotlarni aniqlaydi va maskalaydi." => "A proxy between browser and AI. It detects and masks PII.",
 "1. Dasturni ishga tushiring\n2. AI xizmatlaridan foydalaning\n3. Dastur avtomatik filtrlaydi" => "1. Run the app\n2. Use AI services\n3. The app filters automatically",
 "Qaysi ma'lumotlar?" => "What data?",
 "Telefon, email, karta, passport, PINFL, INN, tug'ilgan sana." => "Phone, email, card, passport, PINFL, TIN, birth date.",
 "Saqlanadimi?" => "Is it saved?",
 "Yo'q! Faqat RAM da. Hech narsa tashqariga yuborilmaydi." => "No! Only in RAM. Nothing is sent outside.",
 "Shaxsiy ma'lumotlarni\nAI xizmatlaridan himoya\nqilish uchun lokal proksi" => "Local proxy to protect\npersonal data from\nAI services",
 "Matn" => "Text",
 "Rasm" => "Image",
 "Maska:" => "Mask:",
 "Asl:" => "Original:",
 "Yangi" => "New",
 _ => key,
 }
 }
}

pub struct AIFilterApp {
 state: Arc<AppState>,
 selected_tab: String,
 filter_phone: bool,
 filter_email: bool,
 filter_card: bool,
 filter_passport: bool,
 filter_pinfl: bool,
 filter_name: bool,
 filter_inn: bool,
 filter_date: bool,
 proxy_port: String,
 mask_style: String,
 test_input: String,
 test_output: String,
 window_visible: bool,
 should_quit: bool,
 is_dark_mode: bool,
 lang: Lang,
 tray_icon: Option<tray_icon::TrayIcon>,
 open_id: tray_icon::menu::MenuId,
 quit_id: tray_icon::menu::MenuId,
}

impl AIFilterApp {
 pub fn new(
 cc: &eframe::CreationContext<'_>,
 state: Arc<AppState>,
 start_minimized: bool,
 ) -> Self {
 let is_dark_mode = false;
 let t = Theme::get(is_dark_mode);
 
 let mut visuals = if is_dark_mode { egui::Visuals::dark() } else { egui::Visuals::light() };
 visuals.panel_fill = t.bg_app;
 visuals.window_fill = t.bg_app;
 visuals.override_text_color = Some(t.text_main);
 visuals.widgets.noninteractive.bg_fill = t.bg_card;
 cc.egui_ctx.set_visuals(visuals);

 let window_visible = !start_minimized;

 if start_minimized {
 cc.egui_ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
 }

        let tray_menu = tray_icon::menu::Menu::new();
        let open_item = tray_icon::menu::MenuItem::new("Ochish / Open", true, None);
        let quit_item = tray_icon::menu::MenuItem::new("Chiqish / Quit", true, None);
        let open_id = open_item.id().clone();
        let quit_id = quit_item.id().clone();
        let _ = tray_menu.append(&open_item);
        let _ = tray_menu.append(&quit_item);
        
        let tray_icon = tray_icon::TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("AI filter (Himoya faol)")
            .with_icon(load_icon())
            .build()
            .ok();

        let ctx_clone = cc.egui_ctx.clone();
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(200));
                ctx_clone.request_repaint();
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
 window_visible,
 should_quit: false,
 is_dark_mode,
 lang: Lang::Uz,
 tray_icon,
 open_id,
 quit_id,
 }
 }
 
 pub fn theme(&self) -> Theme {
 Theme::get(self.is_dark_mode)
 }
}


fn card_frame(border: egui::Color32, bg: egui::Color32) -> egui::Frame {
 egui::Frame::none()
 .fill(bg)
 .rounding(14.0)
 .inner_margin(16.0)
 .stroke(egui::Stroke::new(1.0, border.linear_multiply(0.6)))
}

impl eframe::App for AIFilterApp {
 fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
 let t = self.theme();
 ctx.request_repaint_after(std::time::Duration::from_millis(500));
        if self.should_quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        if ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            self.window_visible = false;
        }

        if let Ok(event) = tray_icon::TrayIconEvent::receiver().try_recv() {
            if let tray_icon::TrayIconEvent::DoubleClick { .. } = event {
                self.window_visible = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
        }
        
        if let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
            if event.id == self.open_id {
                self.window_visible = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            } else if event.id == self.quit_id {
                self.should_quit = true;
            }
        }
 
 // === TOP HEADER ===
 egui::TopBottomPanel::top("header")
 .frame(egui::Frame::none().fill(t.bg_app).inner_margin(egui::Margin::symmetric(20.0, 15.0)))
 .show(ctx, |ui| {
 ui.horizontal_centered(|ui| {
 ui.label(egui::RichText::new("PrivacyProxy").size(20.0).strong().color(t.text_main));
 
 ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
 let lang_label = if self.lang == Lang::Uz { "UZ" } else { "EN" };
 let lang_btn = egui::Button::new(egui::RichText::new(lang_label).size(13.0).color(t.text_main))
 .fill(t.bg_card)
 .stroke(egui::Stroke::new(1.0, t.border))
 .rounding(8.0);
 if ui.add_sized(egui::Vec2::new(50.0, 32.0), lang_btn).clicked() {
 self.lang = if self.lang == Lang::Uz { Lang::En } else { Lang::Uz };
 }
 
 ui.add_space(10.0);

 // 1. Tungi / Kunduzgi tugma
 let icon = self.lang.tr(if self.is_dark_mode { "Tungi" } else { "Kunduzgi" });
 let theme_btn = egui::Button::new(egui::RichText::new(icon).size(13.0).color(t.text_main))
 .fill(t.bg_card)
 .stroke(egui::Stroke::new(1.0, t.border))
 .rounding(8.0);
 
 let theme_resp = ui.add_sized(egui::Vec2::new(80.0, 32.0), theme_btn);
 if theme_resp.clicked() {
 self.is_dark_mode = !self.is_dark_mode;
 let new_t = self.theme();
 let mut visuals = if self.is_dark_mode { egui::Visuals::dark() } else { egui::Visuals::light() };
 visuals.panel_fill = new_t.bg_app;
 visuals.window_fill = new_t.bg_app;
 visuals.override_text_color = Some(new_t.text_main);
 visuals.widgets.noninteractive.bg_fill = new_t.bg_card;
 ctx.set_visuals(visuals);
 }

 ui.add_space(12.0);
 
 // 2. Tizim holati (Yoqish/O'chirish)
 let active = self.state.is_active();
 let (text, color, border_color) = if active {
 ("Yoqilgan", t.accent_green, t.accent_green)
 } else {
 ("O'chirilgan", t.text_dim, t.border)
 };
 
 let proxy_btn = egui::Button::new(egui::RichText::new(text).size(13.0).strong().color(color))
 .fill(t.bg_card)
 .stroke(egui::Stroke::new(1.0, border_color))
 .rounding(8.0);
 
 let proxy_resp = ui.add_sized(egui::Vec2::new(120.0, 32.0), proxy_btn);
 if proxy_resp.clicked() {
 self.state.set_active(!active);
 }
 });
 });
 });

 // === ICON-SIDEBAR ===
 egui::SidePanel::left("sidebar")
 .exact_width(75.0)
 .frame(
 egui::Frame::none()
 .fill(t.bg_sidebar)
 .inner_margin(egui::Margin::symmetric(0.0, 10.0))
 )
 .show(ctx, |ui| {
 ui.vertical_centered(|ui| {
 ui.add_space(4.0);
 
        let tabs = [
            ("Bosh sahifa", t.sidebar_accent),
            ("AI", t.sidebar_text_dim),
            ("Tahlil va Filterlar", t.sidebar_text_dim),
            ("Logs", t.sidebar_text_dim),
            ("Sozlamalar", t.sidebar_text_dim),
            ("Yordam", t.sidebar_text_dim),
            ("Haqida", t.sidebar_text_dim),
        ];
 
 for (name, _accent) in tabs {
 let is_selected = self.selected_tab == name || (name == "AI" && self.selected_tab == "AI Xizmatlari");
 
 let (rect, resp) = ui.allocate_exact_size(egui::Vec2::new(75.0, 60.0), egui::Sense::click());
 let text_color = if is_selected { t.sidebar_text_main } else { t.sidebar_text_dim };
 
 if is_selected {
 let line_rect = egui::Rect::from_min_size(rect.min, egui::Vec2::new(3.0, rect.height()));
 ui.painter().rect_filled(line_rect, 0.0, t.sidebar_accent);
 let bg_rect = egui::Rect::from_min_size(rect.min + egui::Vec2::new(3.0, 0.0), egui::Vec2::new(rect.width() - 3.0, rect.height()));
 ui.painter().rect_filled(bg_rect, 0.0, t.sidebar_bg_selected);
 } else if resp.hovered() {
 ui.painter().rect_filled(rect, 0.0, t.sidebar_bg_hover);
 }
 
 if resp.hovered() {
 ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
 }
 
 let c = rect.center() - egui::Vec2::new(0.0, 10.0);
 
 // Glassmorphic Icon Background
 let icon_size = 32.0;
 let icon_rect = egui::Rect::from_center_size(c, egui::Vec2::new(icon_size, icon_size));
 
 let glass_bg = if is_selected { t.accent_purple.linear_multiply(0.15) } else { t.text_dim.linear_multiply(0.05) };
 let glass_border = if is_selected { t.accent_purple.linear_multiply(0.5) } else { t.border.linear_multiply(0.5) };
 
 ui.painter().rect_filled(icon_rect, 10.0, glass_bg);
 ui.painter().rect_stroke(icon_rect, 10.0, egui::Stroke::new(1.0, glass_border));

 // 3D Shadow and Icon Drawing
 let icon_color = if is_selected { egui::Color32::WHITE } else { t.sidebar_text_dim };
 let shadow_color = egui::Color32::from_black_alpha(100);
 let thick = 2.5;

 // Helper closure to draw paths with shadow
 let mut draw_icon = |offset_y: f32, stroke: egui::Stroke| {
 let c = c + egui::Vec2::new(0.0, offset_y);
 match name {
 "Bosh sahifa" => {
 // House shape
 ui.painter().line_segment([c + egui::vec2(-7.0, 1.0), c + egui::vec2(0.0, -6.0)], stroke);
 ui.painter().line_segment([c + egui::vec2(0.0, -6.0), c + egui::vec2(7.0, 1.0)], stroke);
 ui.painter().line_segment([c + egui::vec2(-5.0, 0.0), c + egui::vec2(-5.0, 7.0)], stroke);
 ui.painter().line_segment([c + egui::vec2(5.0, 0.0), c + egui::vec2(5.0, 7.0)], stroke);
 ui.painter().line_segment([c + egui::vec2(-5.0, 7.0), c + egui::vec2(-2.0, 7.0)], stroke);
 ui.painter().line_segment([c + egui::vec2(5.0, 7.0), c + egui::vec2(2.0, 7.0)], stroke);
 ui.painter().line_segment([c + egui::vec2(-2.0, 7.0), c + egui::vec2(-2.0, 3.0)], stroke);
 ui.painter().line_segment([c + egui::vec2(2.0, 7.0), c + egui::vec2(2.0, 3.0)], stroke);
 ui.painter().line_segment([c + egui::vec2(-2.0, 3.0), c + egui::vec2(2.0, 3.0)], stroke);
 }
 "AI" => {
 ui.painter().circle_stroke(c, 6.0, stroke);
 ui.painter().circle_filled(c + egui::vec2(-2.5, -1.5), 1.2, stroke.color);
 ui.painter().circle_filled(c + egui::vec2(2.5, -1.5), 1.2, stroke.color);
 ui.painter().line_segment([c + egui::vec2(-2.5, 2.5), c + egui::vec2(2.5, 2.5)], stroke);
 ui.painter().line_segment([c + egui::vec2(-8.0, 0.0), c + egui::vec2(-6.0, 0.0)], stroke);
 ui.painter().line_segment([c + egui::vec2(8.0, 0.0), c + egui::vec2(6.0, 0.0)], stroke);
 }
 "Tahlil va Filterlar" => {
 ui.painter().line_segment([c + egui::vec2(-5.0, 7.0), c + egui::vec2(-5.0, 0.0)], stroke);
 ui.painter().line_segment([c + egui::vec2(0.0, 7.0), c + egui::vec2(0.0, -4.0)], stroke);
 ui.painter().line_segment([c + egui::vec2(5.0, 7.0), c + egui::vec2(5.0, -1.0)], stroke);
 ui.painter().line_segment([c + egui::vec2(-8.0, 7.0), c + egui::vec2(8.0, 7.0)], stroke);
 }
                            "Sozlamalar" => {
                                ui.painter().line_segment([c + egui::vec2(-7.0, -3.0), c + egui::vec2(7.0, -3.0)], stroke);
                                ui.painter().circle_filled(c + egui::vec2(-2.0, -3.0), 2.5, stroke.color);
                                ui.painter().line_segment([c + egui::vec2(-7.0, 3.0), c + egui::vec2(7.0, 3.0)], stroke);
                                ui.painter().circle_filled(c + egui::vec2(2.0, 3.0), 2.5, stroke.color);
                            }
                            "Logs" => {
                                ui.painter().line_segment([c + egui::vec2(-2.0, -4.0), c + egui::vec2(6.0, -4.0)], stroke);
                                ui.painter().line_segment([c + egui::vec2(-2.0, 0.0), c + egui::vec2(5.0, 0.0)], stroke);
                                ui.painter().line_segment([c + egui::vec2(-2.0, 4.0), c + egui::vec2(6.0, 4.0)], stroke);
                                ui.painter().circle_filled(c + egui::vec2(-6.0, -4.0), 1.5, stroke.color);
                                ui.painter().circle_filled(c + egui::vec2(-6.0, 0.0), 1.5, stroke.color);
                                ui.painter().circle_filled(c + egui::vec2(-6.0, 4.0), 1.5, stroke.color);
                            }
 "Yordam" => {
 ui.painter().text(c, egui::Align2::CENTER_CENTER, "?", egui::FontId::proportional(16.0).clone(), stroke.color);
 ui.painter().circle_stroke(c, 8.0, stroke);
 }
 "Haqida" => {
 ui.painter().text(c, egui::Align2::CENTER_CENTER, "i", egui::FontId::proportional(14.0).clone(), stroke.color);
 ui.painter().circle_stroke(c, 8.0, stroke);
 }
 _ => {}
 }
 };

 // Draw shadow first (offset down)
 draw_icon(2.0, egui::Stroke::new(thick, shadow_color));
 // Draw main icon
 draw_icon(0.0, egui::Stroke::new(thick, icon_color));
 
 ui.painter().text(rect.center() + egui::Vec2::new(0.0, 16.0), egui::Align2::CENTER_CENTER, self.lang.tr(name), egui::FontId::proportional(9.0), text_color);
 
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
 let dot = if active { t.accent_green } else { t.accent_red };
 ui.horizontal(|ui| {
 let total_w = 40.0;
 ui.add_space((ui.available_width() - total_w) / 2.0);
 let (dot_rect, _) = ui.allocate_exact_size(egui::Vec2::new(6.0, 6.0), egui::Sense::hover());
 ui.painter().circle_filled(dot_rect.center(), 3.0, dot);
 ui.add_space(3.0);
 ui.label(egui::RichText::new(self.lang.tr("Online")).size(9.0).color(t.sidebar_text_main));
 });
 ui.add_space(4.0);
 ui.label(egui::RichText::new("v1.2.3").size(9.0).color(t.sidebar_text_dim));
 });
 });

 // === CENTRAL PANEL ===
 egui::CentralPanel::default()
 .frame(egui::Frame::none().fill(t.bg_app).inner_margin(16.0))
 .show(ctx, |ui| {
 if self.state.update_available.load(std::sync::atomic::Ordering::Relaxed) {
 card_frame(egui::Color32::from_rgb(200, 100, 0), t.bg_card).show(ui, |ui| {
 ui.horizontal(|ui| {
 ui.label(egui::RichText::new(self.lang.tr(" YANGI VERSIYA: ")).strong().color(egui::Color32::from_rgb(200, 100, 0)));
 ui.label(egui::RichText::new(self.lang.tr("Dastur muvaffaqiyatli yangilandi! \nO'zgarishlar kuchga kirishi uchun dasturni yoping va qayta ishga tushiring.")).color(t.text_main).size(11.0));
 });
 });
 ui.add_space(8.0);
 }

 match self.selected_tab.as_str() {
 "Bosh sahifa" => self.page_dashboard(ui, &t),
 "AI Xizmatlari" => self.page_ai_services(ui, &t),
 "Tahlil va Filterlar" => self.page_statistika(ui, &t),
 "Logs" => self.page_logs_content(ui, &t),
 "Sozlamalar" => self.page_sozlamalar(ui, &t),
 "Yordam" => self.page_yordam(ui, &t),
 "Haqida" => self.page_haqida(ui, &t),
 _ => {}
 }
 });

 ctx.request_repaint();
 }
}

// === PAGE IMPLEMENTATIONS ===
impl AIFilterApp {

 // ==================== AI XIZMATLARI ====================
 fn page_ai_services(&self, ui: &mut egui::Ui, t: &Theme) {
 egui::ScrollArea::vertical().show(ui, |ui| {
 ui.horizontal(|ui| {
 let (r, _) = ui.allocate_exact_size(egui::Vec2::new(24.0, 24.0), egui::Sense::hover());
 ui.painter().rect_filled(r, 6.0, t.bg_card);
 ui.painter().rect_stroke(r, 6.0, egui::Stroke::new(1.0, t.border));
 ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, "AI", egui::FontId::proportional(14.0), t.accent_blue);
 ui.add_space(6.0);
 ui.label(egui::RichText::new(self.lang.tr("AI Xizmatlari")).size(18.0).strong().color(t.text_main));
 });
 
 ui.add_space(3.0);
 ui.label(egui::RichText::new(self.lang.tr("Hozirda qaysi AI platformaga kirsangiz, o'sha avtomatik faollashadi.")).size(11.0).color(t.text_dim));
 ui.add_space(10.0);

 let active = self.state.is_active();
 card_frame(if active { t.accent_green } else { t.accent_red }, t.bg_card).show(ui, |ui| {
 ui.set_width(ui.available_width());
 ui.horizontal(|ui| {
 let dot_color = if active { t.accent_green } else { t.accent_red };
 let (r, _) = ui.allocate_exact_size(egui::Vec2::new(10.0, 10.0), egui::Sense::hover());
 ui.painter().circle_filled(r.center(), 5.0, dot_color);
 ui.add_space(5.0);
 ui.label(egui::RichText::new(if active { self.lang.tr("Filter FAOL (Avtomatik kuzatuv)") } else { self.lang.tr("Filter O'CHIRILGAN") }).size(12.0).strong().color(t.text_main));
 });
 });

 ui.add_space(10.0);

 let services = [
 ("ChatGPT", "chat.openai.com", "O", "OpenAI", t.accent_cyan, egui::Color32::from_rgb(0, 200, 100)),
 ("Claude", "claude.ai", "A", "Anthropic", t.accent_purple, egui::Color32::from_rgb(180, 100, 255)),
 ("Gemini", "gemini.google.com", "G", "Google", t.accent_blue, egui::Color32::from_rgb(80, 150, 255)),
 ("Copilot", "copilot.microsoft.com", "M", "Microsoft", t.accent_orange, egui::Color32::from_rgb(255, 120, 50)),
 ("Grok", "grok.x.ai", "X", "xAI", t.text_dim, egui::Color32::from_rgb(200, 200, 200)),
 ("Perplexity", "perplexity.ai", "P", "Perplexity AI", t.accent_green, egui::Color32::from_rgb(50, 200, 200)),
 ];

 let current_active_plat = self.state.active_platform.read().clone();

 for (name, _domain, initial, company, border_col, dot_col) in services {
 let is_active_now = current_active_plat.as_deref() == Some(name) && active;
 let border = if is_active_now { border_col } else { t.border };
 
 card_frame(border, t.bg_card).show(ui, |ui| {
 ui.set_width(ui.available_width());
 ui.horizontal(|ui| {
 let (r, _) = ui.allocate_exact_size(egui::Vec2::new(28.0, 28.0), egui::Sense::hover());
 ui.painter().circle_stroke(r.center(), 14.0, egui::Stroke::new(1.5, if is_active_now { dot_col } else { t.text_dim }));
 ui.painter().circle_filled(r.center(), 11.0, t.bg_app);
 ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, initial, egui::FontId::proportional(14.0), if is_active_now { border_col } else { t.text_dim });
 
 ui.add_space(8.0);
 ui.vertical(|ui| {
 ui.label(egui::RichText::new(name).size(13.0).strong().color(if is_active_now { t.text_main } else { t.text_dim }));
 ui.label(egui::RichText::new(company).size(10.0).color(t.text_dim));
 });
 
 ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Tizim holatini ko'rsatish
            if is_active_now {
                let btn = egui::Button::new(egui::RichText::new(self.lang.tr(" FAOL")).size(10.0).color(t.bg_card))
                    .fill(t.accent_green)
                    .rounding(6.0);
                ui.add(btn);
            } else {
                let btn = egui::Button::new(egui::RichText::new(self.lang.tr("Kutmoqda...")).size(10.0).color(t.text_dim))
                    .fill(t.bg_app)
                    .stroke(egui::Stroke::new(1.0, t.border))
                    .rounding(6.0);
                ui.add(btn);
            }
            
            ui.add_space(8.0);
            
            // Yoqish/O'chirish tugmasi
            let mut enabled = self.state.is_platform_enabled(name);
            if ui.checkbox(&mut enabled, "").changed() {
                self.state.toggle_platform(name, enabled);
            }
 });
 });
 });
 ui.add_space(4.0);
 }

 ui.add_space(10.0);
 card_frame(t.accent_cyan, t.bg_card).show(ui, |ui| {
 ui.set_width(ui.available_width());
 ui.horizontal(|ui| {
 let (r, _) = ui.allocate_exact_size(egui::Vec2::new(18.0, 18.0), egui::Sense::hover());
 ui.painter().circle_stroke(r.center(), 9.0, egui::Stroke::new(1.5, t.accent_cyan));
 ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, "i", egui::FontId::proportional(11.0), t.accent_cyan);
 
 ui.add_space(8.0);
 ui.vertical(|ui| {
 ui.label(egui::RichText::new(self.lang.tr("Qanday ishlaydi?")).size(12.0).strong().color(t.text_main));
 ui.label(egui::RichText::new(
 self.lang.tr("AI xizmatlariga kirsangiz, dastur trafikni tekshiradi va qaysi AIga kirganingizni aniqlab faollashadi.")
 ).size(10.0).color(t.text_dim));
 });
 });
 });
 }); // end ScrollArea
 }

 // ==================== BOSH SAHIFA ====================
 fn page_dashboard(&self, ui: &mut egui::Ui, t: &Theme) {
 egui::ScrollArea::vertical().show(ui, |ui| {
 ui.add_space(5.0);
 
 // 1. Asosiy Banner (Welcome)
 card_frame(t.border, t.accent_blue).show(ui, |ui| {
 ui.set_width(ui.available_width());
 ui.horizontal(|ui| {
 ui.vertical(|ui| {
 ui.label(egui::RichText::new(self.lang.tr("Xush kelibsiz!")).size(22.0).strong().color(egui::Color32::WHITE));
 ui.add_space(4.0);
 ui.label(egui::RichText::new(self.lang.tr("Tizimingiz to'liq himoyalangan va AI orqali ma'lumot sizishidan xavfsiz.")).size(12.0).color(egui::Color32::from_rgb(220, 230, 255)));
 });
 });
 });

 ui.add_space(15.0);

 // 2. JONLI KUZATUV (Live Active AI)
 ui.label(egui::RichText::new(self.lang.tr(" Jonli Kuzatuv")).size(16.0).strong().color(t.text_main));
 ui.add_space(6.0);
 
 let current_active_plat = self.state.active_platform.read().clone();
 let is_monitoring_active = current_active_plat.is_some() && self.state.is_active();
 
 card_frame(if is_monitoring_active { t.accent_cyan } else { t.border }, t.bg_card).show(ui, |ui| {
 ui.set_width(ui.available_width());
 ui.vertical_centered(|ui| {
 ui.add_space(10.0);
 if let Some(plat) = current_active_plat {
 if self.state.is_active() {
 ui.label(egui::RichText::new(self.lang.tr(" HOZIR ISHLATILYAPTI:")).size(11.0).color(t.accent_green).strong());
 ui.add_space(5.0);
 ui.label(egui::RichText::new(&plat).size(32.0).strong().color(t.accent_cyan));
 ui.add_space(5.0);
 ui.label(egui::RichText::new(self.lang.tr("Barcha yuborilayotgan xabarlar xavfsiz filtrdan o'tkazilmoqda...")).size(12.0).color(t.text_dim));
 }
 } else {
 ui.label(egui::RichText::new(self.lang.tr(" KUTMOQDA...")).size(11.0).color(t.text_dim).strong());
 ui.add_space(5.0);
 ui.label(egui::RichText::new(self.lang.tr("Brauzerda AI ochilmagan")).size(24.0).strong().color(t.text_dim));
 ui.add_space(5.0);
 ui.label(egui::RichText::new(self.lang.tr("Qachonki ChatGPT yoki boshqa AI ga kirsangiz, bu yerda ko'rinadi.")).size(12.0).color(t.text_dim));
 }
 ui.add_space(10.0);
 });
 });

 ui.add_space(15.0);

 // 3. Qisqacha Statistika
 ui.label(egui::RichText::new(self.lang.tr(" Umumiy Statistika")).size(16.0).strong().color(t.text_main));
 ui.add_space(6.0);
 
 ui.columns(4, |cols| {
 let items = [
 ("Topilgan PII", self.state.stats_text_filtered.load(Ordering::Relaxed), t.accent_purple, ""),
 ("Telefonlar", self.state.stats_phone_filtered.load(Ordering::Relaxed), t.accent_blue, ""),
 ("Emaillar", self.state.stats_email_filtered.load(Ordering::Relaxed), t.accent_cyan, ""),
 ("So'rovlar", self.state.stats_total_requests.load(Ordering::Relaxed), t.accent_orange, ""),
 ];
 for (i, (title, val, color, icon)) in items.iter().enumerate() {
 egui::Frame::none()
 .fill(t.bg_card)
 .rounding(14.0)
 .inner_margin(egui::Margin::same(16.0))
 .stroke(egui::Stroke::new(1.0, t.border))
 .show(&mut cols[i], |ui| {
 ui.set_width(ui.available_width());
 ui.vertical_centered(|ui| {
 ui.label(egui::RichText::new(*icon).size(24.0).color(*color));
 ui.add_space(6.0);
 ui.label(egui::RichText::new(val.to_string()).size(24.0).strong().color(t.text_main));
 ui.add_space(2.0);
 ui.label(egui::RichText::new(self.lang.tr(title)).size(11.0).color(t.text_dim));
 });
 });
 }
 });
 ui.add_space(15.0);
 }); // end ScrollArea
 }

 // ==================== STATISTIKA ====================
 fn page_statistika(&mut self, ui: &mut egui::Ui, t: &Theme) {
 egui::ScrollArea::vertical().show(ui, |ui| {
 ui.add_space(5.0);
 ui.label(egui::RichText::new(self.lang.tr(" Tahlil va Statistika")).size(20.0).strong().color(t.text_main));
 ui.label(egui::RichText::new(self.lang.tr("Dastur ishga tushgandan beri qancha shaxsiy ma'lumot himoyalanganini ko'rishingiz mumkin.")).size(12.0).color(t.text_dim));
 ui.add_space(15.0);

 let total_req = self.state.stats_total_requests.load(Ordering::Relaxed);
 let text = self.state.stats_text_filtered.load(Ordering::Relaxed);
 let phone = self.state.stats_phone_filtered.load(Ordering::Relaxed);
 let email = self.state.stats_email_filtered.load(Ordering::Relaxed);
 let image = self.state.stats_image_filtered.load(Ordering::Relaxed);
 let total_pii = text + phone + email + image;

 ui.columns(3, |cols| {
 card_frame(t.accent_blue, t.bg_card).show(&mut cols[0], |ui| {
 ui.set_width(ui.available_width());
 ui.vertical_centered(|ui| {
 ui.label(egui::RichText::new("").size(28.0));
 ui.add_space(5.0);
 ui.label(egui::RichText::new(self.lang.tr("Umumiy So'rovlar")).size(11.0).color(t.text_dim));
 ui.add_space(2.0);
 ui.label(egui::RichText::new(total_req.to_string()).size(26.0).strong().color(t.text_main));
 });
 });
 
 card_frame(t.accent_purple, t.bg_card).show(&mut cols[1], |ui| {
 ui.set_width(ui.available_width());
 ui.vertical_centered(|ui| {
 ui.label(egui::RichText::new("").size(28.0));
 ui.add_space(5.0);
 ui.label(egui::RichText::new(self.lang.tr("Himoyalangan sir")).size(11.0).color(t.text_dim));
 ui.add_space(2.0);
 ui.label(egui::RichText::new(total_pii.to_string()).size(26.0).strong().color(t.accent_purple));
 });
 });
 
 card_frame(t.accent_green, t.bg_card).show(&mut cols[2], |ui| {
 ui.set_width(ui.available_width());
 ui.vertical_centered(|ui| {
 ui.label(egui::RichText::new(if total_pii > 0 { "" } else { "" }).size(28.0));
 ui.add_space(5.0);
 ui.label(egui::RichText::new(self.lang.tr("Xavfsizlik Holati")).size(11.0).color(t.text_dim));
 ui.add_space(2.0);
 ui.label(egui::RichText::new(if total_pii > 0 { self.lang.tr("Faol Himoya") } else { self.lang.tr("Kutish holati") }).size(15.0).strong().color(if total_pii > 0 { t.accent_green } else { t.text_dim }));
 });
 });
 });

 ui.add_space(15.0);

 card_frame(t.border, t.bg_card).show(ui, |ui| {
 ui.label(egui::RichText::new(self.lang.tr("Qanday turdagi ma'lumotlar yashirildi?")).size(14.0).strong().color(t.text_main));
 ui.add_space(12.0);
 
 let categories = [
 (" Matn yoki Ismlar", text, t.accent_purple),
 (" Telefon raqamlar", phone, t.accent_blue),
 (" Elektron pochtalar", email, t.accent_cyan),
 (" Rasm yoki Yuzlar", image, t.accent_orange),
 ];
 
 for (label, count, color) in categories {
 ui.horizontal(|ui| {
 let (r, _) = ui.allocate_exact_size(egui::Vec2::new(10.0, 10.0), egui::Sense::hover());
 ui.painter().circle_filled(r.center(), 5.0, color);
 ui.label(egui::RichText::new(self.lang.tr(label)).size(13.0).color(t.text_main));
 ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
 ui.label(egui::RichText::new(if self.lang == Lang::Uz { format!("{} ta", count) } else { format!("{} items", count) }).size(14.0).strong().color(t.text_main));
 });
 });
 
 let max_width = ui.available_width();
 let bar_width = if total_pii > 0 { (count as f32 / total_pii as f32) * max_width } else { 0.0 };
 let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(max_width, 6.0), egui::Sense::hover());
 ui.painter().rect_filled(rect, 3.0, t.bg_app);
 let bar_rect = egui::Rect::from_min_size(rect.min, egui::Vec2::new(bar_width.max(0.0), 6.0));
 ui.painter().rect_filled(bar_rect, 3.0, color);
 ui.add_space(8.0);
 }
 });
    
    ui.add_space(20.0);
    self.page_filterlar_content(ui, t);
    ui.add_space(10.0);
    
 }); // end ScrollArea
 }

 // ==================== FILTERLAR ====================
 fn page_filterlar_content(&mut self, ui: &mut egui::Ui, t: &Theme) {
 ui.label(egui::RichText::new(self.lang.tr(" Himoya Qoidalari")).size(18.0).strong().color(t.text_main));
 ui.label(egui::RichText::new(self.lang.tr("Barcha nozik ma'lumotlar avtomatik tarzda himoyalangan")).size(11.0).color(t.text_dim));
 ui.add_space(10.0);

 card_frame(t.accent_purple, t.bg_card).show(ui, |ui| {
 ui.set_width(ui.available_width());
 ui.label(egui::RichText::new(self.lang.tr(" Faol Himoya")).size(13.0).strong().color(t.text_main));
 ui.add_space(6.0);
 
 let rules = [
 "Telefon raqamlar",
 "Email manzillar",
 "Karta raqamlari (Uzcard, Humo, Visa...)",
 "Passport seriyalari",
 "JSHSHIR (PINFL)",
 "Ism va familiyalar",
 "INN"
 ];
 
 for rule in rules {
 ui.horizontal(|ui| {
 ui.label(egui::RichText::new("✓").color(t.accent_green).strong());
 ui.label(egui::RichText::new(rule).size(11.0).color(t.text_main));
 });
 ui.add_space(2.0);
 }
 });

 ui.add_space(10.0);

 card_frame(t.accent_blue, t.bg_card).show(ui, |ui| {
 ui.set_width(ui.available_width());
 ui.label(egui::RichText::new(self.lang.tr(" Filterni sinab ko'ring")).size(13.0).strong().color(t.text_main));
 ui.add_space(6.0);
 ui.label(egui::RichText::new(self.lang.tr("Matn kiriting (natija avtomatik chiqadi):")).size(10.0).color(t.text_dim));
 ui.add_space(4.0);
 
 let response = ui.add(egui::TextEdit::multiline(&mut self.test_input)
 .hint_text(self.lang.tr("Masalan: mening raqamim +998901234567"))
 .desired_width(f32::INFINITY)
 .desired_rows(2)
 .font(egui::TextStyle::Monospace));
 
 if response.changed() || (self.test_output.is_empty() && !self.test_input.is_empty()) {
 let detector = crate::detector::Detector::new();
 let mapping = crate::filter::MappingStore::new();
 let filter = crate::filter::MultiFilter::new(mapping, "label".to_string());
 let detections = detector.detect_text(&self.test_input, &[]);
 if detections.is_empty() {
 self.test_output = "Xavfsiz matn (PII topilmadi)".to_string();
 } else {
 let masked = filter.mask_text(&self.test_input, &detections);
 self.test_output = format!("Ta'qiqlangan {} ta ma'lumot yashirildi:\n\n{}", detections.len(), masked);
 }
 } else if self.test_input.is_empty() {
 self.test_output.clear();
 }
 
 if !self.test_output.is_empty() {
 ui.add_space(6.0);
 ui.separator();
 ui.add_space(6.0);
 ui.label(egui::RichText::new(&self.test_output).size(12.0).color(
 if self.test_output.contains("topilmadi") { t.accent_green } else { t.accent_orange }
 ));
 }
 });
 }

 // ==================== SOZLAMALAR ====================
 fn page_sozlamalar(&mut self, ui: &mut egui::Ui, t: &Theme) {
 egui::ScrollArea::vertical().show(ui, |ui| {
 ui.label(egui::RichText::new(" Sozlamalar").size(18.0).strong().color(t.text_main));
 ui.label(egui::RichText::new(self.lang.tr("Tizim parametrlari avtomatik moslashtirilgan")).size(11.0).color(t.text_dim));
 ui.add_space(10.0);

 card_frame(t.accent_purple, t.bg_card).show(ui, |ui| {
 ui.set_width(ui.available_width());
 ui.label(egui::RichText::new(self.lang.tr("Tarmoq holati")).size(13.0).strong().color(t.text_main));
 ui.add_space(6.0);
 
 ui.label(egui::RichText::new(self.lang.tr(" Xavfsiz ulanish avtomatik sozlandi va ishga tushdi.")).size(12.0).color(t.text_main));
 ui.add_space(4.0);
 ui.label(egui::RichText::new(self.lang.tr("Dastur sizning kompyuteringizga moslashib, bo'sh portni o'zi topdi va tizim proksisini o'rnatdi. Sizdan hech qanday qo'shimcha sozlash talab etilmaydi!")).size(11.0).color(t.text_dim));
 });

 ui.add_space(10.0);
 
 card_frame(t.accent_blue, t.bg_card).show(ui, |ui| {
 ui.set_width(ui.available_width());
 ui.label(egui::RichText::new(self.lang.tr("Texnik ma'lumot")).size(13.0).strong().color(t.text_main));
 ui.add_space(6.0);
 
 card_frame(t.border, t.bg_app).show(ui, |ui| {
 ui.label(egui::RichText::new(format!("Ajratilgan Port: {}", self.proxy_port)).size(11.0).color(t.accent_green).monospace());
 ui.label(egui::RichText::new(self.lang.tr("Maska turi: Avtomatik (Label)")).size(11.0).color(t.accent_green));
 ui.label(egui::RichText::new(self.lang.tr("Holati: Barqaror ishlamoqda")).size(11.0).color(t.accent_green));
 });
 });
 }); // end ScrollArea
 }

 // ==================== LOGS ====================
 fn page_logs_content(&self, ui: &mut egui::Ui, t: &Theme) {
 ui.label(egui::RichText::new(" Logs").size(18.0).strong().color(t.text_main));
 ui.label(egui::RichText::new(self.lang.tr("Faoliyat tarixi")).size(11.0).color(t.text_dim));
 ui.add_space(10.0);

 let logs = self.state.logs.read();
 
 if logs.is_empty() {
 card_frame(t.border, t.bg_card).show(ui, |ui| {
 ui.set_width(ui.available_width());
 ui.set_min_height(120.0);
 ui.vertical_centered(|ui| {
 ui.add_space(25.0);
 ui.label(egui::RichText::new("").size(32.0));
 ui.add_space(8.0);
 ui.label(egui::RichText::new(self.lang.tr("Faoliyat yo'q")).size(13.0).color(t.text_dim));
 ui.add_space(5.0);
 ui.label(egui::RichText::new(self.lang.tr("So'rov yuboring")).size(10.0).color(t.text_dim));
 });
 });
 } else {
 card_frame(t.border, t.bg_card).show(ui, |ui| {
 ui.set_width(ui.available_width());
 ui.horizontal(|ui| {
 ui.label(egui::RichText::new(self.lang.tr("Vaqt")).size(10.0).strong().color(t.text_dim));
 ui.add_space(10.0);
 ui.label(egui::RichText::new(self.lang.tr("Tur")).size(10.0).strong().color(t.text_dim));
 ui.add_space(10.0);
 ui.label(egui::RichText::new(self.lang.tr("Asl")).size(10.0).strong().color(t.text_dim));
 ui.add_space(10.0);
 ui.label(egui::RichText::new(self.lang.tr("Niqob")).size(10.0).strong().color(t.text_dim));
 });
 ui.separator();
 
 egui::ScrollArea::vertical().max_height(500.0).show(ui, |ui| {
 for entry in logs.iter().rev() {
 ui.horizontal(|ui| {
 ui.label(egui::RichText::new(&entry.timestamp).size(9.0).color(t.text_dim).monospace());
 ui.add_space(4.0);
 let cat_color = match entry.category.as_str() {
 "Telefon" => t.accent_blue,
 "Email" => t.accent_cyan,
 _ => t.accent_purple,
 };
 ui.label(egui::RichText::new(&entry.category).size(9.0).color(cat_color));
 ui.add_space(4.0);
 ui.label(egui::RichText::new(&entry.original).size(9.0).color(t.accent_red).monospace());
 ui.add_space(4.0);
 ui.label(egui::RichText::new(&entry.masked).size(9.0).color(t.accent_green).monospace());
 });
 ui.separator();
 }
 });
 });
 }
 }

 // ==================== YORDAM ====================
 fn page_yordam(&self, ui: &mut egui::Ui, t: &Theme) {
 egui::ScrollArea::vertical().show(ui, |ui| {
 ui.label(egui::RichText::new(" Yordam").size(18.0).strong().color(t.text_main));
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
 card_frame(t.border, t.bg_card).show(ui, |ui| {
 ui.set_width(ui.available_width());
 ui.label(egui::RichText::new(question).size(12.0).strong().color(t.accent_blue));
 ui.add_space(3.0);
 ui.label(egui::RichText::new(answer).size(10.0).color(t.text_dim));
 });
 ui.add_space(5.0);
 }
 }); // end ScrollArea
 }

 // ==================== HAQIDA ====================
 fn page_haqida(&self, ui: &mut egui::Ui, t: &Theme) {
 ui.label(egui::RichText::new(" Haqida").size(18.0).strong().color(t.text_main));
 ui.add_space(10.0);

 card_frame(t.accent_purple, t.bg_card).show(ui, |ui| {
 ui.set_width(ui.available_width());
 ui.vertical_centered(|ui| {
 ui.add_space(10.0);
 ui.label(egui::RichText::new("").size(40.0));
 ui.add_space(6.0);
 ui.label(egui::RichText::new("PrivacyProxy").size(20.0).strong().color(t.text_main));
 ui.label(egui::RichText::new("v1.1.2").size(12.0).color(t.text_dim));
 ui.add_space(10.0);
 ui.label(egui::RichText::new(self.lang.tr("Shaxsiy ma'lumotlarni\nAI xizmatlaridan himoya\nqilish uchun lokal proksi")).size(11.0).color(t.text_dim));
 ui.add_space(10.0);
 });
 });
 }
}

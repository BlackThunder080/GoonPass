use crate::State;
use eframe::egui;

impl State {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("GoonPass");
            ui.separator();

            self.new_password(ui);

            if !self.passwords.is_empty() {
                ui.separator();
                self.saved_passwords(ui);
            }
        });
    }

    fn new_password(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(egui::Color32::from_gray(16))
            .corner_radius(12)
            .inner_margin(8)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                egui::TextEdit::singleline(&mut self.name_field)
                    .hint_text("Name...")
                    .desired_width(f32::INFINITY)
                    .margin(8)
                    .show(ui);

                egui::TextEdit::singleline(&mut self.account_field)
                    .hint_text("Account...")
                    .desired_width(f32::INFINITY)
                    .margin(8)
                    .show(ui);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.set_width(ui.available_width());

                    let icon = egui::Image::new(egui::include_image!("../assets/add.svg"))
                        .fit_to_exact_size(egui::Vec2::new(14.0, 14.0));
                    let button = egui::Button::new(icon)
                        .corner_radius(15)
                        .fill(egui::Color32::from_gray(10))
                        .sense(egui::Sense::CLICK);
                    if ui.add(button).clicked() {
                        self.add_password();
                    }

                    egui::TextEdit::singleline(&mut self.plaintext_field)
                        .password(true)
                        .hint_text("Enter Password...")
                        .desired_width(f32::INFINITY)
                        .margin(8)
                        .show(ui);
                });
            });
    }

    fn saved_passwords(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(egui::Color32::from_gray(16))
            .corner_radius(12)
            .inner_margin(8)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, password) in self.passwords.clone().iter().enumerate() {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            let icon =
                                egui::Image::new(egui::include_image!("../assets/trash.svg"))
                                    .fit_to_exact_size(egui::Vec2::new(14.0, 14.0));
                            let button = egui::Button::new(icon)
                                .corner_radius(15)
                                .fill(egui::Color32::from_gray(10))
                                .sense(egui::Sense::CLICK);
                            if ui.add(button).clicked() {
                                self.remove_password(i, &password.name);
                            }

                            let icon =
                                egui::Image::new(egui::include_image!("../assets/clipboard.svg"))
                                    .fit_to_exact_size(egui::Vec2::new(14.0, 14.0));
                            let button = egui::Button::new(icon)
                                .corner_radius(15)
                                .fill(egui::Color32::from_gray(10))
                                .sense(egui::Sense::CLICK);
                            if ui.add(button).clicked() {
                                self.copy_password(password, ui.ctx());
                            }

                            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                                ui.label(egui::RichText::new(&password.name).strong());
                                ui.label(&password.account);
                            });
                        });
                    }
                });
            });
    }
}

pub fn login(master: &mut String, ui: &mut egui::Ui) -> Option<String> {
    let mut login = None;

    ui.vertical_centered(|ui| {
        ui.set_width(ui.available_width() - 8.0);

        ui.heading("GoonPass");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            if ui.button("Log In").clicked() && !master.is_empty() {
                login = Some(master.clone());
            }

            egui::TextEdit::singleline(master)
                .password(true)
                .hint_text("Master Password...")
                .margin(8)
                .desired_width(f32::INFINITY)
                .show(ui);
        });
    });

    login
}

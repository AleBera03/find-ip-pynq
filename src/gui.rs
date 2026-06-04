use eframe::egui;

#[derive(Default)]
pub struct MyEguiApp {
    result_ip: String
}

impl MyEguiApp {
    pub fn new(cc: &eframe::CreationContext<'_>, result_ip: String) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_global_style.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let mut app =  Self::default();
        app.result_ip = result_ip;
        app
    }
}

impl eframe::App for MyEguiApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0); // Margine superiore
                ui.heading("RESULT REMOTE IP");
                ui.add_space(10.0); // Spazio tra elementi
                ui.label(&self.result_ip);
                ui.add_space(15.0);
                if ui.button("copy to clipboard").clicked() {
                    ui.ctx().copy_text(self.result_ip.clone());
                }
            });
       });
   }
}
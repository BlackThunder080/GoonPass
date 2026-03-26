#![windows_subsystem = "windows"]

mod aes;
mod algo;
mod sha;
mod sqlite;
mod ui;

use eframe::egui;

fn main() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        default_hook(info);

        if let Some(payload) = info.payload_as_str() {
            _ = native_dialog::DialogBuilder::message()
                .set_title("Error")
                .set_level(native_dialog::MessageLevel::Error)
                .set_text(payload)
                .alert()
                .show();
        }
    }));

    eframe::run_native(
        "GoonPass",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
    .unwrap();
}

#[derive(Clone)]
struct Password {
    pub name: String,
    pub account: String,
    pub cyphertext: Vec<u8>,
}

struct State {
    // list of passwords
    passwords: Vec<Password>,
    // master password hash
    master: [u8; 32],
    // database connection
    connection: sqlite::Connection,
    // contents of the text fields
    name_field: String,
    account_field: String,
    plaintext_field: String,
}

impl State {
    pub fn new(master: &str) -> Self {
        let connection = sqlite::Connection::open("db").expect("failed to open database");
        connection
            .execute("CREATE TABLE IF NOT EXISTS passwords (name STRING PRIMARY KEY, account STRING, cyphertext BLOB)")
            .expect("corrupt database");
        connection
            .execute("CREATE TABLE IF NOT EXISTS master (hash BLOB, salt BLOB)")
            .expect("corrupt database");

        let master = if let sqlite::Step::Row(row) = connection
            .prepare("SELECT hash, salt FROM master")
            .expect("corrupt database")
            .step()
            .expect("corrupt database")
        {
            let hash = row.column_blob(0).expect("corrupt database");
            let salt = row.column_blob(1).expect("corrupt database");

            let mut master = master.as_bytes().to_vec();
            master.extend_from_slice(salt);

            if sha::hash(&master) != row.column_blob(0).expect("corrupt database") {
                todo!("wrong master password");
            }

            hash.try_into().expect("corrupt database")
        } else {
            let salt = rand::random::<[u8; 4]>();

            let mut master = master.as_bytes().to_vec();
            master.extend_from_slice(&salt);

            let hash = sha::hash(&master);

            let statement = connection
                .prepare("INSERT INTO master (hash, salt) VALUES (?, ?)")
                .expect("corrupt database");
            statement.bind_blob(1, &hash).expect("corrupt database");
            statement.bind_blob(2, &salt).expect("corrupt database");
            statement.execute().expect("corrupt database");

            hash
        };

        let mut passwords = Vec::new();
        let statement = connection
            .prepare("SELECT name, account, cyphertext FROM passwords")
            .expect("corrupt database");
        for row in &statement.rows() {
            let name = row.column_text(0).expect("corrupt database").to_string();
            let account = row.column_text(1).expect("corrupt database").to_string();
            let cyphertext = row.column_blob(2).expect("corrupt database").to_vec();

            passwords.push(Password {
                name,
                account,
                cyphertext,
            });
        }

        algo::sort(&mut passwords);

        Self {
            passwords,
            master,
            connection,
            name_field: String::new(),
            account_field: String::new(),
            plaintext_field: String::new(),
        }
    }

    fn add_password(&mut self) {
        let name = self.name_field.clone();
        let account = self.account_field.clone();
        let plaintext = self.plaintext_field.clone();

        // validation
        // - `name` must be between 1 and 30 characters
        // - `account` must be between 1 and 40 characters
        // - `plaintext` must be between 1 and 40 characters
        if name.is_empty()
            || account.is_empty()
            || plaintext.is_empty()
            || name.len() > 30
            || account.len() > 40
            || plaintext.len() > 40
        {
            return;
        }

        // return if a password with the same name already exists
        if algo::contains(&name, &self.passwords) {
            return;
        }

        let cyphertext = aes::encrypt(plaintext.as_bytes(), &self.master);

        let statement = self
            .connection
            .prepare("INSERT INTO passwords (name, account, cyphertext) VALUES (?, ?, ?)")
            .expect("corrupt database");
        statement.bind_text(1, &name).expect("corrupt database");
        statement.bind_text(2, &account).expect("corrupt database");
        statement
            .bind_blob(3, &cyphertext)
            .expect("corrupt database");
        statement.execute().expect("corrupt database");

        self.passwords.push(Password {
            name,
            account,
            cyphertext,
        });

        algo::sort(&mut self.passwords);

        self.name_field.clear();
        self.account_field.clear();
        self.plaintext_field.clear();
    }

    fn remove_password(&mut self, index: usize, name: &str) {
        self.passwords.remove(index);

        let statement = self
            .connection
            .prepare("DELETE FROM passwords WHERE name = ?")
            .expect("database");
        statement.bind_text(1, &name).expect("database");
        statement.execute().expect("database");
    }

    fn copy_password(&self, password: &Password, ctx: &egui::Context) {
        let bytes = aes::decrypt(&password.cyphertext, &self.master);
        let plaintext = String::from_utf8_lossy(&bytes);
        ctx.copy_text(plaintext.into_owned());
    }
}

enum App {
    LoggedIn(State),
    LoggedOut(String),
}

impl App {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        let ctx = &cc.egui_ctx;

        egui_extras::install_image_loaders(ctx);

        ctx.set_theme(egui::Theme::Dark);
        ctx.set_zoom_factor(2.0);

        ctx.style_mut(|style| {
            style.spacing.item_spacing = egui::Vec2::splat(8.0);
            style.spacing.button_padding = egui::Vec2::splat(8.0);
            style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(12);
            style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(12);
            style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(12);
        });

        Self::LoggedOut(String::new())
    }

    fn login(&mut self, master: &str) {
        *self = App::LoggedIn(State::new(master));
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| match self {
            Self::LoggedIn(state) => state.ui(ui),
            Self::LoggedOut(master) => {
                if let Some(master) = ui::login(master, ui) {
                    // validation
                    // `master` must be between 1 and 40 characters
                    if master.is_empty() || master.len() > 40 {
                        return;
                    }

                    self.login(&master);
                }
            }
        });
    }
}

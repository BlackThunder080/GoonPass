mod aes;
mod algo;
mod sha;
mod sqlite;
mod ui;

use eframe::egui;

fn main() {
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
        let master = sha::hash(master.as_bytes());

        // open database
        let connection = sqlite::Connection::open("db").unwrap();
        connection
            .execute("CREATE TABLE IF NOT EXISTS passwords (name STRING PRIMARY KEY, account STRING, cyphertext BLOB)")
            .unwrap();
        connection
            .execute("CREATE TABLE IF NOT EXISTS master (hash BLOB)")
            .unwrap();

        // compare the given master password to the password in the database
        let statement = connection.prepare("SELECT hash FROM master").unwrap();
        if let sqlite::Step::Row(row) = statement.step().unwrap() {
            if master != row.column_blob(0).unwrap() {
                todo!("wrong master password");
            }
        } else {
            let statement = connection
                .prepare("INSERT INTO master (hash) VALUES (?)")
                .unwrap();
            statement.bind_blob(1, &master).unwrap();
            statement.execute().unwrap();
        }

        drop(statement);

        // create list of passwords from database
        let mut passwords = Vec::new();
        let statement = connection
            .prepare("SELECT name, account, cyphertext FROM passwords")
            .unwrap();
        for row in &statement.rows() {
            let name = row.column_text(0).unwrap().to_string();
            let account = row.column_text(1).unwrap().to_string();
            let cyphertext = row.column_blob(2).unwrap().to_vec();

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
            .unwrap();
        statement.bind_text(1, &name).unwrap();
        statement.bind_text(2, &account).unwrap();
        statement.bind_blob(3, &cyphertext).unwrap();
        statement.execute().unwrap();

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
            .unwrap();
        statement.bind_text(1, &name).unwrap();
        statement.execute().unwrap();
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
        // if logged in render state, otherwise display login ui
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

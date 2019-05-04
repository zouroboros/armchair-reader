extern crate gtk;

use std::env;

use gtk::prelude::*;
use gtk::*;
use gio;
use gio::prelude::*;

#[derive(Debug, Clone)]
pub struct Ui {
    application: gtk::Application,
    main_view: gtk::ApplicationWindow
}

impl Ui {
    pub fn new() {
        let application = gtk::Application::new("me.murks.armchairreader", gio::ApplicationFlags::empty())
            .expect("Application initialization failed...");
        
        application.connect_activate(|app| {
            let main_view = gtk::ApplicationWindow::new(app);
            main_view.set_title("Armchair Reader");
            
            let builder = gtk::Builder::new_from_string(include_str!("views/headerbar.ui"));
            let header: gtk::HeaderBar = builder.get_object("headerbar").unwrap();
            main_view.set_titlebar(&header);
            
            main_view.maximize();
            main_view.present();
            
        });
        
        let args: Vec<String> = env::args().collect();
        ApplicationExtManual::run(&application, &args);
    }
}

use std::env;
extern crate chrono;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate glib;
extern crate gtk;
extern crate timer;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

mod configuration;
mod database;

enum Message {
    UpdateImg(String),
}

fn slider(config: &configuration::Configuration) {
    let database = database::Database::new(&config.database_file);
    let timer = timer::Timer::new();

    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    let glade_src = include_str!("slider.glade");
    let builder = gtk::Builder::new_from_string(glade_src);

    let slider_window: gtk::Window = builder.get_object("slider_window").unwrap();
    let slider_img: gtk::Image = builder.get_object("slider_img").unwrap();

    slider_window
        .override_background_color(slider_window.get_state_flags(), Some(&(gdk::RGBA::black())));

    let mon = gdk::Screen::get_default().unwrap();
    let monitor_width = mon.get_width();
    let monitor_height = mon.get_height();

    let picture_path = format!(
        "{}/{}",
        config.picture_folder.as_ref().unwrap().clone(),
        database.get_one().unwrap().path
    );
    let _ = sender.send(Message::UpdateImg(picture_path));

    let _guard = {
        let picture_path_clone = config.picture_folder.as_ref().unwrap().clone();
        timer.schedule_repeating(chrono::Duration::seconds(5), move || {
            let picture_path = format!(
                "{}/{}",
                picture_path_clone,
                database.get_one().unwrap().path
            );
            let _ = sender.send(Message::UpdateImg(picture_path));
        })
    };

    let slider_img_clone = slider_img;
    receiver.attach(None, move |msg| {
        match msg {
            Message::UpdateImg(picture_path) => {
                let img = gdk_pixbuf::Pixbuf::new_from_file_at_scale(
                    picture_path,
                    monitor_width,
                    monitor_height,
                    true,
                )
                .unwrap();
                slider_img_clone.set_from_pixbuf(Some(img.as_ref()));
            }
        }
        glib::Continue(true)
    });

    slider_window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    slider_window.fullscreen();
    slider_window.show_all();

    gtk::main();
}

fn move_files() {
    println!("Move")
}

fn update_database() {
    println!("Update")
}

fn database_stats(config: &configuration::Configuration) {
    let database = database::Database::new(&config.database_file);
    let stats = database.stats().unwrap();
    let nv = stats.not_viewed as f32;
    let a = stats.all as f32;
    println!(
        "{} pictures\n{} pictures viewed\n{} pictures not viewed\n{:.2}% not viewed",
        stats.all,
        stats.viewed,
        stats.not_viewed,
        nv / a * 100.0
    );
}

fn program_settings(config: &configuration::Configuration) {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }
    let glade_src = include_str!("settings.glade");
    let builder = gtk::Builder::new_from_string(glade_src);

    let settings_window: gtk::Window = builder.get_object("settings_window").unwrap();
    let database_button: gtk::Button = builder.get_object("database_button").unwrap();
    let pictures_button: gtk::Button = builder.get_object("pictures_button").unwrap();
    let save_button: gtk::Button = builder.get_object("save_button").unwrap();
    let database_entry: gtk::Entry = builder.get_object("database_entry").unwrap();
    let pictures_entry: gtk::Entry = builder.get_object("pictures_entry").unwrap();

    database_entry.set_buffer(&gtk::EntryBuffer::new(config.database_file.as_deref()));
    pictures_entry.set_buffer(&gtk::EntryBuffer::new(config.picture_folder.as_deref()));

    let settings_window_clone1 = settings_window.clone();
    database_button.connect_clicked(move |_| {
        let database_chooser = gtk::FileChooserDialog::new(
            Some("Select database"),
            Some(&settings_window_clone1),
            gtk::FileChooserAction::Open,
        );
        database_chooser.add_buttons(&[
            ("Open", gtk::ResponseType::Ok),
            ("Cancel", gtk::ResponseType::Cancel),
        ]);
        if database_chooser.run() == gtk::ResponseType::Ok {
            let filename = database_chooser
                .get_filename()
                .expect("Couldn't get filename");
            let buffer = gtk::EntryBuffer::new(filename.as_path().to_str());
            database_entry.set_buffer(&buffer);
        }
        database_chooser.destroy();
    });

    let settings_window_clone2 = settings_window.clone();
    pictures_button.connect_clicked(move |_| {
        let picture_chooser = gtk::FileChooserDialog::new(
            Some("Select picture folder"),
            Some(&settings_window_clone2),
            gtk::FileChooserAction::SelectFolder,
        );
        picture_chooser.add_buttons(&[
            ("Open", gtk::ResponseType::Ok),
            ("Cancel", gtk::ResponseType::Cancel),
        ]);
        if picture_chooser.run() == gtk::ResponseType::Ok {
            let filename = picture_chooser
                .get_filename()
                .expect("Couldn't get filename");
            let buffer = gtk::EntryBuffer::new(filename.as_path().to_str());
            pictures_entry.set_buffer(&buffer);
        }
        picture_chooser.destroy();
    });

    let config_clone: Rc<RefCell<configuration::Configuration>> =
        Rc::new(RefCell::new(configuration::Configuration::new()));
    let database_entry_clone: Rc<RefCell<gtk::Entry>> =
        Rc::new(RefCell::new(builder.get_object("database_entry").unwrap()));
    let pictures_entry_clone: Rc<RefCell<gtk::Entry>> =
        Rc::new(RefCell::new(builder.get_object("pictures_entry").unwrap()));
    let settings_window_clone3 = settings_window.clone();
    save_button.connect_clicked(move |_| {
        config_clone.borrow_mut().database_file =
            Some(database_entry_clone.borrow_mut().get_buffer().get_text());
        config_clone.borrow_mut().picture_folder =
            Some(pictures_entry_clone.borrow_mut().get_buffer().get_text());
        config_clone.borrow_mut().save();
        settings_window_clone3.close();
    });

    settings_window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    settings_window.show_all();

    gtk::main();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut config = configuration::Configuration::new();
    config.read();
    if config.database_file == None || config.picture_folder == None {
        program_settings(&config);
    }
    if args.len() == 1 {
        slider(&config);
    } else if args.len() == 2 {
        match args[1].as_str() {
            "update" => update_database(),
            "move" => move_files(),
            "stats" => database_stats(&config),
            "settings" => program_settings(&config),
            _ => println!("Unsupported argument"),
        }
    } else {
        println!("Unsupported argument length {}", args.len());
    }
}

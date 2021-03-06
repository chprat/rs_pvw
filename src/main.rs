use std::env;
extern crate chrono;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate glib;
extern crate gtk;
extern crate timer;
use gtk::prelude::*;
use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;
use walkdir::WalkDir;

mod configuration;
mod database;

enum Message {
    UpdateImg((u32, String)),
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
    let builder = gtk::Builder::from_string(glade_src);

    let slider_window: gtk::Window = builder.get_object("slider_window").unwrap();
    let slider_img: gtk::Image = builder.get_object("slider_img").unwrap();

    slider_window
        .override_background_color(slider_window.get_state_flags(), Some(&(gdk::RGBA::black())));

    let mon = gdk::Screen::get_default().unwrap();
    let monitors = mon.get_n_monitors();
    let mut monitor_width = 0;
    let mut monitor_height = 0;
    let mut monitor_pixels = 0;
    for i in 0..monitors {
        let monitor = mon.get_monitor_geometry(i);
        let pixels = monitor.width * monitor.height;
        if pixels > monitor_pixels {
            monitor_pixels = pixels;
            monitor_height = monitor.height;
            monitor_width = monitor.width;
        }
    }
    let entry = database.get_one().unwrap();
    let picture_path = format!(
        "{}/{}",
        config.picture_folder.as_ref().unwrap().clone(),
        entry.path
    );
    let _ = database.increment(&entry);
    let _ = sender.send(Message::UpdateImg((entry.id, picture_path)));

    let _guard = {
        let picture_path_clone = config.picture_folder.as_ref().unwrap().clone();
        timer.schedule_repeating(chrono::Duration::seconds(5), move || {
            let entry = database.get_one().unwrap();
            let picture_path = format!("{}/{}", picture_path_clone, entry.path);
            let _ = database.increment(&entry);
            let _ = sender.send(Message::UpdateImg((entry.id, picture_path)));
        })
    };

    let slider_img_clone = slider_img.clone();
    let slider_window_clone = slider_window.clone();
    receiver.attach(None, move |msg| {
        match msg {
            Message::UpdateImg((id, picture_path)) => {
                let img = gdk_pixbuf::Pixbuf::from_file_at_scale(
                    picture_path,
                    monitor_width,
                    monitor_height,
                    true,
                )
                .unwrap();
                slider_img_clone.set_from_pixbuf(Some(img.as_ref()));
                // TODO: abusing the title to transfer data is probably not the best idea
                slider_window_clone.set_title(&id.to_string()[..]);
            }
        }
        glib::Continue(true)
    });

    slider_window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    // TODO: using a second database is probably not the best idea
    // probably we should at least throw in some mutex?
    let database2 = database::Database::new(&config.database_file);
    slider_window.connect_key_press_event(move |window, gdk| {
        match gdk.get_keyval() {
            gdk::keys::constants::space => database2
                .mark(
                    &database2
                        .get_entry_by_id(
                            window.get_title().unwrap().as_str().parse::<u32>().unwrap(),
                        )
                        .unwrap(),
                )
                .unwrap(),
            // TODO: implement moving during slideshow
            gdk::keys::constants::Left => println!("left"),
            gdk::keys::constants::Right => println!("right"),
            _ => (),
        };
        Inhibit(false)
    });

    slider_window.fullscreen();
    slider_window.show_all();

    gtk::main();
}

fn move_files(config: &configuration::Configuration) {
    let database = database::Database::new(&config.database_file);
    let entries = database.get_marked().unwrap();
    for entry in &entries {
        let base_path = Path::new(config.picture_folder.as_ref().unwrap())
            .parent()
            .unwrap();
        let old_filename =
            Path::new(config.picture_folder.as_ref().unwrap()).join(entry.path.clone());
        let del_base_path = base_path.join("delete");
        let new_filename = del_base_path.join(entry.path.clone());
        let new_filefolder = new_filename.parent().unwrap();
        if !new_filefolder.exists() {
            let _ = fs::create_dir_all(new_filefolder);
        }
        let _ = database.remove(entry);
        let _ = fs::rename(old_filename, new_filename);
    }
}

fn update_database(config: &configuration::Configuration) {
    let database = database::Database::new(&config.database_file);
    let mut files: Vec<PathBuf> = Vec::new();
    let mut entries: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(config.picture_folder.as_ref().unwrap()) {
        let entry = entry.unwrap();
        if !entry.path().is_dir() {
            files.push(entry.path().to_path_buf());
        }
    }
    for entry in database.get_all().unwrap() {
        entries.push(Path::new(config.picture_folder.as_ref().unwrap()).join(entry.path.clone()));
    }
    for entry in &entries {
        if !files.contains(&entry) {
            let path = entry
                .strip_prefix(PathBuf::from(config.picture_folder.as_ref().unwrap()))
                .unwrap();
            let old = database.get_entry_by_path(path.to_str().unwrap()).unwrap();
            let _ = database.remove(&old);
        }
    }
    for entry in &files {
        if !entries.contains(&entry) {
            let new = entry
                .strip_prefix(PathBuf::from(config.picture_folder.as_ref().unwrap()))
                .unwrap();
            let _ = database.add(new.to_str().unwrap());
        }
    }
}

fn database_stats(config: &configuration::Configuration) {
    let database = database::Database::new(&config.database_file);
    let stats = database.stats();
    let mut all = 0;
    for entry in stats {
        if entry.1 > 0 {
            println!("{}: {}", entry.0, entry.1);
        }
        all += entry.1;
    }
    println!("{} entries overall", all);
}

fn database_init(config: &configuration::Configuration) {
    let database = database::Database::new(&config.database_file);
    let e = database.init();
    if e.is_ok() {
        update_database(config);
        database_stats(config);
    } else {
        println!("database already initialized");
    }
}

fn program_settings(config: &configuration::Configuration) {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }
    let glade_src = include_str!("settings.glade");
    let builder = gtk::Builder::from_string(glade_src);

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
        unsafe {
            database_chooser.destroy();
        }
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
        unsafe {
            picture_chooser.destroy();
        }
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
            "update" => update_database(&config),
            "move" => move_files(&config),
            "stats" => database_stats(&config),
            "settings" => program_settings(&config),
            "init" => database_init(&config),
            _ => println!("Unsupported argument"),
        }
    } else {
        println!("Unsupported argument length {}", args.len());
    }
}

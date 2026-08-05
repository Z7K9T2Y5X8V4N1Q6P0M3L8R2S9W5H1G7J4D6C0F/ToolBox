#![windows_subsystem = "windows"]

use winsafe::{self as w, gui, prelude::*};

fn main() {
    if let Err(e) = MyWindow::create_and_run() {
        eprintln!("{}", e);
    }
}

#[derive(Clone)]
pub struct MyWindow {
    wnd: gui::WindowMain,
    btn_hello: gui::Button,
}

impl MyWindow {
    pub fn create_and_run() -> w::AnyResult<i32> {
        let wnd = gui::WindowMain::new(gui::WindowMainOpts {
            title: "My window title",
            size: gui::dpi(300, 150),
            ..Default::default()
        });

        let btn_hello = gui::Button::new(
            &wnd,
            gui::ButtonOpts {
                text: "&Click me",
                position: gui::dpi(20, 20),
                ..Default::default()
            },
        );

        let new_self = Self { wnd, btn_hello };
        new_self.events();

        new_self.wnd.run_main(None)
    }

    fn events(&self) {
        let wnd = self.wnd.clone();
        self.btn_hello.on().bn_clicked(move || {
            wnd.hwnd().SetWindowText("Hello, world!")?;
            Ok(())
        });
    }
}

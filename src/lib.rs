#![feature(proc_macro_hygiene)]

use skyline::install_hook;
use skyline_web::{Webpage, WebSession, Visibility};
use smash::lib::lua_const::*;
use smash::app::{self, lua_bind::*};
use nnsdk::web::offlinewebsession::OfflineWebSession;

static SESSION: WebSession = start_session();

fn start_session() -> WebSession {
    let render = r#"<!doctype html>
    <html>
        <head>
            <title>Web Session Test</title>
        </head>
        <body>
            <p>This is a test of the skyline-web session functionality</p>
            <div>
                <p>Send message: </p> <input type="text" id="send_content"
                    name="send_content">
                <button type="button" , onclick="send_message();">SEND</button>
            </div>
            <div>
                <p>Receive message: </p><div id="receive_content"></div>
            </div>
        </body>
        <script>
            function myFunction() {
                var content = document.getElementById("send_content").textContent;
                window.nx.send(content);
            }
    
            window.nx.addEventListener("message", function (e) {
            document.getElementById("receive_content").innerHTML = e.data;
            }
        </script>
    </html>"#;
    let session = Webpage::new()
        .htdocs_dir("contents")
        .file("index.html", &render)
        .open_session(Visibility::InitiallyHidden).unwrap();
    session
}

pub unsafe fn menu_condition(module_accessor: &mut smash::app::BattleObjectModuleAccessor) -> bool {
    ControlModule::check_button_on(module_accessor, *CONTROL_PAD_BUTTON_SPECIAL)
        && ControlModule::check_button_on_trriger(
            module_accessor,
            *CONTROL_PAD_BUTTON_APPEAL_HI,
        )
}

unsafe fn once_per_frame_per_fighter(
    module_accessor: &mut app::BattleObjectModuleAccessor,
    category: i32,
) {
    if category != FIGHTER_PAD_COMMAND_CATEGORY1 {
        return;
    }

    if menu_condition(module_accessor) {
        loop {
            if let Some(msg) = SESSION.try_recv() {
                dbg!(msg);

                SESSION.show();
                SESSION.send("test");
                SESSION.wait_for_exit();
                break;
            }
        }
    }
}

#[skyline::hook(replace = ControlModule::get_command_flag_cat)]
pub unsafe fn handle_get_command_flag_cat(
    module_accessor: &mut app::BattleObjectModuleAccessor,
    category: i32,
) -> i32 {
    let flag = original!()(module_accessor, category);
    once_per_frame_per_fighter(module_accessor, category);

    flag
}

#[skyline::main(name = "web-sessions")]
pub fn main() {
    println!("[web-sessions] Initialized");
    install_hook!(handle_get_command_flag_cat);
}

#![feature(proc_macro_hygiene)]
#![feature(const_panic)]
#![feature(const_mut_refs)]

use skyline::install_hook;
use skyline_web::{Visibility, Webpage};
use smash::app::{self, lua_bind::*};
use smash::lib::lua_const::*;

static mut SHOW_SESSION: bool = false;

fn log(msg: &str) {
    let plugin_name = "web-session";
    println!("[{}] {}", plugin_name, msg);
}

fn start_session() {
    std::thread::spawn(|| {
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
                function send_message() {
                    var content = document.getElementById("send_content").textContent;
                    window.nx.send(content);
                }
                function goBack() {
                    location.href = "http://localhost/";
                    window.history.back();
                }
                function closeSession() {
                    window.nx.endApplet();
                }
        
                window.nx.addEventListener("message", function (e) {
                document.getElementById("receive_content").innerHTML = e.data;
                });
                nx.footer.setAssign("B", "Go Back", goBack);
            </script>
        </html>"#;
        let session = Webpage::new()
            .htdocs_dir("contents")
            .file("index.html", &render)
            .open_session(Visibility::InitiallyHidden)
            .unwrap();
        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
            unsafe {
                if SHOW_SESSION {
                    SHOW_SESSION = false;
                    session.send("Hello from Rust!");
                    log("Sent message \"Hello from Rust!\"");

                    log("Showing session");
                    session.show();

                    // session.send("test two");
                    // log("Sent message \"test two\"");
                    
                    // log("Waiting for exit");
                    // session.wait_for_exit();
                    // log("Exited");
                }
                if let Some(msg) = session.try_recv_max(0x10000000) {
                    log(&format!("Received message from JS: {}", msg));
                }
            }
        }
    });
}

pub unsafe fn menu_condition(module_accessor: &mut smash::app::BattleObjectModuleAccessor) -> bool {
    ControlModule::check_button_on(module_accessor, *CONTROL_PAD_BUTTON_SPECIAL)
        && ControlModule::check_button_on_trriger(module_accessor, *CONTROL_PAD_BUTTON_APPEAL_HI)
}

unsafe fn once_per_frame_per_fighter(
    module_accessor: &mut app::BattleObjectModuleAccessor,
    category: i32,
) {
    if category != FIGHTER_PAD_COMMAND_CATEGORY1 {
        return;
    }

    if menu_condition(module_accessor) {
        log("Menu condition triggered");
        log(&format!("SHOW_SESSION: {}", SHOW_SESSION));
        SHOW_SESSION = true;
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
    log("Initialized");
    install_hook!(handle_get_command_flag_cat);
    start_session();
}

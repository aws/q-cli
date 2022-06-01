use super::constants::CONSTANTS_SCRIPT;

pub fn javascript_init() -> String {
    let mut init = String::new();
    init.push_str(
        r#"
console.log("[fig] declaring constants...")

if (!window.fig) {
    window.fig = {}
}

if (!window.fig.constants) {
    window.fig.constants = {}
}
"#,
    );
    init.push_str(&CONSTANTS_SCRIPT);
    init
}

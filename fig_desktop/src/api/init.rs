use super::constants::Constants;

pub fn javascript_init() -> String {
    vec![
        "if (!window.fig) window.fig = {};".into(),
        "if (!window.fig.constants) fig.constants = {};".into(),
        Constants::default().script(),
        r#"console.log("[fig] declaring constants...");"#.into(),
    ]
    .join("\n")
}

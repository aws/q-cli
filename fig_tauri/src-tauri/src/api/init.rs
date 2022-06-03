use super::constants::Constants;

pub fn javascript_init() -> String {
    vec![
        "if (!window.fig) {\n\
            window.fig = {}\n\
        }\n\
        if (!window.fig.constants) {\n\
            fig.constants = {}\n\
        }\n"
        .into(),
        Constants::default().script(),
        r#"console.log("[fig] declaring constants...");"#.into(),
    ]
    .join("\n")
}

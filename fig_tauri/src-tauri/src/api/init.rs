use std::borrow::Cow;

use super::constants::Constants;

pub fn javascript_init() -> String {
    let mut init = Vec::<Cow<'static, str>>::new();
    init.push(
        "if (!window.fig) {\n\
            window.fig = {}\n\
        }\n\
        if (!window.fig.constants) {\n\
            fig.constants = {}\n\
        }\n"
        .into(),
    );
    init.push(Constants::default().script().into());
    init.push(r#"console.log("[fig] declaring constants...");"#.into());
    init.join("\n")
}

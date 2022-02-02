#[derive(Debug, Clone, Default)]
pub struct LaunchdPlist {
    label: String,
    program: Option<String>,
    program_arguments: Option<Vec<String>>,
    keep_alive: Option<bool>,
}

impl LaunchdPlist {
    pub fn new(label: impl Into<String>) -> LaunchdPlist {
        LaunchdPlist {
            label: label.into(),
            ..LaunchdPlist::default()
        }
    }

    pub fn plist(&self) -> String {
        let mut plist = String::new();

        let indent = "    ";
        let mut indent_level = 0;

        macro_rules! indent_block {
            ($block:block) => {{
                {
                    indent_level += 1;

                    $block;

                    indent_level -= 1;
                }
            }};
        }

        macro_rules! push_line {
            ($line:expr) => {{
                for _ in 0..indent_level {
                    plist.push_str(indent);
                }
                plist.push_str($line);
                plist.push('\n');
            }};
        }

        macro_rules! push_key_val {
            ($key:expr, String, $val:expr) => {{
                push_line!(&format!("<key>{}</key>", $key));
                push_line!(&format!("<string>{}</string>", $val));
            }};
            ($key:expr, &[String], $val:expr) => {{
                push_line!(&format!("<key>{}</key>", $key));
                push_line!("<array>");
                indent_block!({
                    for s in $val.iter() {
                        push_line!(&format!("<string>{}</string>", s));
                    }
                });
                push_line!("</array>");
            }};
            ($key:expr, bool, $val:expr) => {{
                push_line!(&format!("<key>{}</key>", $key));
                push_line!(if $val { "<true/>" } else { "<false/>" });
            }};
        }

        push_line!(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        push_line!(
            r#"<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">"#
        );
        push_line!(r#"<plist version="1.0">"#);

        indent_block!({
            push_line!("<dict>");

            indent_block!({
                push_key_val!("Label", String, &self.label);

                if let Some(program) = &self.program {
                    push_key_val!("Program", String, &program);
                }
    
                if let Some(program_arguments) = &self.program_arguments {
                    push_key_val!("ProgramArguments", &[String], program_arguments);
                }
    
                if let Some(keep_alive) = &self.keep_alive {
                    push_key_val!("KeepAlive", bool, *keep_alive);
                }    
            });

            push_line!("</dict>");
        });

        push_line!("</plist>");

        plist
    }

    pub fn label(mut self, label: impl Into<String>) -> LaunchdPlist {
        self.label = label.into();
        self
    }

    pub fn program(mut self, program: impl Into<String>) -> LaunchdPlist {
        self.program = Some(program.into());
        self
    }

    pub fn program_arguments<I, T>(mut self, program_arguments: I) -> LaunchdPlist
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        self.program_arguments = Some(program_arguments.into_iter().map(|s| s.into()).collect());
        self
    }

    pub fn keep_alive(mut self, keep_alive: bool) -> LaunchdPlist {
        self.keep_alive = Some(keep_alive);
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_plist() {
        let plist = LaunchdPlist::new("io.fig.test")
            .program("hello")
            .program_arguments(["hello", "test"])
            .keep_alive(true)
            .plist();

let valid_plist = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
    <dict>
        <key>Label</key>
        <string>io.fig.test</string>
        <key>Program</key>
        <string>hello</string>
        <key>ProgramArguments</key>
        <array>
            <string>hello</string>
            <string>test</string>
        </array>
        <key>KeepAlive</key>
        <true/>
    </dict>
</plist>
"#;
    
        assert_eq!(plist, valid_plist);
    }
}

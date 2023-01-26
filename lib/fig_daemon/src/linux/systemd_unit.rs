use std::fmt::Write;

#[derive(Debug, Clone, Default)]
pub struct SystemdUnit {
    unit: UnitUnit,
    service: UnitService,
    install: UnitInstall,
}

impl SystemdUnit {
    pub fn new(description: impl Into<String>) -> SystemdUnit {
        SystemdUnit {
            unit: UnitUnit {
                description: description.into(),
            },
            ..SystemdUnit::default()
        }
    }

    pub fn unit(&self) -> String {
        let mut unit = String::new();

        unit.push_str("[Unit]\n");
        writeln!(unit, "Description={}", &self.unit.description).ok();

        unit.push('\n');

        unit.push_str("[Service]\n");

        if let Some(exec_start) = &self.service.exec_start {
            writeln!(unit, "ExecStart={exec_start}").ok();
        }

        if let Some(standard_output) = &self.service.standard_output {
            writeln!(unit, "StandardOutput={standard_output}").ok();
        }

        if let Some(standard_error) = &self.service.standard_error {
            writeln!(unit, "StandardError={standard_error}").ok();
        }

        if let Some(restart) = &self.service.restart {
            writeln!(unit, "Restart={restart}").ok();
        }

        if let Some(restart_sec) = &self.service.restart_sec {
            writeln!(unit, "RestartSec={restart_sec}").ok();
        }

        unit.push('\n');

        unit.push_str("[Install]\n");

        if let Some(wanted_by) = &self.install.wanted_by {
            writeln!(unit, "WantedBy={wanted_by}").ok();
        }

        unit
    }

    pub fn exec_start(mut self, exec_start: impl Into<String>) -> SystemdUnit {
        self.service.exec_start = Some(exec_start.into());
        self
    }

    #[allow(dead_code)]
    pub fn standard_output(mut self, standard_output: impl Into<String>) -> SystemdUnit {
        self.service.standard_output = Some(standard_output.into());
        self
    }

    #[allow(dead_code)]
    pub fn standard_error(mut self, standard_error: impl Into<String>) -> SystemdUnit {
        self.service.standard_error = Some(standard_error.into());
        self
    }

    pub fn restart(mut self, restart: impl Into<String>) -> SystemdUnit {
        self.service.restart = Some(restart.into());
        self
    }

    pub fn restart_sec(mut self, restart_sec: usize) -> SystemdUnit {
        self.service.restart_sec = Some(restart_sec);
        self
    }

    pub fn wanted_by(mut self, wanted_by: impl Into<String>) -> SystemdUnit {
        self.install.wanted_by = Some(wanted_by.into());
        self
    }
}

#[derive(Debug, Clone, Default)]
struct UnitUnit {
    description: String,
}

#[derive(Debug, Clone, Default)]
struct UnitService {
    exec_start: Option<String>,
    restart: Option<String>,
    restart_sec: Option<usize>,
    standard_output: Option<String>,
    standard_error: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct UnitInstall {
    wanted_by: Option<String>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn unit_test() {
        let unit = SystemdUnit::new("test")
            .exec_start("/usr/bin/local/exe hi")
            .restart("always")
            .restart_sec(5)
            .wanted_by("test.target")
            .unit();

        let unit_valid = "[Unit]
Description=test

[Service]
ExecStart=/usr/bin/local/exe hi
Restart=always
RestartSec=5

[Install]
WantedBy=test.target
";

        assert_eq!(unit, unit_valid);
    }
}

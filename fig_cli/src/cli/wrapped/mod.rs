use std::collections::HashMap;

use clap::Args;
use eyre::Result;
use fig_history::{
    CommandInfo,
    History,
};
use rand::Rng;
use time::{
    Date,
    Duration,
    OffsetDateTime,
    Time,
    UtcOffset,
    Weekday,
};
use tui::component::{
    Div,
    Layout,
    P,
};
use tui::{
    ColorAttribute,
    Component,
    ControlFlow,
    InputMethod,
    ParserOptions,
    State,
    StyleSheet,
    Surface,
};

struct Wrapped {
    pub top_commands: Vec<(String, usize)>,
    pub top_directories: Vec<(String, usize)>,
    pub weekly_activity: Vec<(String, f64)>,
    pub daily_activity: Vec<f64>,
    pub shortest_commit_message: Option<(String, String)>,
    pub most_errors_in_a_day: Option<(Date, usize)>,
    pub most_commands_in_a_day: Option<(Date, usize)>,
    pub longest_running_command: Option<(String, Duration)>,
}

impl Wrapped {
    fn new(history: Vec<CommandInfo>) -> Self {
        let current_local_offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
        let current_year = OffsetDateTime::now_utc().to_offset(current_local_offset).year();

        let mut commands_by_occurrence = HashMap::new();
        let mut occurrence_by_date = HashMap::new();
        let mut longest_running_command = None;
        let mut directories_by_occurrence = HashMap::new();
        let mut errors_by_day = HashMap::new();
        let mut shortest_commit_message = None;
        let mut weekly_activity = vec![0_f64; 7];
        let mut times = vec![];
        for row in history {
            if let Some(start_time) = row.start_time {
                let date = OffsetDateTime::from(start_time).to_offset(current_local_offset).date();

                // Ignore commands not from the current year
                if date.year() != current_year {
                    continue;
                }

                *occurrence_by_date.entry(date).or_insert_with(|| 0) += 1;
            }

            if let Some(command) = row.command {
                let command = match command.split_once(' ') {
                    Some((command, rest)) => {
                        if let (Some(start_time), Some(end_time)) = (row.start_time, row.end_time) {
                            let duration1 = OffsetDateTime::from(end_time) - OffsetDateTime::from(start_time);

                            match longest_running_command {
                                Some((_, duration2)) => {
                                    if duration1 > duration2 {
                                        longest_running_command = Some((command.to_owned(), duration1));
                                    }
                                },
                                None => longest_running_command = Some((command.to_owned(), duration1)),
                            }
                        }

                        if command == "git" {
                            if let Some(args) = shlex::split(rest) {
                                let message = args.iter().take_while(|arg| arg != &"-m" || arg != &"--message").next();
                                let date = row.start_time.map(|start_time| {
                                    let date_time = OffsetDateTime::from(start_time)
                                        .to_offset(UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC));

                                    let mut month = date_time.month().to_string();
                                    month.truncate(3);

                                    format!("{month} {}", date_time.day())
                                });

                                if let (Some(message), Some(date)) = (message, date) {
                                    shortest_commit_message = Some((message.to_owned(), date));
                                }
                            }
                        }

                        command.to_owned()
                    },
                    None => command.to_owned(),
                };

                match row.exit_code {
                    Some(0) => *commands_by_occurrence.entry(command.to_owned()).or_insert_with(|| 0) += 1,
                    Some(exit_code) => {
                        if let Some(end_time) = row.end_time {
                            *errors_by_day
                                .entry(
                                    OffsetDateTime::from(end_time)
                                        .to_offset(UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC))
                                        .date(),
                                )
                                .or_insert_with(|| 0) += 1;
                        }

                        if exit_code != 127 {
                            *commands_by_occurrence.entry(command.to_owned()).or_insert_with(|| 0) += 1;
                        }
                    },
                    None => *commands_by_occurrence.entry(command.to_owned()).or_insert_with(|| 0) += 1,
                }
            }

            if let Some(directory) = row.cwd {
                *directories_by_occurrence.entry(directory).or_insert_with(|| 0) += 1;
            }

            if let Some(start_time) = row.start_time {
                let date_time = OffsetDateTime::from(start_time)
                    .to_offset(UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC));
                weekly_activity[date_time.weekday().number_days_from_sunday() as usize] += 1.0;
                times.push(date_time.time());
            }
        }

        let mut top_commands: Vec<(String, usize)> = commands_by_occurrence.into_iter().collect();
        top_commands.sort_by(|(_, cnt), (_, cnt2)| cnt2.cmp(cnt));

        let mut top_directories: Vec<(String, usize)> = directories_by_occurrence.into_iter().collect();
        top_directories.sort_by(|(_, cnt), (_, cnt2)| cnt2.cmp(cnt));

        let weekly_max = weekly_activity
            .iter()
            .max_by(|x, y| x.total_cmp(y))
            .cloned()
            .unwrap_or(1.0);

        let weekly_activity: Vec<(String, f64)> = weekly_activity
            .into_iter()
            .zip(
                [
                    Weekday::Sunday,
                    Weekday::Monday,
                    Weekday::Tuesday,
                    Weekday::Wednesday,
                    Weekday::Thursday,
                    Weekday::Friday,
                    Weekday::Saturday,
                ]
                .iter(),
            )
            .map(|(cnt, weekday)| {
                let mut weekday = weekday.to_string();
                weekday.truncate(3);
                (weekday, cnt / weekly_max)
            })
            .collect();

        let intervals_len = 24;
        let mut intervals = vec![0_f64; intervals_len];
        let interval = Duration::seconds_f64((Duration::days(1).as_seconds_f64() - 1.0) / intervals_len as f64);

        while let Some(time) = times.pop() {
            let mut interval_time = Time::MIDNIGHT;
            for i in intervals.iter_mut() {
                match time >= interval_time && time < interval_time + interval {
                    true => {
                        *i += 1.0;
                        break;
                    },
                    false => interval_time += interval,
                }
            }
        }

        let interval_max = intervals.iter().max_by(|x, y| x.total_cmp(y)).cloned().unwrap_or(1.0);
        let daily_activity: Vec<f64> = intervals.into_iter().map(|interval| interval / interval_max).collect();

        let mut errors_by_day: Vec<(Date, usize)> = errors_by_day.into_iter().collect();
        errors_by_day.sort_by(|(_, cnt1), (_, cnt2)| cnt2.cmp(cnt1));
        let most_errors_in_a_day = errors_by_day.first().cloned();

        let mut occurrences_by_date: Vec<(Date, usize)> = occurrence_by_date.into_iter().collect();
        occurrences_by_date.sort_by(|(_, cnt1), (_, cnt2)| cnt2.cmp(cnt1));
        let most_commands_in_a_day = occurrences_by_date.first().cloned();

        Self {
            top_commands,
            top_directories,
            weekly_activity,
            daily_activity,
            shortest_commit_message,
            most_errors_in_a_day,
            most_commands_in_a_day,
            longest_running_command,
        }
    }
}

#[derive(Debug)]
struct Center {
    component: Box<dyn Component>,
    resize_warning: P,
}

impl Center {
    fn new(component: impl Component + 'static) -> Self {
        let resize_warning = P::new().push_text(
            "
            ▁▁▁▁▁▁▁▁▁▁▁▁
            ▏↖        ↗▕
            ▏          ▕
            ▏          ▕
            ▏↙        ↘▕
            ▔▔▔▔▔▔▔▔▔▔▔▔
Expand your terminal or decrease your
 font size to see your #FigWrapped!",
        );

        Self {
            component: Box::new(component),
            resize_warning,
        }
    }
}

impl Component for Center {
    fn draw(&self, state: &mut State, surface: &mut Surface, _: f64, _: f64, given_width: f64, given_height: f64) {
        let style = self.component.style(state);

        let size = self.component.size(state);
        let mut width = style.width().unwrap_or(size.0) + style.spacing_horizontal();
        let mut height = style.height().unwrap_or(size.1) + style.spacing_vertical();

        match width <= given_width && height <= given_height {
            true => self.component.draw(
                state,
                surface,
                given_width / 2.0 - width / 2.0,
                given_height / 2.0 - height / 2.0,
                width,
                height,
            ),
            false => {
                let style = self.resize_warning.style(state);
                let size = self.resize_warning.size(state);
                width = style.width().unwrap_or(size.0) + style.spacing_horizontal();
                height = style.height().unwrap_or(size.1) + style.spacing_vertical();
                self.resize_warning.draw(
                    state,
                    surface,
                    given_width / 2.0 - 16.0,
                    given_height / 2.0 - 5.0,
                    width,
                    height,
                )
            },
        }
    }

    fn type_selector(&self) -> &'static str {
        ""
    }

    fn inner(&self) -> &tui::component::ComponentData {
        self.component.inner()
    }

    fn inner_mut(&mut self) -> &mut tui::component::ComponentData {
        self.component.inner_mut()
    }

    fn size(&self, _: &mut State) -> (f64, f64) {
        (10000.0, 10000.0)
    }

    fn as_dyn_mut(&mut self) -> &mut dyn Component {
        self
    }
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct WrappedArgs;

impl WrappedArgs {
    pub async fn execute(self) -> Result<()> {
        // Time gate to December
        let current_time = time::OffsetDateTime::now_utc();
        if current_time.month() != time::Month::December {
            println!(
                "#FigWrapped is over, but you can still use Fig to make your terminal awesome.\n\nCheck back next December for #FigWrapped {}!",
                current_time.year()
            );
            return Ok(());
        }

        // We do the following first since it can fail
        let history = History::load()?.all_rows()?;
        let wrapped = Wrapped::new(history);

        tui::EventLoop::new(
            Center::new(P::new().push_text(
                " \"──────────*@@*──────────\"     .--~~~~~~~~~~~~~------.
                           /--===============------\\
We're glad you could make it   | |⎺⎺⎺⎺⎺⎺⎺⎺⎺⎺⎺⎺⎺⎺⎺|     |
We've had a great year      | | > fig|        |     |
                           | |               |     |
Thanks to you, we're       | |               |     |
spreading some holiday cheer   | |_______________|     |
                           |                   ::::|
  So press any key         '======================='
 to change the view        //-'-'-'-'-'-'-'-'-'-'-\\\\
                          //_'_'_'_'_'_'_'_'_'_'_'_\\\\
Here's to a bright future    [-------------------------]
and a happy new year to you!  \\_________________________/",
            )),
            tui::DisplayMode::AlternateScreen,
            InputMethod::new_exit_any(),
            StyleSheet::parse(include_str!("wrapped.css"), ParserOptions::default())?,
        )
        .run(|event, _, control_flow| {
            if let tui::Event::Quit | tui::Event::Terminate = event {
                *control_flow = ControlFlow::Quit;
            }
        })?;

        let view = Center::new(
            Div::new()
                .with_id("view")
                .push(
                    Div::new()
                        .with_layout(Layout::Horizontal)
                        .push(top_commands(&wrapped))
                        .push(Div::new().push(fig_logo()).push(top_directories(&wrapped)?)),
                )
                .push(
                    Div::new()
                        .with_layout(Layout::Horizontal)
                        .push(
                            Div::new()
                                .push(match rand::thread_rng().gen::<u32>() % 4 {
                                    0 if wrapped.shortest_commit_message.is_some() => shortest_commit_message(&wrapped),
                                    1 if wrapped.most_errors_in_a_day.is_some() => most_errors_in_a_day(&wrapped),
                                    2 if wrapped.longest_running_command.is_some() => longest_running_command(&wrapped),
                                    _ => busiest_day(&wrapped),
                                })
                                .push(weekly_activity(&wrapped)),
                        )
                        .push(daily_activity(&wrapped)),
                )
                .push(footer()),
        );

        tui::EventLoop::new(
            view,
            tui::DisplayMode::AlternateScreen,
            InputMethod::new_exit_any(),
            StyleSheet::parse(include_str!("wrapped.css"), ParserOptions::default())?,
        )
        .run(|event, _, control_flow| {
            if let tui::Event::Quit | tui::Event::Terminate = event {
                *control_flow = ControlFlow::Quit;
            }
        })?;

        Ok(())
    }
}

fn fig_logo() -> Div {
    Div::new()
        .with_id("fig_logo_div")
        .push(P::new().with_id("fig_logo").push_styled_text(
            "\
███████╗██╗ ██████╗
██╔════╝██║██╔════╝
█████╗  ██║██║  ███╗
██╔══╝  ██║██║   ██║
██║     ██║╚██████╔╝ 2022
╚═╝     ╚═╝ ╚═════╝  Wrapped",
            ColorAttribute::Default,
            ColorAttribute::Default,
            true,
            false,
        ))
}

fn top_commands(wrapped: &Wrapped) -> Div {
    let mut container = Div::new()
        .with_id("top_commands_div")
        .push(P::new().with_id("label").push_styled_text(
            "🏆 Top Commands",
            ColorAttribute::Default,
            ColorAttribute::Default,
            true,
            false,
        ));

    for command in wrapped.top_commands.iter().take(15) {
        container = container.push(P::new().push_text(format!(
            "{}{} {}",
            " ".repeat(5 - command.1.to_string().len().min(5)),
            command.1,
            command.0
        )));
    }

    container
}

fn top_directories(wrapped: &Wrapped) -> Result<Div> {
    let mut container = Div::new()
        .with_id("top_directories_div")
        .push(P::new().with_id("label").push_styled_text(
            "📁 Top Directories",
            ColorAttribute::Default,
            ColorAttribute::Default,
            true,
            false,
        ));

    let user_home = fig_util::directories::home_dir_utf8()?;
    let top_directories = wrapped.top_directories.iter().take(5).map(|(directory, cnt)| {
        match directory.strip_prefix(user_home.as_str()) {
            Some(dir) => (format!("~{dir}"), cnt),
            None => (directory.to_owned(), cnt),
        }
    });

    for directory in top_directories {
        container = container.push(P::new().push_text(format!(
            "{}{} {}",
            " ".repeat(5 - directory.1.to_string().len().min(5)),
            directory.1,
            directory.0
        )));
    }

    Ok(container)
}

fn weekly_activity(wrapped: &Wrapped) -> Div {
    let mut container = Div::new()
        .with_id("weekly_activity_div")
        .push(P::new().with_id("label").push_styled_text(
            "📅 Weekly Activity",
            ColorAttribute::Default,
            ColorAttribute::Default,
            true,
            false,
        ));

    for activity in &wrapped.weekly_activity {
        container = container.push(P::new().push_text(format!(
            "{} {}",
            activity.0,
            // todo: validate the 20.0 here
            "█".repeat((20.0 * activity.1).round() as usize)
        )));
    }

    container
}

fn daily_activity(wrapped: &Wrapped) -> Div {
    let mut container = Div::new()
        .with_id("daily_activity_div")
        .push(P::new().with_id("label").push_styled_text(
            "🕑 Daily Activity",
            ColorAttribute::Default,
            ColorAttribute::Default,
            true,
            false,
        ));

    for i in 0..wrapped.daily_activity.len() / 2 {
        let top = wrapped.daily_activity[i * 2] * 17.0 * 2.0;
        let bottom = wrapped.daily_activity[i * 2 + 1] * 17.0 * 2.0;

        let mut fill = String::new();
        for x in 0..17 {
            let top_left = top > x as f64 * 2.0;
            let top_right = top >= x as f64 * 2.0 + 1.0;
            let bottom_left = bottom > x as f64 * 2.0;
            let bottom_right = bottom >= x as f64 * 2.0 + 1.0;

            fill.push(match (top_left, top_right, bottom_left, bottom_right) {
                (true, true, true, true) => '█',
                (true, true, true, false) => '▛',
                (true, true, false, false) => '▀',
                (true, false, true, true) => '▙',
                (true, false, true, false) => '▌',
                (true, false, false, false) => '▘',
                (false, false, true, true) => '▄',
                (false, false, true, false) => '▖',
                (false, false, false, false) => ' ',
                _ => unreachable!(),
            });
        }

        container = container.push(
            P::new()
                .push_text(match i {
                    i if i == 0 => "12am ",
                    i if i == 3 => " 6am ",
                    i if i == 6 => "12pm ",
                    i if i == 9 => " 6pm ",
                    _ => "     ",
                })
                .push_text(fill),
        );
    }

    container
}

fn footer() -> P {
    P::new()
        .with_id("footer")
        .push_text("🎁 Share your ")
        .push_styled_text(
            "#FigWrapped",
            ColorAttribute::Default,
            ColorAttribute::Default,
            true,
            false,
        )
        .push_text(" with ")
        .push_styled_text("@fig", ColorAttribute::Default, ColorAttribute::Default, true, false)
}

fn shortest_commit_message(wrapped: &Wrapped) -> Div {
    Div::new()
        .with_id("factoid_div")
        .push(P::new().with_id("label").push_styled_text(
            "😬 Shortest Commit Msg",
            ColorAttribute::Default,
            ColorAttribute::Default,
            true,
            false,
        ))
        .push(match &wrapped.shortest_commit_message {
            Some((message, date)) => P::new().push_text(format!("'{message}' on ")).push_styled_text(
                date,
                ColorAttribute::Default,
                ColorAttribute::Default,
                false,
                true,
            ),
            None => P::new(),
        })
}

fn most_errors_in_a_day(wrapped: &Wrapped) -> Div {
    Div::new()
        .with_id("factoid_div")
        .push(P::new().with_id("label").push_styled_text(
            "❌ Most Errors in a Day",
            ColorAttribute::Default,
            ColorAttribute::Default,
            true,
            false,
        ))
        .push(match &wrapped.most_errors_in_a_day {
            Some((date, cnt)) => {
                let mut month = date.month().to_string();
                month.truncate(3);

                P::new().push_text(format!("{cnt} errors on ",)).push_styled_text(
                    format!("{month} {}", date.day()),
                    ColorAttribute::Default,
                    ColorAttribute::Default,
                    false,
                    true,
                )
            },
            None => P::new(),
        })
}

fn longest_running_command(wrapped: &Wrapped) -> Div {
    Div::new()
        .with_id("factoid_div")
        .push(P::new().with_id("label").push_styled_text(
            "⌛ Longest Running Command",
            ColorAttribute::Default,
            ColorAttribute::Default,
            true,
            false,
        ))
        .push(match &wrapped.longest_running_command {
            Some((command, duration)) => {
                let mut time_scale = "seconds";
                let mut length = duration.as_seconds_f64();

                if length / 60.0 > 1.0 {
                    time_scale = "minutes";
                    length /= 60.0;
                }

                if length / 60.0 > 1.0 {
                    time_scale = "hours";
                    length /= 60.0;
                }

                if length / 24.0 > 1.0 {
                    time_scale = "days";
                    length /= 24.0;
                }

                if length / 30.0 > 1.0 {
                    time_scale = "months";
                    length /= 30.0;
                }

                P::new().push_text(format!("'{command}' took ",)).push_styled_text(
                    format!("{length:.2} {time_scale}"),
                    ColorAttribute::Default,
                    ColorAttribute::Default,
                    false,
                    true,
                )
            },
            None => P::new(),
        })
}

fn busiest_day(wrapped: &Wrapped) -> Div {
    Div::new()
        .with_id("factoid_div")
        .push(P::new().with_id("label").push_styled_text(
            "🧰 Busiest Day",
            ColorAttribute::Default,
            ColorAttribute::Default,
            true,
            false,
        ))
        .push(match &wrapped.most_commands_in_a_day {
            Some((date, cnt)) => {
                let mut month = date.month().to_string();
                month.truncate(3);

                P::new().push_text(format!("{cnt} commands on ",)).push_styled_text(
                    format!("{month} {}", date.day()),
                    ColorAttribute::Default,
                    ColorAttribute::Default,
                    false,
                    true,
                )
            },
            None => P::new(),
        })
}

//  what could have been...
//
//                   @
//                  @@@
//              @@@@@@@@@@@
//               @@@@@@@@@
//                 @@@@@@
//                @@@@@`@b
//              @@@ @@ @@
//             @@@  @   @@`
//          @@@@@@    @@  @@@``
//           `@@  @   @@@   @@@
//          @@  @@@    @@b   @@@
//        `@@  @@@    d@@@@    @@
//     @@@@   @@@@  @  @ @@@@  @@@@b
//      @@@@@@@    .@  @`   @@ @
//          @@    @@@  @   @@  @@@
//         @@   @@@@@  @@  @@@   @@
//        @@   @@@@@   @@.     @@@@b.
//       @@    @@@@   @@@@   @@@  @@@@@
//    @@@    @  @"      @@   @@b
//  .@@@@@@@@` "  @@@  @@`".    @@
//       @@    @@@@@@@ @@@    @  @@@@
//      @@  .@@@@@@@@   @@@@ @@@    @@
//   @@@      `@@@       @@@@`  @@@@@@@@b
// d@@@@@@@` @@`   @  @@@  @@@@@@
//      @@ @@@@b  @@  @@@    @@@@
//     @@  @@@@@ @@@  @@@  @    @@@@
//   @@@  @@@@@@ @@   @@@` @@@      @@@@
// @@   d@@@          @@@    @@   @     @@@
//         @@@@    @@@ @@     @@b  @@@     @@@
// @@@@@@@@" @@ @@@@ @@@@@  @@ "@@@@@@@@@@@@  `
//           @@@@ @    @@@@@@
//                @     @ @@
//                @     @
//                @@   @@
//                 @@@@@

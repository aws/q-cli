use std::collections::HashMap;

use clap::Args;
use fig_history::History;
use time::{
    Duration,
    OffsetDateTime,
    Time,
    UtcOffset,
    Weekday,
};
use tui::component::{
    Container,
    Layout,
    Paragraph,
};
use tui::{
    BorderStyle,
    ColorAttribute,
    ControlFlow,
    InputMethod,
};

#[derive(Debug, Args, PartialEq, Eq)]
pub struct WrappedArgs;

impl WrappedArgs {
    pub async fn execute(self) -> eyre::Result<()> {
        let mut commands_by_occurrence = HashMap::new();
        let mut directories_by_occurrence = HashMap::new();
        let mut shortest_commit_message = None;
        let mut weekly_activity = vec![0_f64; 7];
        let mut times = vec![];
        for row in History::load()?.all_rows()? {
            if let Some(command) = row.command {
                let command = match command.split_once(' ') {
                    Some((command, rest)) => {
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

                *commands_by_occurrence.entry(command.to_owned()).or_insert_with(|| 0) += 1;
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

        let mut commands: Vec<(String, usize)> = commands_by_occurrence.into_iter().collect();
        commands.sort_by(|(_, cnt), (_, cnt2)| cnt2.cmp(cnt));

        let mut directories: Vec<(String, usize)> = directories_by_occurrence.into_iter().collect();
        directories.sort_by(|(_, cnt), (_, cnt2)| cnt2.cmp(cnt));

        let top_commands = commands.iter().take(15);

        let user_home = fig_util::directories::home_dir_utf8()?;
        let most_used_directories =
            directories
                .iter()
                .take(5)
                .map(|(directory, cnt)| match directory.strip_prefix(user_home.as_str()) {
                    Some(dir) => (format!("~{dir}"), cnt),
                    None => (directory.to_owned(), cnt),
                });

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

        let style_sheet = tui::style_sheet! {
            "*" => {
                border_style: BorderStyle::Ascii { top_left: '┌', top: '─', top_right: '┐', left: '│', right: '│', bottom_left: '└', bottom: '─', bottom_right: '┘' };
            },
            "p" => {
                margin_left: 1.0;
                margin_right: 1.0;
            },
            "#view" => {
                margin_left: 2.0;
                margin_right: 2.0;
                margin_top: 1.0;
                margin_bottom: 1.0;
            },
            "#top_commands_div" => {
                border_color: ColorAttribute::PaletteIndex(4);
                border_width: 1.0;
                height: Some(17.0);
                width: Some(25.0);
            },
            "#fig_logo_div" => {
                border_color: ColorAttribute::PaletteIndex(1);
                border_width: 1.0;
                padding_left: 2.0;
                width: Some(33.0);
            },
            "#most_used_directories_div" => {
                border_color: ColorAttribute::PaletteIndex(5);
                border_width: 1.0;
                height: Some(7.0);
                width: Some(35.0);
            },
            "#shortest_commit_message_div" => {
                border_color: ColorAttribute::PaletteIndex(3);
                border_width: 1.0;
                width: Some(30.0);
            },
            "#weekly_activity_div" => {
                border_color: ColorAttribute::PaletteIndex(2);
                border_width: 1.0;
                width: Some(30.0);
            },
            "#daily_activity_div" => {
                border_color: ColorAttribute::PaletteIndex(6);
                border_width: 1.0;
                width: Some(30.0);
            },
            "#label" => {
                padding_bottom: 1.0;
            },
            "#footer" => {
                margin_top: 1.0;
                margin_left: 14.0;
            }
        };

        let mut top_commands_div = Container::new("top_commands_div", Layout::Vertical);
        top_commands_div.push(Paragraph::new("label").push_styled_text(
            "🏆 Top Commands",
            ColorAttribute::Default,
            ColorAttribute::Default,
            true,
            false,
        ));

        for command in top_commands {
            top_commands_div.push(Paragraph::new("").push_text(format!(
                "{}{} {}",
                " ".repeat(5 - command.1.to_string().len().min(5)),
                command.1,
                command.0
            )));
        }

        let mut fig_logo_div = Container::new("fig_logo_div", Layout::Vertical);
        fig_logo_div.push(Paragraph::new("fig_logo").push_styled_text(
            "
███████╗██╗ ██████╗
██╔════╝██║██╔════╝
█████╗  ██║██║  ███╗
██╔══╝  ██║██║   ██║
██║     ██║╚██████╔╝ 2022
╚═╝     ╚═╝ ╚═════╝  Wrapped
        ",
            ColorAttribute::Default,
            ColorAttribute::Default,
            true,
            false,
        ));

        let mut most_used_directories_div = Container::new("most_used_directories_div", Layout::Vertical);
        most_used_directories_div.push(Paragraph::new("label").push_styled_text(
            "📁 Most Used Directories",
            ColorAttribute::Default,
            ColorAttribute::Default,
            true,
            false,
        ));

        for directory in most_used_directories {
            most_used_directories_div.push(Paragraph::new("").push_text(format!(
                "{}{} {}",
                " ".repeat(5 - directory.1.to_string().len().min(5)),
                directory.1,
                directory.0
            )));
        }

        let mut top_right = Container::new("top_right", Layout::Vertical);
        top_right.push(fig_logo_div).push(most_used_directories_div);

        let mut top_half = Container::new("top_half", Layout::Horizontal);
        top_half.push(top_commands_div).push(top_right);

        let mut shortest_commit_message_div = Container::new("shortest_commit_message_div", Layout::Vertical);
        shortest_commit_message_div
            .push(Paragraph::new("label").push_styled_text(
                "😬 Shortest Commit Msg",
                ColorAttribute::Default,
                ColorAttribute::Default,
                true,
                false,
            ))
            .push(match shortest_commit_message {
                Some((message, date)) => Paragraph::new("")
                    .push_text(format!("'{}' on ", message))
                    .push_styled_text(date, ColorAttribute::Default, ColorAttribute::Default, false, true),
                None => Paragraph::new(""),
            });

        let mut weekly_activity_div = Container::new("weekly_activity_div", Layout::Vertical);
        weekly_activity_div.push(Paragraph::new("label").push_styled_text(
            "📅 Weekly Activity",
            ColorAttribute::Default,
            ColorAttribute::Default,
            true,
            false,
        ));

        for activity in weekly_activity {
            weekly_activity_div.push(Paragraph::new("").push_text(format!(
                "{} {}",
                activity.0,
                // todo: validate the 20.0 here
                "█".repeat((20.0 * activity.1).round() as usize)
            )));
        }

        let mut bottom_left = Container::new("bottom_left", Layout::Vertical);
        bottom_left.push(shortest_commit_message_div).push(weekly_activity_div);

        let mut daily_activity_div = Container::new("daily_activity_div", Layout::Vertical);
        daily_activity_div.push(Paragraph::new("label").push_styled_text(
            "🕑 Daily Activity",
            ColorAttribute::Default,
            ColorAttribute::Default,
            true,
            false,
        ));

        for i in 0..daily_activity.len() / 2 {
            let top = daily_activity[i * 2] * 23.0 * 2.0;
            let bottom = daily_activity[i * 2 + 1] * 23.0 * 2.0;

            let mut fill = String::new();
            for x in 0..23 {
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

            daily_activity_div.push(
                Paragraph::new("")
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

        let mut bottom_half = Container::new("bottom_half", Layout::Horizontal);
        bottom_half.push(bottom_left).push(daily_activity_div);

        let footer = Paragraph::new("footer")
            .push_text("🎁 Share your ")
            .push_styled_text(
                "#FigWrapped",
                ColorAttribute::Default,
                ColorAttribute::Default,
                true,
                false,
            )
            .push_text(" with ")
            .push_styled_text("@fig", ColorAttribute::Default, ColorAttribute::Default, true, false);

        let mut view = Container::new("view", Layout::Vertical);
        view.push(top_half).push(bottom_half).push(footer);

        tui::EventLoop::new().run(&mut view, InputMethod::None, style_sheet, |event, _, control_flow| {
            if let tui::Event::Quit | tui::Event::Terminate = event {
                *control_flow = ControlFlow::Quit;
            }
        })?;

        Ok(())
    }
}

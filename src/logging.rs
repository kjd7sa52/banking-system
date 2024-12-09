use simplelog::{ColorChoice, ConfigBuilder, LevelPadding, TermLogger, TerminalMode};

pub fn setup() -> Result<(), log::SetLoggerError> {
    TermLogger::init(
        log::LevelFilter::Debug,
        ConfigBuilder::new()
            .set_level_padding(LevelPadding::Right)
            .set_thread_level(log::LevelFilter::Off)
            .set_target_level(log::LevelFilter::Off)
            .build(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )
}

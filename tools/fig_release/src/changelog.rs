use eyre::ContextCompat;

use crate::utils::{
    read_release_file,
    write_release_file,
};

pub fn add() -> eyre::Result<()> {
    let mut release = read_release_file()?;
    let edited = dialoguer::Editor::new()
        .edit("")?
        .context("new changelog item not saved")?;
    release.changelog.push(edited);
    write_release_file(&release)?;
    Ok(())
}

pub fn edit() -> eyre::Result<()> {
    let mut release = read_release_file()?;
    let idx = dialoguer::FuzzySelect::new().items(&release.changelog).interact()?;
    let edited = dialoguer::Editor::new()
        .edit(&release.changelog[idx])?
        .context("edited changelog item not saved")?;
    release.changelog[idx] = edited;
    write_release_file(&release)?;
    Ok(())
}

pub fn remove() -> eyre::Result<()> {
    let mut release = read_release_file()?;
    let idx = dialoguer::FuzzySelect::new().items(&release.changelog).interact()?;
    release.changelog.remove(idx);
    write_release_file(&release)?;
    Ok(())
}

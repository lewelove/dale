mod cli;
mod diff;
mod mapping;
mod models;
mod tag;
#[cfg(test)]
mod tests;

use anyhow::Result;
use models::{AutoMode, CliOptions, CoverDiffDisplay, CoverStatus};

fn process_album_batch(mut opts: CliOptions) -> Result<()> {
    let disk_cover = tag::read_disk_cover(opts.cover.as_deref())?;
    let cover_status = tag::resolve_cover_status(&opts.tasks, disk_cover.as_ref())?;
    let mut has_changes = cover_status == CoverStatus::Update;

    for task in &mut opts.tasks {
        if diff::evaluate_track_diffs(
            task,
            opts.delete_other_tags,
            opts.delete_other_covers,
        )? {
            has_changes = true;
        }
    }

    if !has_changes {
        return Ok(());
    }

    let common_diffs = diff::dedup_common_diffs(&mut opts.tasks);
    let show_cover_diff = if cover_status == CoverStatus::Update {
        CoverDiffDisplay::Show
    } else {
        CoverDiffDisplay::Hide
    };

    let header = diff::resolve_header(&opts.tasks);
    diff::print_diffs(&header, &opts.tasks, &common_diffs, show_cover_diff);

    if opts.auto_mode == AutoMode::Prompt && !diff::prompt_confirm()? {
        return Ok(());
    }

    tag::write_tasks(
        &opts.tasks,
        cover_status,
        disk_cover.as_ref(),
        opts.delete_other_tags,
        opts.delete_other_covers,
    )?;

    println!("\x1b[32m✔ Done.\x1b[0m");
    Ok(())
}

fn main() -> Result<()> {
    let opts = cli::parse_cli_args()?;
    process_album_batch(opts)?;
    Ok(())
}

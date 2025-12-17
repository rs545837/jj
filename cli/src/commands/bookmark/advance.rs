// Copyright 2026 The Jujutsu Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use clap_complete::ArgValueCandidates;
use clap_complete::ArgValueCompleter;

use super::BookmarkMoveArgs;
use super::move_bookmarks_with_args;
use crate::cli_util::CommandHelper;
use crate::cli_util::RevisionArg;
use crate::command_error::CommandError;
use crate::command_error::user_error;
use crate::complete;
use crate::ui::Ui;

/// Move existing bookmarks to a target revision with smart defaults
///
/// This is a specialization of `jj bookmark move` that provides convenient
/// defaults for `--from` and `--to`, which makes it easy to move bookmarks
/// relative to the working copy.
///
/// The argument `--to` defaults to `closest_pushable(@)` and `--from` defaults
/// to `closest_bookmarks(to)`. So by default, the command finds the closest
/// bookmarks to @ and moves them to the nearest revision considered pushable.
/// The default notion of pushable are described revisions that either have
/// contents, or are merges.
///
/// Examples:
///
/// `jj bookmark advance` - Move closest bookmarks to the nearest pushable
/// commit
///
/// `jj bookmark advance --to @-` - Move closest bookmarks to the working-copy
/// parent
///
/// `jj bookmark advance --from main` Move a specific bookmark to the nearest
/// pushable commit
#[derive(clap::Args, Clone, Debug)]
pub struct BookmarkAdvanceArgs {
    /// Move bookmarks matching the given name patterns
    ///
    /// By default, the specified pattern matches bookmark names with glob
    /// syntax. You can also use other [string pattern syntax].
    ///
    /// [string pattern syntax]:
    ///     https://docs.jj-vcs.dev/latest/revsets/#string-patterns
    #[arg(add = ArgValueCandidates::new(complete::local_bookmarks))]
    names: Option<Vec<String>>,

    /// Move bookmarks from the given revisions
    ///
    /// If no bookmark names or source revisions are specified, then this revset
    /// defaults to `closest_bookmarks(to)`.
    #[arg(long, short, value_name = "REVSETS")]
    #[arg(add = ArgValueCompleter::new(complete::revset_expression_all))]
    from: Vec<RevisionArg>,

    /// Move bookmarks to this revision
    #[arg(
        long,
        short,
        default_value = "closest_pushable(@)",
        value_name = "REVSET"
    )]
    #[arg(add = ArgValueCompleter::new(complete::revset_expression_all))]
    to: RevisionArg,

    /// Allow moving bookmarks backwards or sideways
    #[arg(long, short = 'B')]
    allow_backwards: bool,
}

pub fn cmd_bookmark_advance(
    ui: &mut Ui,
    command: &CommandHelper,
    args: &BookmarkAdvanceArgs,
) -> Result<(), CommandError> {
    let workspace_command = command.workspace_helper(ui)?;

    // Validate the target revision and provide a better error message for the
    // default
    workspace_command
        .resolve_single_rev(ui, &args.to)
        .map_err(|err| {
            // Provide a better error message when the default
            // closest_pushable(@) doesn't resolve.
            if args.to.as_ref() == "closest_pushable(@)" {
                user_error("No suitable revision to advance to.").hinted(
                    "The revset `closest_pushable` controls the default target. You can also \
                     specify a specific target with `--to`.",
                )
            } else {
                err
            }
        })?;

    let from = if args.from.is_empty() && args.names.is_none() {
        // If neither `--from` nor `names` are provided, default to closest bookmark.
        vec![RevisionArg::from("closest_bookmarks(@)".to_string())]
    } else {
        args.from.clone()
    };

    // Delegate to the move command
    move_bookmarks_with_args(
        ui,
        command,
        &BookmarkMoveArgs {
            names: args.names.clone(),
            from,
            to: args.to.clone(),
            allow_backwards: args.allow_backwards,
        },
    )
}

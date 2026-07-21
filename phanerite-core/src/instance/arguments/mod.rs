use crate::instance::arguments::variables::Variables;
use crate::instance::instance_info::{Action, Argument, VersionManifest};
use std::collections::HashSet;
use std::iter::Peekable;

pub mod variables;

pub struct LaunchArguments {
    pub args: Vec<(String, Option<String>)>,
}

impl LaunchArguments {
    pub fn from_vars(manifest: &VersionManifest, variables: Variables) -> Self {
        if let Some(args) = &manifest.arguments {
            let raw = args.jvm.iter().chain(args.game.iter());
            let flattened = flatten_arguments(raw, &variables.feat).peekable();
            let chunked = chunk_arguments(flattened);
            let args = chunked
                .filter_map(|(x, y)| match variables.resolve(x) {
                    None => None,
                    Some(x) => match y {
                        None => Some((x, None)),
                        Some(y) => variables.resolve(y).map(|y| (x, Some(y))),
                    },
                })
                .collect();
            Self { args }
        } else if let Some(args) = &manifest.minecraft_arguments
            && let Some(args) = variables.resolve(args)
        {
            Self {
                args: vec![(args, None)],
            }
        } else {
            Self { args: vec![] }
        }
    }
}

fn flatten_arguments<'a>(
    arguments: impl Iterator<Item = &'a Argument> + 'a,
    features: &'a HashSet<&'static str>,
) -> impl Iterator<Item = &'a String> + 'a {
    arguments.flat_map(|arg| match arg {
        Argument::String(s) => vec![s],
        Argument::Conditional(c) => {
            if c.rules.iter().fold(false, |b, rule| {
                rule.evaluate(features)
                    .map_or(b, |a| matches!(a, Action::Allow))
            }) {
                c.value.iter().collect()
            } else {
                vec![]
            }
        }
    })
}

fn chunk_arguments<'a, I>(
    mut input: Peekable<I>,
) -> impl Iterator<Item = (&'a String, Option<&'a String>)>
where
    I: Iterator<Item = &'a String> + 'a,
{
    std::iter::from_fn(move || {
        let first = input.next()?;

        let second = match input.peek() {
            Some(s) if pure_var(s) => input.next(),
            _ => None,
        };

        Some((first, second))
    })
}

fn pure_var(input: &str) -> bool {
    input
        .strip_prefix("${")
        .and_then(|x| x.strip_suffix('}'))
        .is_some()
}

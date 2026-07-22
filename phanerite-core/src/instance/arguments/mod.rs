use crate::instance::arguments::variables::Variables;
use crate::instance::instance_info::{Action, Argument};
use crate::instance::Instance;
use std::collections::HashSet;
use std::iter::Peekable;

pub mod variables;

pub struct LaunchArguments {
    pub args: Vec<(String, Option<String>)>,
}

impl Instance {
    pub fn to_arguments(&self, variables: Variables) -> LaunchArguments {
        let main_class = Argument::String(self.manifest.main_class.to_string());
        let main_class = std::iter::once(&main_class);
        if let Some(args) = &self.manifest.arguments {
            let raw = args.jvm.iter().chain(main_class).chain(args.game.iter());
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
            LaunchArguments { args }
        } else if let Some(args) = &self.manifest.minecraft_arguments
            && let Some(args) = variables.resolve(args)
        {
            LaunchArguments {
                args: vec![(args, None)],
            }
        } else {
            LaunchArguments { args: vec![] }
        }
    }
}

impl LaunchArguments {
    pub fn flatten_iter(self) -> impl IntoIterator<Item = String> {
        self.args
            .into_iter()
            .flat_map(|(x, y)| std::iter::once(x).chain(y))
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

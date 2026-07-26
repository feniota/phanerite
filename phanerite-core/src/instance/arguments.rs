use crate::instance::instance_info::{Action, Argument};
use crate::instance::variables::Variables;
use crate::instance::Instance;
use std::collections::HashSet;
use std::iter::Peekable;

pub struct LaunchArguments {
    main_class: String,
    jvm: Vec<(String, Option<String>)>,
    game: Vec<(String, Option<String>)>,
}

impl Variables {
    pub fn to_arguments(&self, instance: &Instance) -> LaunchArguments {
        let main_class = instance.manifest.main_class.to_string();
        if let Some(args) = &instance.manifest.arguments {
            let flattened_jvm = flatten_arguments(args.jvm.iter(), &self.feat).peekable();
            let flattened_game = flatten_arguments(args.jvm.iter(), &self.feat).peekable();
            let chunked_jvm = chunk_arguments(flattened_jvm);
            let chunked_game = chunk_arguments(flattened_game);
            let jvm = chunked_jvm
                .filter_map(|(x, y)| filter_none(&self, x, y))
                .collect();
            let game = chunked_game
                .filter_map(|(x, y)| filter_none(&self, x, y))
                .collect();
            LaunchArguments {
                main_class,
                jvm,
                game,
            }
        } else if let Some(args) = &instance.manifest.minecraft_arguments
            && let Some(args) = self.resolve(args)
        {
            LaunchArguments {
                main_class,
                jvm: vec![],
                game: vec![(args, None)],
            }
        } else {
            LaunchArguments {
                main_class,
                jvm: vec![],
                game: vec![],
            }
        }
    }
}

impl LaunchArguments {
    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.jvm
            .iter()
            .flat_map(|(x, y)| std::iter::once(x).chain(y))
            .chain(std::iter::once(&self.main_class))
            .chain(
                self.game
                    .iter()
                    .flat_map(|(x, y)| std::iter::once(x).chain(y)),
            )
    }
}

/// 将条件参数转换为字符串
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

/// 对参数分块
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

/// 排除无法替换的参数
fn filter_none(
    variables: &Variables,
    x: &str,
    y: Option<&String>,
) -> Option<(String, Option<String>)> {
    match variables.resolve(x) {
        None => None,
        Some(x) => match y {
            None => Some((x, None)),
            Some(y) => variables.resolve(y).map(|y| (x, Some(y))),
        },
    }
}

#[inline]
fn pure_var(input: &str) -> bool {
    input
        .strip_prefix("${")
        .and_then(|x| x.strip_suffix('}'))
        .is_some()
}

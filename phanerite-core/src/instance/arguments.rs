use crate::error::Result;
use crate::instance::Instance;
use crate::instance::manifest::{Action, Argument};
use crate::instance::variables::Variables;
use crate::utils::state::{NotReady, Ready};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::iter::Peekable;

// jvm 参数 + main_class + game 参数
/// JVM arguments + main class + game arguments
pub struct LaunchArguments {
    pub(crate) main_class: String,
    pub(crate) jvm: HashMap<String, Option<String>>,
    pub(crate) game: HashMap<String, Option<String>>,
}

impl LaunchArguments {
    // 设置 JVM 内存，单位 MiB
    /// Sets the JVM memory, in MiB
    pub fn set_memory(mut self, min: Option<u64>, max: Option<u64>) -> Self {
        match min {
            Some(size) => {
                self.jvm.insert("-Xms".into(), Some(format!("{size}M")));
            }
            None => {
                self.jvm.remove("-Xms");
            }
        }
        match max {
            Some(size) => {
                self.jvm.insert("-Xmx".into(), Some(format!("{size}M")));
            }
            None => {
                self.jvm.remove("-Xmx");
            }
        }
        self
    }
}

impl Display for LaunchArguments {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (key, value) in &self.jvm {
            write!(f, "{key}")?;
            if let Some(value) = value {
                write!(f, " {value}")?;
            }
            writeln!(f)?;
        }

        writeln!(f, "{}", self.main_class)?;

        for (key, value) in &self.game {
            write!(f, "{key}")?;
            if let Some(value) = value {
                write!(f, " {value}")?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl Variables<NotReady> {
    pub fn to_arguments<R: Clone, C: Clone>(
        self,
        instance: &Instance<R, C>,
    ) -> Result<LaunchArguments> {
        let vars = self.generated(instance)?;
        Ok(vars.to_arguments(instance))
    }
}

impl Variables<Ready> {
    pub fn to_arguments<R: Clone, C: Clone>(&self, instance: &Instance<R, C>) -> LaunchArguments {
        let main_class = instance.manifest.main_class.to_string();
        if let Some(args) = &instance.manifest.arguments {
            let flattened_jvm = flatten_arguments(args.jvm.iter(), &self.feat).peekable();
            let flattened_game = flatten_arguments(args.game.iter(), &self.feat).peekable();
            let chunked_jvm = chunk_arguments(flattened_jvm);
            let chunked_game = chunk_arguments(flattened_game);
            let jvm = chunked_jvm
                .filter_map(|(x, y)| filter_none(self, x, y))
                .collect();
            let game = chunked_game
                .filter_map(|(x, y)| filter_none(self, x, y))
                .collect();
            LaunchArguments {
                main_class,
                jvm,
                game,
            }
        } else if let Some(args) = &instance.manifest.minecraft_arguments
            && let Some(args) = self.resolve(args)
        {
            let mut game = HashMap::new();
            game.insert(args, None);
            LaunchArguments {
                main_class,
                jvm: HashMap::new(),
                game,
            }
        } else {
            LaunchArguments {
                main_class,
                jvm: HashMap::new(),
                game: HashMap::new(),
            }
        }
    }
}

// 将条件参数转换为字符串
/// Turns the conditional arguments into strings
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

// 对参数分块
/// Chunks the arguments
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

// 排除无法替换的参数
/// Filters out the arguments that cannot be substituted
fn filter_none(
    variables: &Variables<Ready>,
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

fn pure_var(input: &str) -> bool {
    input
        .strip_prefix("${")
        .and_then(|x| x.strip_suffix('}'))
        .is_some()
}

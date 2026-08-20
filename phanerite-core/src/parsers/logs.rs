use futures::Stream;
use std::{
    collections::{HashMap, VecDeque},
    pin::Pin,
};

/// Minecraft 日志解析器。
///
/// Parser 是有状态的：
///
/// - update() 接收任意大小的日志 chunk
/// - 内部自动处理不完整行
/// - 每一行可能产生多个 Parsed
/// - State 保存从历史日志中推导出的上下文
#[derive(Default)]
pub struct LogsParser {
    state: State,
    config: Config,
    buffer: String,
}

/// Parser 的配置。
#[derive(Clone, Debug)]
pub struct Config {
    /// 是否产生普通的 Log 事件。
    pub emit_logs: bool,

    /// 是否产生无法识别格式的 Raw 事件。
    pub emit_raw: bool,

    /// 是否解析玩家事件。
    pub parse_players: bool,

    /// 是否解析聊天。
    pub parse_chat: bool,

    /// 是否解析死亡。
    pub parse_death: bool,

    /// 是否解析 advancement。
    pub parse_advancement: bool,

    /// 是否解析性能相关日志。
    pub parse_performance: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            emit_logs: true,
            emit_raw: true,
            parse_players: true,
            parse_chat: true,
            parse_death: true,
            parse_advancement: true,
            parse_performance: true,
        }
    }
}

/// Minecraft 日志的持久化解析状态。
///
/// 这里放“从过去日志中可以推导出来、并且对未来解析有意义”的信息。
#[derive(Debug, Default)]
pub struct State {
    /// 当前服务器生命周期。
    pub server: ServerState,

    /// 当前已知的玩家。
    pub players: HashMap<String, PlayerState>,

    /// 最近一次观察到的性能状态。
    pub performance: PerformanceState,

    /// 当前世界相关状态。
    pub world: WorldState,
}

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub name: String,
    pub uuid: Option<String>,
    pub online: bool,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerStatus {
    #[default]
    Unknown,
    Starting,
    Running,
    Stopping,
    Stopped,
}

#[derive(Debug, Default)]
pub struct ServerState {
    pub status: ServerStatus,
    pub version: Option<String>,
    pub startup_time_secs: Option<f64>,
}

#[derive(Debug, Default)]
pub struct PerformanceState {
    /// 最近一次 "Can't keep up" 的延迟。
    pub last_lag_ms: Option<u64>,

    /// 最近一次 "Can't keep up" 报告的 tick 数。
    pub last_lag_ticks: Option<u64>,
}

#[derive(Debug, Default)]
pub struct WorldState {
    pub saving: bool,
}

/// 一个完整日志行解析后产生的事件。
#[derive(Debug, Clone)]
pub enum Parsed {
    /// 标准 Minecraft 日志。
    Log(Log),

    /// 无法识别为标准 Minecraft 日志的输入。
    Raw(String),

    Server(ServerEvent),

    Player(PlayerEvent),

    Chat {
        player: String,
        message: String,
    },

    Death {
        player: String,
        message: String,
    },

    Advancement {
        player: String,
        advancement: String,
    },

    Performance(PerformanceEvent),

    World(WorldEvent),
}

#[derive(Debug, Clone)]
pub enum ServerEvent {
    Starting { version: Option<String> },

    Started { startup_time_secs: Option<f64> },

    Stopping,
}

#[derive(Debug, Clone)]
pub enum PlayerEvent {
    Joined {
        player: String,
    },

    Left {
        player: String,
    },

    Connected {
        player: Option<String>,
    },

    Disconnected {
        player: Option<String>,
        reason: String,
    },
}

#[derive(Debug, Clone)]
pub enum PerformanceEvent {
    Lagging {
        milliseconds: u64,
        ticks: Option<u64>,
    },
}

#[derive(Debug, Clone)]
pub enum WorldEvent {
    Saving,
    SaveFinished,
}

#[derive(Debug, Clone)]
pub struct Log {
    pub time: LogTime,
    pub thread: String,
    pub level: Level,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogTime {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub millis: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogsParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    /// 当前解析状态。
    pub fn state(&self) -> &State {
        &self.state
    }

    /// 输入任意大小的日志 chunk。
    ///
    /// 输入不要求：
    ///
    /// - 一次只有一行
    /// - 一定以换行符结尾
    ///
    /// 只有已经完整接收到的行才会被解析。
    pub fn update(&mut self, input: impl AsRef<str>) -> Vec<Parsed> {
        self.buffer.push_str(input.as_ref());

        let mut result = Vec::new();

        while let Some(pos) = self.buffer.find('\n') {
            let mut line = self.buffer.drain(..=pos).collect::<String>();

            // 去掉 \n。
            line.pop();

            // 同时支持 Windows 的 \r\n。
            if line.ends_with('\r') {
                line.pop();
            }

            if !line.is_empty() {
                result.extend(self.parse_line(&line));
            }
        }

        result
    }

    /// 输入结束时调用。
    ///
    /// 用于处理最后一个没有 `\n` 的日志行。
    pub fn finish(&mut self) -> Vec<Parsed> {
        if self.buffer.is_empty() {
            return Vec::new();
        }

        let line = std::mem::take(&mut self.buffer);

        self.parse_line(&line)
    }

    fn parse_line(&mut self, line: &str) -> Vec<Parsed> {
        let Some(log) = parse_log_line(line) else {
            if self.config.emit_raw {
                return vec![Parsed::Raw(line.to_owned())];
            }

            return Vec::new();
        };

        let mut result = Vec::new();

        self.update_state(&log, &mut result);

        if self.config.parse_players {
            self.parse_player_event(&log, &mut result);
        }

        if self.config.parse_chat {
            self.parse_chat(&log, &mut result);
        }

        if self.config.parse_death {
            self.parse_death(&log, &mut result);
        }

        if self.config.parse_advancement {
            self.parse_advancement(&log, &mut result);
        }

        if self.config.parse_performance {
            self.parse_performance(&log, &mut result);
        }

        self.parse_world(&log, &mut result);

        if self.config.emit_logs {
            result.push(Parsed::Log(log));
        }

        result
    }

    /// 更新长期状态。
    fn update_state(&mut self, log: &Log, events: &mut Vec<Parsed>) {
        let message = log.message.as_str();

        /*
         * Starting
         *
         * [12:00:00] [Server thread/INFO]: Starting minecraft server version 1.21.8
         */
        if let Some(version) = message.strip_prefix("Starting minecraft server version") {
            let version = version.trim();

            self.state.server.status = ServerStatus::Starting;
            self.state.server.version = (!version.is_empty()).then(|| version.to_owned());

            events.push(Parsed::Server(ServerEvent::Starting {
                version: self.state.server.version.clone(),
            }));

            return;
        }

        /*
         * Started
         *
         * [12:00:03] [Server thread/INFO]: Done (3.123s)! For help, type "help"
         */
        if let Some(rest) = message.strip_prefix("Done (") {
            self.state.server.status = ServerStatus::Running;

            let startup_time_secs = rest
                .split_once(')')
                .and_then(|(value, _)| value.trim_end_matches('s').parse().ok());

            self.state.server.startup_time_secs = startup_time_secs;

            events.push(Parsed::Server(ServerEvent::Started { startup_time_secs }));

            return;
        }

        /*
         * Stopping
         */
        if message == "Stopping server" || message.starts_with("Stopping server") {
            self.state.server.status = ServerStatus::Stopping;

            events.push(Parsed::Server(ServerEvent::Stopping));
        }
    }

    fn parse_player_event(&mut self, log: &Log, events: &mut Vec<Parsed>) {
        let message = log.message.as_str();

        /*
         * Steve joined the game
         */
        if let Some(player) = message.strip_suffix(" joined the game") {
            let player = player.to_owned();

            self.state
                .players
                .entry(player.clone())
                .and_modify(|state| {
                    state.online = true;
                })
                .or_insert_with(|| PlayerState {
                    name: player.clone(),
                    uuid: None,
                    online: true,
                });

            events.push(Parsed::Player(PlayerEvent::Joined { player }));

            return;
        }

        /*
         * Steve left the game
         */
        if let Some(player) = message.strip_suffix(" left the game") {
            let player = player.to_owned();

            if let Some(state) = self.state.players.get_mut(&player) {
                state.online = false;
            }

            events.push(Parsed::Player(PlayerEvent::Left { player }));

            return;
        }

        /*
         * Disconnecting ...
         */
        if let Some(player) = parse_disconnect_player(message) {
            events.push(Parsed::Player(PlayerEvent::Disconnected {
                player: player.map(str::to_owned),
                reason: message.to_owned(),
            }));
        }
    }

    fn parse_chat(&self, log: &Log, events: &mut Vec<Parsed>) {
        let message = log.message.as_str();

        /*
         * Vanilla：
         *
         * <Steve> hello
         */
        let Some(message) = message.strip_prefix('<') else {
            return;
        };

        let Some((player, message)) = message.split_once("> ") else {
            return;
        };

        if player.is_empty() {
            return;
        }

        events.push(Parsed::Chat {
            player: player.to_owned(),
            message: message.to_owned(),
        });
    }

    fn parse_death(&self, log: &Log, events: &mut Vec<Parsed>) {
        let message = log.message.as_str();

        /*
         * Minecraft 的死亡消息非常多。
         *
         * 不建议在这里试图完整枚举所有可能性。
         * 后续可以把这一块单独抽成 DeathMessageParser。
         */
        const PATTERNS: &[&str] = &[
            " was slain by ",
            " was shot by ",
            " was killed by ",
            " was blown up by ",
            " was fireballed by ",
            " was pummeled by ",
            " was stung to death by ",
            " was squashed by ",
            " was impaled by ",
            " was skewered by ",
            " was doomed to fall by ",
            " hit the ground too hard",
            " fell from a high place",
            " fell off ",
            " drowned",
            " burned to death",
            " went up in flames",
            " walked into fire",
            " was burned to a crisp",
            " froze to death",
            " starved to death",
            " suffocated in a wall",
            " was squished too much",
            " was pricked to death",
            " blew up",
            " died",
        ];

        for pattern in PATTERNS {
            let Some(index) = message.find(pattern) else {
                continue;
            };

            let player = message[..index].trim();

            if player.is_empty() || player.contains(' ') {
                continue;
            }

            events.push(Parsed::Death {
                player: player.to_owned(),
                message: message.to_owned(),
            });

            break;
        }
    }

    fn parse_advancement(&self, log: &Log, events: &mut Vec<Parsed>) {
        let message = log.message.as_str();

        const PREFIXES: &[&str] = &[
            " has made the advancement [",
            " has reached the goal [",
            " has completed the challenge [",
        ];

        for prefix in PREFIXES {
            let Some(index) = message.find(prefix) else {
                continue;
            };

            let player = &message[..index];
            let rest = &message[index + prefix.len()..];

            let Some(advancement) = rest.strip_suffix(']') else {
                continue;
            };

            events.push(Parsed::Advancement {
                player: player.to_owned(),
                advancement: advancement.to_owned(),
            });

            break;
        }
    }

    fn parse_performance(&mut self, log: &Log, events: &mut Vec<Parsed>) {
        let message = log.message.as_str();

        /*
         * Can't keep up!
         * Is the server overloaded? Running 12345ms or 246 ticks behind
         */
        if !message.starts_with("Can't keep up!") {
            return;
        }

        let milliseconds = parse_lag_ms(message);
        let ticks = parse_lag_ticks(message);

        let Some(milliseconds) = milliseconds else {
            return;
        };

        self.state.performance.last_lag_ms = Some(milliseconds);
        self.state.performance.last_lag_ticks = ticks;

        events.push(Parsed::Performance(PerformanceEvent::Lagging {
            milliseconds,
            ticks,
        }));
    }

    fn parse_world(&mut self, log: &Log, events: &mut Vec<Parsed>) {
        let message = log.message.as_str();

        if message == "Saving the game" {
            self.state.world.saving = true;

            events.push(Parsed::World(WorldEvent::Saving));

            return;
        }

        /*
         * 不同 MC 版本 / loader 的保存日志差异比较大。
         *
         * 这里暂时只处理明确的 Saving the game。
         */
        if message == "Saved the game" {
            self.state.world.saving = false;

            events.push(Parsed::World(WorldEvent::SaveFinished));
        }
    }

    /// 将 parser 自己转换成一个 Stream。
    ///
    /// 注意这里是 `self` by-value：
    /// parser 的状态会被 stream 接管。
    pub fn stream<S>(self, input: S) -> impl Stream<Item = Parsed>
    where
        S: Stream<Item = String>,
    {
        Box::pin(ParserStream {
            parser: self,
            input: Box::pin(input),
            pending: VecDeque::new(),
            finished: false,
        })
    }
}

struct ParserStream<S> {
    parser: LogsParser,
    input: Pin<Box<S>>,
    pending: VecDeque<Parsed>,
    finished: bool,
}

impl<S> Stream for ParserStream<S>
where
    S: Stream<Item = String>,
{
    type Item = Parsed;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use std::task::Poll;

        if let Some(item) = self.pending.pop_front() {
            return Poll::Ready(Some(item));
        }

        loop {
            match self.input.as_mut().poll_next(cx) {
                Poll::Ready(Some(chunk)) => {
                    let parsed = self.parser.update(chunk);
                    self.pending.extend(parsed);

                    if let Some(item) = self.pending.pop_front() {
                        return Poll::Ready(Some(item));
                    }
                }

                Poll::Ready(None) => {
                    if self.finished {
                        return Poll::Ready(None);
                    }

                    self.finished = true;
                    let parsed = self.parser.finish();
                    self.pending.extend(parsed);

                    if let Some(item) = self.pending.pop_front() {
                        return Poll::Ready(Some(item));
                    }

                    return Poll::Ready(None);
                }

                Poll::Pending => {
                    return Poll::Pending;
                }
            }
        }
    }
}

fn parse_log_line(line: &str) -> Option<Log> {
    /*
     * Vanilla：
     *
     * [12:34:56] [Server thread/INFO]: Hello
     * [12:34:56] [main/WARN]: Hello
     *
     * 这里只解析 Minecraft 最常见的格式。
     */

    let line = line.trim();

    let line = line.strip_prefix('[')?;

    let (time, rest) = line.split_once("] [")?;

    let (thread, rest) = rest.split_once('/')?;

    let (level, message) = rest.split_once("]: ")?;

    Some(Log {
        time: parse_time(time)?,
        thread: thread.to_owned(),
        level: parse_level(level)?,
        message: message.to_owned(),
    })
}

fn parse_time(input: &str) -> Option<LogTime> {
    let (hour, rest) = input.split_once(':')?;
    let (minute, rest) = rest.split_once(':')?;

    let (second, millis) = match rest.split_once('.') {
        Some((second, millis)) => {
            let millis = match millis.len() {
                1 => millis.parse::<u16>().ok()?.checked_mul(100)?,
                2 => millis.parse::<u16>().ok()?.checked_mul(10)?,
                3 => millis.parse::<u16>().ok()?,
                _ => return None,
            };

            (second, Some(millis))
        }

        None => (rest, None),
    };

    Some(LogTime {
        hour: hour.parse().ok()?,
        minute: minute.parse().ok()?,
        second: second.parse().ok()?,
        millis,
    })
}

fn parse_level(input: &str) -> Option<Level> {
    match input {
        "TRACE" => Some(Level::Trace),
        "DEBUG" => Some(Level::Debug),
        "INFO" => Some(Level::Info),
        "WARN" | "WARNING" => Some(Level::Warn),
        "ERROR" | "SEVERE" => Some(Level::Error),
        _ => None,
    }
}

fn parse_lag_ms(input: &str) -> Option<u64> {
    input
        .split_whitespace()
        .find_map(|word| word.strip_suffix("ms"))
        .and_then(|value| value.parse().ok())
}

fn parse_lag_ticks(input: &str) -> Option<u64> {
    let index = input.find("ticks behind")?;

    input[..index]
        .split_whitespace()
        .last()
        .and_then(|value| value.parse().ok())
}

fn parse_disconnect_player(input: &str) -> Option<Option<&str>> {
    /*
     * 例如：
     *
     * Disconnecting com.mojang.authlib.GameProfile@xxxx[
     *     id=...,
     *     name=Steve,
     *     ...
     * ]
     */

    let start = input.find("name=")? + "name=".len();

    let rest = &input[start..];

    let end = rest
        .find(',')
        .or_else(|| rest.find(']'))
        .unwrap_or(rest.len());

    let player = &rest[..end];

    if player.is_empty() {
        Some(None)
    } else {
        Some(Some(player))
    }
}

use crate::error::Error;
use crate::parsers::options::Options;
use futures::io::BufReader;
use futures::{AsyncBufReadExt, AsyncReadExt, AsyncSeekExt, AsyncWriteExt, StreamExt};
use std::borrow::Cow;
use std::path::Path;

pub struct OptionEditor {
    file: async_fs::File,
    content: Options,
}

impl OptionEditor {
    pub async fn open(path: impl AsRef<Path>) -> crate::error::Result<OptionEditor> {
        let mut file = async_fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .await?;

        let mut content = String::new();
        file.read_to_string(&mut content).await?;
        let content = content
            .parse()
            .map_err(|_| Error::other("Invalid options"))?;

        Ok(OptionEditor { file, content })
    }

    pub async fn edit<F, R>(&mut self, edit: F) -> crate::error::Result<R>
    where
        F: AsyncFnOnce(&mut Cow<'_, Options>) -> R,
    {
        let mut content = Cow::Borrowed(&self.content);

        let ret = edit(&mut content).await;

        let diff = self.content.diff(&content);

        if !diff.is_empty() {
            let new = content.into_owned();
            self.apply(diff).await?;

            // apply 成功后再更新内存中的配置
            self.content = new;
        }

        Ok(ret)
    }

    async fn apply(&mut self, mut diff: Vec<(String, String)>) -> crate::error::Result<()> {
        self.file.seek(std::io::SeekFrom::Start(0)).await?;

        let mut lines = BufReader::new(&mut self.file).lines();
        let mut output = String::new();

        while let Some(line) = lines.next().await {
            let line = line?;

            if let Some(key) = parse_config_line(&line).map(|(key, _)| key)
                && let Some(pos) = diff.iter().position(|(k, _)| k == key)
            {
                {
                    let value = &diff[pos].1;

                    let colon = line
                        .find(':')
                        .expect("parse_config_key returned Some without ':'");

                    // 保留 key 和 ':' 之前的所有格式，
                    // 只替换 ':' 后面的内容。
                    output.push_str(&line[..=colon]);
                    output.push(' ');
                    output.push_str(value);
                    output.push('\n');

                    diff.swap_remove(pos);
                    continue;
                }
            }

            // 未修改的行原样保留
            output.push_str(&line);
            output.push('\n');
        }

        // 文件中不存在的 key 追加到末尾
        for (key, value) in diff {
            output.push_str(&key);
            output.push_str(": ");
            output.push_str(&value);
            output.push('\n');
        }

        // 清空原文件并重新写入
        self.file.set_len(0).await?;
        self.file.seek(std::io::SeekFrom::Start(0)).await?;
        self.file.write_all(output.as_bytes()).await?;
        self.file.flush().await?;

        Ok(())
    }
}

// 解析一行配置，返回 `(key, value)`。
//
// 无法确定是配置的行会被忽略。
/// Parses one configuration line and returns `(key, value)`.
///
/// Lines that cannot be identified as configuration are ignored.
pub(super) fn parse_config_line(line: &str) -> Option<(&str, &str)> {
    let line = line.trim();

    // 空行 / 注释
    if line.is_empty() || line.starts_with(['#', ';']) {
        return None;
    }

    let (key, value) = line.split_once(':')?;

    let key = key.trim();

    if key.is_empty() {
        return None;
    }

    Some((key, value.trim()))
}

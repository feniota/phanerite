use crate::error::Result;
use crate::instance::instance_info::VersionType;
use crate::instance::{Instance, find_manifest};
use crate::storage::Storage;
use futures::Stream;
use futures::StreamExt;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

/// 简化的版本清单
#[derive(Debug, Clone, Deserialize)]
pub struct SimpleInfo {
    // 基本信息，构造依赖树需要
    pub id: String,
    pub inherits_from: Option<String>,
    // 额外信息
    pub version_type: VersionType,
}

#[derive(Debug, Clone)]
pub struct InfoForest {
    pub forest: Vec<InfoNode>,
}

#[derive(Debug, Clone)]
pub struct InfoNode {
    pub info: SimpleInfo,
    pub children: Vec<InfoNode>,
}

pub struct Iter<'a> {
    stack: Vec<&'a InfoNode>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a SimpleInfo;
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        self.stack.extend(node.children.iter().rev());
        Some(&node.info)
    }
}

impl InfoForest {
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            stack: self.forest.iter().rev().collect(),
        }
    }
    /// 构造依赖森林
    /// 并且去环（会有什么人构造出环形的继承实例来炸启动器？
    pub async fn build(stream: impl Stream<Item = SimpleInfo>) -> InfoForest {
        let infos: Vec<_> = stream.collect().await;

        let map: HashMap<String, SimpleInfo> =
            infos.into_iter().map(|x| (x.id.clone(), x)).collect();

        let mut state = HashMap::<String, u8>::new();
        let mut cycle_nodes = HashSet::<String>::new();

        fn dfs(
            id: &str,
            map: &HashMap<String, SimpleInfo>,
            state: &mut HashMap<String, u8>,
            stack: &mut Vec<String>,
            cycle_nodes: &mut HashSet<String>,
        ) {
            match state.get(id).copied() {
                Some(1) => {
                    // 找到环
                    if let Some(pos) = stack.iter().position(|x| x == id) {
                        for node in &stack[pos..] {
                            cycle_nodes.insert(node.clone());
                        }
                    }
                    return;
                }
                Some(2) => return,
                _ => {}
            }

            state.insert(id.to_string(), 1);
            stack.push(id.to_string());

            if let Some(info) = map.get(id)
                && let Some(parent) = &info.inherits_from
                && map.contains_key(parent)
            {
                dfs(parent, map, state, stack, cycle_nodes);
            }

            stack.pop();
            state.insert(id.to_string(), 2);
        }

        for id in map.keys() {
            dfs(id, &map, &mut state, &mut Vec::new(), &mut cycle_nodes);
        }

        // 建立 parent -> children
        let mut children: HashMap<String, Vec<String>> = HashMap::new();

        for info in map.values() {
            if cycle_nodes.contains(&info.id) {
                continue;
            }

            if let Some(parent) = &info.inherits_from
                && !cycle_nodes.contains(parent)
            {
                children
                    .entry(parent.clone())
                    .or_default()
                    .push(info.id.clone());
            }
        }

        fn build_node(
            id: &str,
            map: &HashMap<String, SimpleInfo>,
            children: &HashMap<String, Vec<String>>,
        ) -> InfoNode {
            InfoNode {
                info: map[id].clone(),
                children: children
                    .get(id)
                    .into_iter()
                    .flatten()
                    .map(|x| build_node(x, map, children))
                    .collect(),
            }
        }

        // 没有父节点的就是根
        Self {
            forest: map
                .values()
                .filter(|x| {
                    !cycle_nodes.contains(&x.id)
                        && x.inherits_from
                            .as_ref()
                            .is_none_or(|p| cycle_nodes.contains(p) || !map.contains_key(p))
                })
                .map(|x| build_node(&x.id, &map, &children))
                .collect(),
        }
    }
}

impl Instance {
    /// 简要列出实例列表
    pub async fn list(storage: &Storage) -> Result<InfoForest> {
        let stream = async_fs::read_dir(storage.versions_dir())
            .await?
            .filter_map(async |x| x.ok())
            .filter_map(async |x| {
                find_manifest(x.file_name().to_string_lossy(), &x.path())
                    .await
                    .ok()
            })
            .filter_map(async |x| serde_json::from_slice::<SimpleInfo>(&x).ok());
        Ok(InfoForest::build(stream).await)
    }
}

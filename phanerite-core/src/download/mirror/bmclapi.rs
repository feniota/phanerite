use crate::download::mirror::Mirror;
use crate::download::task::DownloadTask;

pub struct Bmclapi;

impl Mirror for Bmclapi {
    const NAME: &str = "BMCLAPI";
    const ATTRIBUTION: &str = "BMCLAPI - Minecraft mirror service provided by bangbang93";
    const NOTICE: &str = r#"
BMCLAPI 使用声明：

1. BMCLAPI 下的所有文件，除 BMCLAPI 本身的源码之外，归源站点所有。

2. BMCLAPI 会尽量保证文件的完整性、有效性和实时性，对于使用 BMCLAPI 带来的一切纠纷，与 BMCLAPI 无关。

3. BMCLAPI 和 BMCL 不同，属于非开源项目。

4. 所有使用 BMCLAPI 的程序必须在下载界面或其他可视部分标明来源。

5. 禁止在 BMCLAPI 上二次封装其他协议。
"#;
    fn resolve(&self, task: &mut DownloadTask) {
        let url = &mut task.url;

        if let Some(path) = url.strip_prefix("https://libraries.minecraft.net/") {
            *url = format!("https://bmclapi2.bangbang93.com/maven/{path}");
        } else if let Some(path) = url.strip_prefix("https://resources.download.minecraft.net/") {
            *url = format!("https://bmclapi2.bangbang93.com/assets/{path}");
        } else if let Some(path) = url.strip_prefix("https://piston-meta.mojang.com/") {
            *url = format!("https://bmclapi2.bangbang93.com/{path}");
        } else if let Some(path) = url.strip_prefix("https://launchermeta.mojang.com/") {
            *url = format!("https://bmclapi2.bangbang93.com/{path}");
        }
    }
}

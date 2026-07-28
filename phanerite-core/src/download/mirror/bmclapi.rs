use crate::download::mirror::Mirror;
use url::Url;

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

    fn resolve(&self, url: &mut Url) {
        match url.host_str() {
            Some("launchermeta.mojang.com") | Some("launcher.mojang.com") => {
                url.set_host(Some("bmclapi2.bangbang93.com")).unwrap();
            }

            Some("resources.download.minecraft.net") => {
                let path = url.path().to_string();

                url.set_host(Some("bmclapi2.bangbang93.com")).unwrap();

                url.set_path(&format!("/assets{}", path));
            }

            Some("libraries.minecraft.net") => {
                let path = url.path().to_string();

                url.set_host(Some("bmclapi2.bangbang93.com")).unwrap();

                url.set_path(&format!("/maven{}", path));
            }

            Some("files.minecraftforge.net") => {
                let path = url.path().to_string();

                if path.starts_with("/maven") {
                    url.set_host(Some("bmclapi2.bangbang93.com")).unwrap();
                }
            }

            Some("authlib-injector.yushi.moe") => {
                let path = url.path().to_string();

                url.set_host(Some("bmclapi2.bangbang93.com")).unwrap();

                url.set_path(&format!("/mirrors/authlib-injector{}", path));
            }

            Some("meta.fabricmc.net") => {
                let path = url.path().to_string();

                url.set_host(Some("bmclapi2.bangbang93.com")).unwrap();

                url.set_path(&format!("/fabric-meta{}", path));
            }

            Some("maven.fabricmc.net")
            | Some("maven.neoforged.net")
            | Some("maven.quiltmc.org") => {
                let path = url.path().to_string();

                url.set_host(Some("bmclapi2.bangbang93.com")).unwrap();

                url.set_path(&format!("/maven{}", path));
            }

            _ => (),
        }
    }
}

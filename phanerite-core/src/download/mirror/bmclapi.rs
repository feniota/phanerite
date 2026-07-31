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
            // ----------------------------------------
            // Mojang version / json / runtime runtime
            // ----------------------------------------
            Some("launchermeta.mojang.com") | Some("launcher.mojang.com") => {
                url.set_scheme("https").ok();
                url.set_host(Some("bmclapi2.bangbang93.com")).unwrap();
            }

            // ----------------------------------------
            // Assets
            // http(s)://resources.download.minecraft.net
            // ->
            // https://bmclapi2.bangbang93.com/assets
            // ----------------------------------------
            Some("resources.download.minecraft.net") => {
                let path = url.path().to_string();

                url.set_scheme("https").ok();
                url.set_host(Some("bmclapi2.bangbang93.com")).unwrap();

                url.set_path(&format!("/assets{path}"));
            }

            // ----------------------------------------
            // Minecraft Libraries
            // ----------------------------------------
            Some("libraries.minecraft.net") => {
                let path = url.path().to_string();

                url.set_scheme("https").ok();
                url.set_host(Some("bmclapi2.bangbang93.com")).unwrap();

                url.set_path(&format!("/maven{path}"));
            }

            // ----------------------------------------
            // Forge
            // https://files.minecraftforge.net/maven
            // ->
            // https://bmclapi2.bangbang93.com/maven
            // ----------------------------------------
            Some("files.minecraftforge.net") => {
                let path = url.path().to_string();

                if let Some(path) = path.strip_prefix("/maven") {
                    url.set_scheme("https").ok();
                    url.set_host(Some("bmclapi2.bangbang93.com")).unwrap();

                    url.set_path(&format!("/maven{path}"));
                }
            }

            // ----------------------------------------
            // LiteLoader
            // http://dl.liteloader.com/versions/versions.json
            // ->
            // https://bmclapi.bangbang93.com/maven/com/mumfrey/liteloader/versions.json
            // ----------------------------------------
            Some("dl.liteloader.com") => {
                let path = url.path().to_string();

                url.set_scheme("https").ok();
                url.set_host(Some("bmclapi.bangbang93.com")).unwrap();

                url.set_path(&format!("/maven{path}"));
            }

            // ----------------------------------------
            // Authlib Injector
            // ----------------------------------------
            Some("authlib-injector.yushi.moe") => {
                let path = url.path().to_string();

                url.set_scheme("https").ok();
                url.set_host(Some("bmclapi2.bangbang93.com")).unwrap();

                url.set_path(&format!("/mirrors/authlib-injector{path}"));
            }

            // ----------------------------------------
            // Fabric meta
            // ----------------------------------------
            Some("meta.fabricmc.net") => {
                let path = url.path().to_string();

                url.set_scheme("https").ok();
                url.set_host(Some("bmclapi2.bangbang93.com")).unwrap();

                url.set_path(&format!("/mod_loader-meta{path}"));
            }

            // Fabric Maven
            Some("maven.fabricmc.net") => {
                let path = url.path().to_string();

                url.set_scheme("https").ok();
                url.set_host(Some("bmclapi2.bangbang93.com")).unwrap();

                url.set_path(&format!("/maven{path}"));
            }

            // ----------------------------------------
            // NeoForge
            // /releases 需要去掉
            // ----------------------------------------
            Some("maven.neoforged.net") => {
                let path = url
                    .path()
                    .strip_prefix("/releases")
                    .unwrap_or(url.path())
                    .to_string();

                url.set_scheme("https").ok();
                url.set_host(Some("bmclapi2.bangbang93.com")).unwrap();

                url.set_path(&format!("/maven{path}"));
            }

            // ----------------------------------------
            // Quilt Maven
            // /repository/release 需要去掉
            // ----------------------------------------
            Some("maven.quiltmc.org") => {
                let path = url
                    .path()
                    .strip_prefix("/repository/release")
                    .unwrap_or("")
                    .to_string();

                url.set_scheme("https").ok();
                url.set_host(Some("bmclapi2.bangbang93.com")).unwrap();

                url.set_path(&format!("/maven{path}"));
            }

            // Quilt Meta
            Some("meta.quiltmc.org") => {
                let path = url.path().to_string();

                url.set_scheme("https").ok();
                url.set_host(Some("bmclapi2.bangbang93.com")).unwrap();

                url.set_path(&format!("/quilt-meta{path}"));
            }

            _ => {}
        }
    }
}

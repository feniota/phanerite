use crate::download::mirror::Mirror;
use url::Url;

/// Granodiorite — Minecraft mirror service provided by Ferris Love.
///
/// Granodiorite is part of the Phenocryst system, running on
/// Cloudflare Workers, caching Minecraft resources in R2.
/// https://github.com/feniota/granodiorite
pub struct Granodiorite;

impl Mirror for Granodiorite {
    const NAME: &str = "Granodiorite";
    const ATTRIBUTION: &str = "Granodiorite - Minecraft mirror service provided by Ferris Love";
    const NOTICE: &str = "";

    fn resolve(&self, url: &mut Url) {
        match url.host_str() {
            // ── Minecraft 原版 ──
            Some(
                "launchermeta.mojang.com"
                | "launcher.mojang.com"
                | "piston-meta.mojang.com"
                | "piston-data.mojang.com",
            ) => {
                url.set_host(Some("granodiorite.ferris.love")).unwrap();
            }

            Some("resources.download.minecraft.net") => {
                let path = url.path().to_string();

                url.set_host(Some("granodiorite.ferris.love")).unwrap();
                url.set_path(&format!("/assets{}", path));
            }

            Some("libraries.minecraft.net") => {
                let path = url.path().to_string();

                url.set_host(Some("granodiorite.ferris.love")).unwrap();
                url.set_path(&format!("/libraries{}", path));
            }

            // ── Fabric ──
            Some("meta.fabricmc.net") => {
                let path = url.path().to_string();

                url.set_host(Some("granodiorite.ferris.love")).unwrap();
                url.set_path(&format!("/fabric-meta{}", path));
            }

            Some("maven.fabricmc.net") => {
                let path = url.path().to_string();

                url.set_host(Some("granodiorite.ferris.love")).unwrap();
                url.set_path(&format!("/maven/fabric{}", path));
            }

            // ── NeoForge ──
            Some("maven.neoforged.net") => {
                let path = url.path().to_string();

                url.set_host(Some("granodiorite.ferris.love")).unwrap();
                url.set_path(&format!("/maven/neoforge{}", path));
            }

            // ── Forge ──
            Some("maven.minecraftforge.net") => {
                let path = url.path().to_string();

                url.set_host(Some("granodiorite.ferris.love")).unwrap();
                url.set_path(&format!("/maven/forge{}", path));
            }

            Some("files.minecraftforge.net") => {
                let path = url.path().to_string();

                if path.starts_with("/maven") {
                    url.set_host(Some("granodiorite.ferris.love")).unwrap();
                    url.set_path(&format!("/maven/forge-legacy{}", &path[6..]));
                }
            }

            _ => (),
        }
    }
}

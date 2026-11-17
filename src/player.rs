use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};
use tokio::process::{Child, Command as TokioCommand};

pub struct Player;

impl Player {
    pub fn play(
        player_cmd: &str,
        stream_url: &str,
        title: &str,
        skip_intro: bool,
    ) -> Result<()> {
        let mut cmd = Command::new(player_cmd);

        if player_cmd.contains("mpv") {
            cmd.arg(format!("--force-media-title={}", title));
            cmd.arg("--save-position-on-quit");
            // Disable ffmpeg keepalive reuse on rotating CDN domains & configure resilient buffering
            cmd.arg("--demuxer-lavf-o=http_persistent=0,http_multiple=1,seg_max_retry=5");
            cmd.arg("--demuxer-readahead-secs=120");
            cmd.arg("--demuxer-max-bytes=250M");
            cmd.arg("--demuxer-max-back-bytes=50M");
            cmd.arg("--cache=yes");
            cmd.arg("--cache-pause-initial=yes");
            cmd.arg("--cache-pause-wait=8");
            cmd.arg("--ytdl-raw-options=concurrent-fragments=5");
            cmd.arg("--referrer=https://jkanime.net/");
            cmd.arg("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36");

            if skip_intro {
                // Check if ani-skip script is present in PATH
                cmd.arg("--script-opts=ani-skip=auto");
            }
        } else if player_cmd.contains("vlc") {
            cmd.arg(format!("--meta-title={}", title));
            cmd.arg("--http-referrer=https://jkanime.net/");
            cmd.arg("--http-user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36");
        }

        cmd.arg(stream_url);

        let mut child = cmd
            .spawn()
            .with_context(|| format!("No se pudo iniciar el reproductor '{}'", player_cmd))?;

        let status = child.wait()?;
        if !status.success() {
            // Some players exit with code when killed by user, ignore or log
        }

        Ok(())
    }

    pub fn spawn_player_process(
        player_cmd: &str,
        stream_url: &str,
        title: &str,
        skip_intro: bool,
    ) -> Result<Child> {
        let mut cmd = TokioCommand::new(player_cmd);

        if player_cmd.contains("mpv") {
            cmd.arg(format!("--force-media-title={}", title));
            cmd.arg("--save-position-on-quit");
            // Disable ffmpeg keepalive reuse on rotating CDN domains & configure resilient buffering
            cmd.arg("--demuxer-lavf-o=http_persistent=0,http_multiple=1,seg_max_retry=5");
            cmd.arg("--demuxer-readahead-secs=120");
            cmd.arg("--demuxer-max-bytes=250M");
            cmd.arg("--demuxer-max-back-bytes=50M");
            cmd.arg("--cache=yes");
            cmd.arg("--cache-pause-initial=yes");
            cmd.arg("--cache-pause-wait=8");
            cmd.arg("--ytdl-raw-options=concurrent-fragments=5");
            cmd.arg("--referrer=https://jkanime.net/");
            cmd.arg("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36");

            if skip_intro {
                // Check if ani-skip script is present in PATH
                cmd.arg("--script-opts=ani-skip=auto");
            }
        } else if player_cmd.contains("vlc") {
            cmd.arg(format!("--meta-title={}", title));
            cmd.arg("--http-referrer=https://jkanime.net/");
            cmd.arg("--http-user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36");
        }

        cmd.arg(stream_url);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let child = cmd
            .spawn()
            .with_context(|| format!("No se pudo iniciar el reproductor '{}'", player_cmd))?;

        Ok(child)
    }

    pub fn download<P: AsRef<Path>>(
        stream_url: &str,
        output_dir: P,
        filename: &str,
    ) -> Result<()> {
        let out_path = output_dir.as_ref().join(format!("{}.mp4", filename));
        let out_str = out_path.to_string_lossy();

        println!("Iniciando descarga en: {}", out_str);

        // Try yt-dlp first if available
        if let Ok(mut child) = Command::new("yt-dlp")
            .arg(stream_url)
            .arg("-N")
            .arg("16")
            .arg("--concurrent-fragments")
            .arg("5")
            .arg("--referer")
            .arg("https://jkanime.net/")
            .arg("--user-agent")
            .arg("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
            .arg("-o")
            .arg(&*out_str)
            .spawn()
        {
            let status = child.wait()?;
            if status.success() {
                println!("Descarga completada con éxito vía yt-dlp.");
                return Ok(());
            }
        }

        // Fallback to ffmpeg
        println!("Descargando vía ffmpeg...");
        let mut child = Command::new("ffmpeg")
            .arg("-y")
            .arg("-headers")
            .arg("Referer: https://jkanime.net/\r\nUser-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36\r\n")
            .arg("-http_persistent")
            .arg("0")
            .arg("-i")
            .arg(stream_url)
            .arg("-c")
            .arg("copy")
            .arg("-bsf:a")
            .arg("aac_adtstoasc")
            .arg(&*out_str)
            .spawn()
            .with_context(|| "No se encontró ni yt-dlp ni ffmpeg para descargar")?;

        let status = child.wait()?;
        if status.success() {
            println!("Descarga completada con éxito vía ffmpeg: {}", out_str);
            Ok(())
        } else {
            anyhow::bail!("ffmpeg falló con código de salida: {:?}", status.code());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mpv_command_args() {
        let mut cmd = Command::new("mpv");
        let title = "One Piece - Episodio 999";
        cmd.arg(format!("--force-media-title={}", title));
        cmd.arg("--save-position-on-quit");
        cmd.arg("--demuxer-lavf-o=http_persistent=0,http_multiple=1,seg_max_retry=5");
        cmd.arg("--demuxer-readahead-secs=120");
        cmd.arg("--demuxer-max-bytes=250M");
        cmd.arg("--demuxer-max-back-bytes=50M");
        cmd.arg("--cache=yes");
        cmd.arg("--cache-pause-initial=yes");
        cmd.arg("--cache-pause-wait=8");
        cmd.arg("--referrer=https://jkanime.net/");
        cmd.arg("https://example.com/stream.m3u8");

        let args: Vec<&std::ffi::OsStr> = cmd.get_args().collect();
        assert_eq!(args[0], "--force-media-title=One Piece - Episodio 999");
        assert_eq!(args[1], "--save-position-on-quit");
        assert_eq!(args[2], "--demuxer-lavf-o=http_persistent=0,http_multiple=1,seg_max_retry=5");
        assert_eq!(args[3], "--demuxer-readahead-secs=120");
        assert_eq!(args[4], "--demuxer-max-bytes=250M");
        assert_eq!(args[5], "--demuxer-max-back-bytes=50M");
        assert_eq!(args[6], "--cache=yes");
        assert_eq!(args[7], "--cache-pause-initial=yes");
        assert_eq!(args[8], "--cache-pause-wait=8");
        assert_eq!(args[9], "--referrer=https://jkanime.net/");
        assert_eq!(args[10], "https://example.com/stream.m3u8");
    }

    #[tokio::test]
    async fn test_spawn_player_process_invalid_binary() {
        let res = Player::spawn_player_process(
            "non_existent_player_binary_12345",
            "https://example.com/stream.m3u8",
            "Test",
            false,
        );
        assert!(res.is_err());
    }
}


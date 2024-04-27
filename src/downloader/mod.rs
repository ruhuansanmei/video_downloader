mod downloader {
    use core::fmt;
    use std::{
        collections::HashMap,
        error::Error,
        fmt::Display,
        fs::File,
        io::{BufWriter, Bytes, Write},
        path::Path,
        string, vec,
    };

    use essi_ffmpeg::FFmpeg;
    use reqwest::header;

    #[derive(Debug)]
    struct DownloadError<'a> {
        msg: &'a str,
    }

    impl<'a> Display for DownloadError<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "下载出错: {}", self.msg)
        }
    }

    impl<'a> Error for DownloadError<'a> {}

    /**
     * 下载一个文件
     */
    async fn fn_download_file(
        url: &str,
        file_name: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let resp = reqwest::get(url).await?;
        if (resp.status() == 403) {
            return Err(Box::new(DownloadError {
                msg: "权限有问题，请获取最新链接",
            }));
        }
        if resp.status() == 404 {
            return Ok(false);
        }
        let bbs = resp.bytes().await?;
        if bbs.len() == 0 {
            return Ok(false);
        }
        let mut file = File::create(Path::new(file_name))?;
        file.write_all(&bbs)?;
        Ok(true)
    }

    /**
     * 根据前缀，后缀，数字长度构造下载url
     */
    fn make_url(prefix: &str, num: u32, url_suffix: &str, num_length: usize) -> String {
        format!(
            "{prefix}{number:>0width$}{url_suffix}",
            prefix = prefix,
            number = num,
            width = num_length,
            url_suffix = url_suffix
        )
    }

    /**
     * 一个个下载
     */
    async fn download_one_by_one(
        prefix: &str,
        num: u32,
        url_suffix: &str,
        num_length: usize,
        dir_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut num_p = num;
        let mut flag = true;
        while flag {
            let url = make_url(prefix, num_p, url_suffix, num_length);
            let cont = fn_download_file(
                &url,
                &format!(
                    "{dir_name}/{file_name}.ts",
                    dir_name = dir_name,
                    file_name = num_p
                ),
            )
            .await?;
            if !cont {
                flag = false;
            }
            num_p = num_p + 1;
        }
        Ok(())
    }

    /**
     * 合并视频
     */
    async fn merge_video(dir_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let dirs = Path::new(dir_name).read_dir()?;
        let file_names = dirs
            .flatten()
            .map(|x| format!("{file_name}", file_name = x.file_name().to_str().unwrap()));
        let mut files = vec::Vec::from_iter(file_names);
        files.sort_by(|a, b| {
            let parts = a.split_once('.').unwrap();
            let b_parts = b.split_once('.').unwrap();
            (parts.0.parse::<u32>().unwrap() as u32)
                .cmp(&(b_parts.0.parse::<u32>().unwrap() as u32))
        });
        let mut file_list_content: Vec<String> = vec![];

        for ele in files {
            file_list_content.push(format!(
                "file   '{dir_name}/{file_name}'",
                dir_name = dir_name,
                file_name = ele
            ))
        }

        write!(
            File::create("filelist.txt").unwrap(),
            "{}",
            file_list_content.join("\r\n")
        )
        .unwrap();

        // Automatically download FFmpeg if not found
        if let Some((handle, mut progress)) = FFmpeg::auto_download().await.unwrap() {
            tokio::spawn(async move {
                while let Some(state) = progress.recv().await {
                    println!("{:?}", state);
                }
            });

            handle.await.unwrap().unwrap();
        } else {
            println!("FFmpeg is downloaded, using existing installation");
        }

        // Build and execute an FFmpeg command
        let mut ffmpeg = FFmpeg::new()
            .stderr(std::process::Stdio::inherit())
            .args(vec![
                "-y",
                "-f",
                "concat",
                "-safe",
                "0",
                "-i",
                "filelist.txt",
                "-c",
                "copy",
                "pp.mp4",
            ])
            .start()
            .unwrap();

        ffmpeg.wait().unwrap();

        Ok(())
    }

    async fn merge_webm_and_video(
        mp4_path: &str,
        webm_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Automatically download FFmpeg if not found
        if let Some((handle, mut progress)) = FFmpeg::auto_download().await.unwrap() {
            tokio::spawn(async move {
                while let Some(state) = progress.recv().await {
                    println!("{:?}", state);
                }
            });

            handle.await.unwrap().unwrap();
        } else {
            println!("FFmpeg is downloaded, using existing installation");
        }
        // Build and execute an FFmpeg command
        let mut ffmpeg = FFmpeg::new()
            .stderr(std::process::Stdio::inherit())
            .args(vec![
                "-y",
                "-i",
                mp4_path,
                "-i",
                webm_path,
                "-c-v",
                "copy",
                "-c-a",
                "aac",
                "./merged.mp4",
            ])
            .start()
            .unwrap();
        ffmpeg.wait().unwrap();
        Ok(())
    }

    // 构造下载url
    #[test]
    fn test_make_url() {
        println!("{}", make_url("prefix", 1, "suffix", 3))
    }

    // 测试下载
    #[tokio::test]
    async fn test_download() {
        let result = fn_download_file("http://baidu.com", "./aaa").await.unwrap();
        println!("{}", result)
    }

    // 将文件下载到一个文件夹中
    #[tokio::test]
    async fn test_one_by_one_make_url() {
        // 0000000
        // 下载链接前缀
        let prefix = "https://play-tx-recpub.douyucdn2.cn/live/high_live-52876rjEzXV54VQi--20240427165720/transcode_live-52876rjEzXV54VQi--20240427165720_1446613_";
        // 下载链接后缀
        let suffix = ".ts?cdn=tx&ct=web_share&d=MTYzOTEzMDJ8aDVvdXRlcnBsYXllcg%3D%3D&exper=0&nlimit=5&pt=2&sign=b9a093251edf2299b3ac25459e52d190&tlink=662d1664&tplay=662da304&u=62649895&us=MTYzOTEzMDJ8aDVvdXRlcnBsYXllcg%3D%3D&vid=41062478";
        // 下载链接中间编码数字位数
        let num_length = 7;
        let dir = "./zzz";
        download_one_by_one(&prefix, 0, &suffix, num_length, dir)
            .await
            .unwrap();
    }
    // 合并文件
    #[tokio::test]
    async fn test_merge_video() {
        merge_video("./zzz").await.unwrap();
    }

    // 合并音频文件和视频文件
    #[tokio::test]
    async fn test_merge_webm_and_video() {
        merge_webm_and_video("./mp4.mp4", "./voice.mp3").await.unwrap();
    }
}

# 目标：
    本项目为一个视频下载器，可以通过视频链接下载视频到一个文件夹中，并进行合并
    
# 安装：
## 1.安装vcpkg
https://github.com/Microsoft/vcpkg


# 用法

直接执行src/downloader/mod.rs中的测试，就可以使用功能：
主要功能：
1.将文件下载到一个文件夹中 `download_one_by_one`
2.合并文件 `merge_video`

# TODO
合并音频和视频
```

(defn merge-av
  "合并音频和视频"
  [dirname ^Path mp4 ^Path webm]
  (try (ffmpeg! :y :i  mp4 :i  webm  :c:v "copy" :c:a "aac"  (str dirname "/" "pp.mp4") )
       (catch Throwable e
         (.printStackTrace e)))
  )

```

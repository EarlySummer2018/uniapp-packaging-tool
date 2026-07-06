# 基础及非三方模块 Pod 集成

> 官方文档：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/common.html
> 适用版本：HBuilderX 5.13+

## Pod subspec

| 功能 | Pod subspec | 说明 |
| --- | --- | --- |
| 基础运行模块 | `Core` | 必须保留 |
| 加速度计 | `Accelerometer` | 加速度传感器 |
| 音频 | `Audio` | 录音、音频相关能力 |
| 相机/相册 | `CameraGallery` | 拍照、选择图片/视频 |
| 通讯录 | `Contacts` | 通讯录能力 |
| 文件系统 | `File` | 文件读写 |
| 短彩邮件消息 | `Messaging` | 系统短信、邮件等消息能力 |
| 屏幕方向 | `Orientation` | 设备方向 |
| 距离传感器 | `Proximity` | 距离传感器 |
| 网络请求 | `XMLHttpRequest` | XHR 网络请求 |
| 压缩解压 | `Zip` | zip 能力 |
| 扫码 | `Barcode` | 条码/二维码扫描 |
| Canvas | `Canvas` | Canvas 能力 |
| 视频播放 | `Video` | video 播放能力 |
| 指纹识别 | `Fingerprint` | Touch ID |
| Face ID | `FaceId` | Face ID |
| 蓝牙 | `BlueTooth` | 蓝牙能力 |
| SQLite | `Sqlite` | SQLite 数据库 |
| iBeacon | `IBeacon` | iBeacon |
| 日志 | `Log` | 输出 `console.log()` 等日志 |

## Podfile 示例

```ruby
uniapp_subspecs = [
  'Core',
  'CameraGallery',
  'Barcode',
  'Video',
  'Log',
]
```

## 注意事项

- `Core` 是基础运行模块，官方要求必须保留。
- 权限描述、URL Scheme、三方后台配置等不属于基础 Pod 自动完成的全部范围；涉及权限的能力仍需按官方模块说明检查 `Info.plist`。
- 下方旧版依赖表只用于手动集成或排查问题；5.13+ 优先使用本地 Pod。

# LivePusher（直播推流）（iOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：iOS (iPhone/iPad)
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/

---

iOS 直播推流模块基于腾讯直播 SDK（LiteAVSDK）实现。

## 需要引入的系统框架

| 框架 | 说明 |
|------|------|
| AVFoundation.framework | 音视频采集与播放 |
| Accelerate.framework | 加速框架 |
| AudioToolbox.framework | 音频工具 |
| VideoToolbox.framework | 硬件编码加速 |
| CoreMedia.framework | 核心媒体库 |
| CoreMotion.framework | 传感器数据（防抖） |
| OpenGLES.framework | OpenGL 渲染 |
| QuartzCore.framework | 图形渲染 |
| UIKit.framework | UI组件 |
| Foundation.framework | 基础框架 |
| libresolv.tbd | DNS解析 |
| libc++.tbd | C++ 运行时 |

## Info.plist 配置

```xml
<!-- 相机权限 -->
<key>NSCameraUsageDescription</key>
<string>我们需要使用摄像头来进行直播推流</string>

<!-- 麦克风权限 -->
<key>NSMicrophoneUsageDescription</key>
<string>我们需要使用麦克风来采集声音</string>

<!-- 网络权限 -->
<key>NSAppTransportSecurity</key>
<dict>
    <key>NSAllowsArbitraryLoads</key>
    <true/>
</dict>

<!-- 后台音频（可选） -->
<key>UIBackgroundModes</key>
<array>
    <string>audio</string>
</array>
```

## CocoaPods 依赖

```ruby
pod 'TXLiteAVSDK_Professional', '~> 11.x.x'  # 腾讯直播专业版
# 或者
pod 'TXLiteAVSDK_Enterprise', '~> 11.x.x'     # 企业版（功能更全）
```

## 需要拷贝的文件

| 路径 | 文件 |
|------|------|
| SDK/libs | `TXLiteAVSDK_Professional.framework` 或 `TXLiveSDK.framework` |

## Objective-C 代码初始化

```objc
#import <TXLiteAVSDK_Professional/TXLiteAVSDK.h>

// 初始化直播引擎
TXLivePushConfig *config = [[TXLivePushConfig alloc] init];
config.videoQuality = VIDEO_QUALITY_HIGH_DEFINITION;  // 高清画质
config.frontCamera = YES;                              // 默认前置摄像头
config.enableAudioPreview = YES;                       // 开启耳返

TXLivePush *livePush = [[TXLivePush alloc] initWithConfig:config];

// 设置推流地址
[livePush startPush:@"rtmp://你的推流地址/live/streamkey"];

// 开始预览
[livePush startPreview:self.previewView];
```

## dcloud_properties.xml 配置

```xml
<feature name="LivePusher" value="io.dcloud.media.live.LiveMediaFeatureImpl"/>
```

## ⚠️ 直播推流注意事项

1. **硬件要求**：直播推流对设备性能有一定要求，低端设备可能出现卡顿
2. **网络优化**：建议使用 CDN 推流，并根据网络状况动态调整码率
3. **美颜滤镜**：腾讯 SDK 内置美颜功能，可按需开启
4. **横竖屏切换**：需要处理好屏幕旋转逻辑
5. **后台限制**：iOS 对后台摄像头有限制，进入后台后需暂停推流

---

## 交叉引用

- 上一篇：[Speech（语音输入）](speech.md)
- 下一篇：[Statistic（统计）](statistic.md)
- 相关模块：[Speech（语音输入）](speech.md)（同样需要麦克风权限）、[uni-AD（广告）](uni-ad.md)

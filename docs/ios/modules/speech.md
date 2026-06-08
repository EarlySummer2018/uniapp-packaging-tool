# Speech（语音输入）（iOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：iOS (iPhone/iPad)
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/

---

iOS 语音识别模块支持百度语音和讯飞语音两种引擎。

## 5.1 百度语音

### 需要引入的系统框架

| 框架 | 说明 |
|------|------|
| AVFoundation.framework | 音频视频处理 |
| AudioToolbox.framework | 音频工具箱 |
| CFNetwork.framework | 网络通信 |
| CoreBluetooth.framework | 蓝牙（可选） |
| CoreLocation.framework | 定位（可选） |
| SystemConfiguration.framework | 系统配置 |
| Security.framework | 安全服务 |
| libc++.tbd | C++ 运行时 |

### Info.plist 配置

```xml
<!-- 麦克风权限 -->
<key>NSMicrophoneUsageDescription</key>
<string>我们需要使用麦克风来录制您的语音</string>

<!-- 网络权限（用于上传语音数据） -->
<key>NSAppTransportSecurity</key>
<dict>
    <key>NSAllowsArbitraryLoads</key>
    <true/>
</dict>

<!-- 百度语音配置 -->
<key>BDSpeechAPPID</key>
<string>%百度语音AppID%</string>
<key>BDSpeechAPIKey</key>
<string>%百度语音APIKey%</string>
<key>BDSpeechSecretKey</key>
<string>%百度语音SecretKey%</string>
```

### CocoaPods 依赖

```ruby
pod 'BDSpeechSDK', '~> 3.x.x'  # 百度语音识别SDK
```

### 需要拷贝的文件

| 路径 | 文件 |
|------|------|
| SDK/libs | `BDVoiceRecognitionClientSDK.framework` 等 |

### dcloud_properties.xml 配置

```xml
<feature name="Speech" value="io.dcloud.feature.speech.SpeechFeatureImpl">
    <module name="baidu" value="io.dcloud.feature.speech.BaiduSpeechEngine"/>
</feature>
```

## 5.2 讯飞语音

### 需要引入的系统框架

| 框架 | 说明 |
|------|------|
| AVFoundation.framework | 音频视频处理 |
| AudioToolbox.framework | 音频工具箱 |
| CoreTelephony.framework | 电话信息 |
| SystemConfiguration.framework | 系统配置 |
| Foundation.framework | 基础框架 |
| UIKit.framework | UI框架 |

### Info.plist 配置

```xml
<!-- 麦克风权限 -->
<key>NSMicrophoneUsageDescription</key>
<string>我们需要使用麦克风来录制您的语音</string>

<!-- 讯飞语音 AppID -->
<key>IFlySpeechAppID</key>
<string>%讯飞语音AppID%</string>
```

### CocoaPods 依赖

```ruby
pod 'iflyMSC', '~> 1.x.x'  # 讯飞语音SDK
```

### 需要拷贝的文件

| 路径 | 文件 |
|------|------|
| SDK/libs | `iflyMSC.framework` |

### dcloud_properties.xml 配置

```xml
<feature name="Speech" value="io.dcloud.feature.speech.SpeechFeatureImpl">
    <module name="iFly" value="io.dcloud.feature.speech.IflySpeechEngine"/>
</feature>
```

## ⚠️ iOS 语音识别注意事项

1. **权限重要性**：必须在 Info.plist 中声明 NSMicrophoneUsageDescription，否则会崩溃
2. **网络需求**：在线语音识别需要稳定的网络连接
3. **离线能力**：部分 SDK 支持离线语音识别，但需要下载离线资源包
4. **隐私合规**：录音前应告知用户并获得同意

---

## 交叉引用

- 上一篇：[Map（地图）](map.md)
- 下一篇：[LivePusher（直播推流）](livepusher.md)
- 相关模块：[LivePusher（直播推流）](livepusher.md)（同样需要音频权限）

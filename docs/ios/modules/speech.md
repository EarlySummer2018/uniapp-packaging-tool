# Speech（语音输入）（iOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：iOS (iPhone/iPad)
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/speech.html

---

## HBuilderX 5.13+ 本地 Pod 集成（推荐）

HBuilderX 5.13+ 推荐使用本地 Pod 集成语音模块。语音基础模块使用 `Speech`，百度语音使用 `Speech-Baidu`，讯飞语音使用 `Speech-Ifly`。

手动集成时再参考下方依赖表。

## 百度语音

### 将百度语音模块依赖库及资源添加到工程

| 依赖库 | 系统库 | 依赖资源 |
|---|---|---|
| liblibSpeech.a、libBaiduSpeechSDK.a、libbaiduSpeech.a | libc++.tbd、libz.tbd、libsqlite3.tbd、AudioToolbox.framework、AVFoundation.framework、CFNetwork.framework、CoreLocation.framework、CoreTelephony.framework、SystemConfiguration.framework、GLKit.framework | BDSClientEASRResources 文件夹里的资源文件 |

### 帐号配置

1. 首先到[百度语音官方网站](https://ai.baidu.com/tech/speech/asr)创建应用获取 appkey 等信息。

2. 打开 info.plist，并创建 `baiduspeech` 节点，填入自己帐号的信息，按照下图中的格式创建。

3. 把 BDSClientEASRResources 文件夹里的资源文件引入到工程里。

---

## 交叉引用

- 上一篇：[Map（地图）](map.md)
- 下一篇：[LivePusher（直播推流）](livepusher.md)
- 相关模块：[LivePusher（直播推流）](livepusher.md)（同样需要音频权限）

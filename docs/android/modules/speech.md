# Speech 语音输入（Android）

> **适用版本**：HBuilderX 5.0+
> **平台**：Android
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/

## 7. Speech（语音输入）

### 百度语音

#### 需要添加的文件

| 路径 | 文件名 |
|---|---|
| SDK\libs | `speech-release.aar`, `speech_baidu-release.aar` |

#### AndroidManifest.xml配置

**权限：**
```xml
<uses-permission android:name="android.permission.RECORD_AUDIO" />
<uses-permission android:name="android.permission.INTERNET" />
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
<uses-permission android:name="android.permission.ACCESS_WIFI_STATE" />
<uses-permission android:name="android.permission.CHANGE_NETWORK_STATE" />
<uses-permission android:name="android.permission.READ_PHONE_STATE" />
<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" />
```

**application节点下：**
```xml
<meta-data android:name="com.baidu.speech.APP_ID" android:value="${百度语音申请的appid}"/>
<meta-data android:name="com.baidu.speech.API_KEY" android:value="${百度语音申请的apikey}"/>
<meta-data android:name="com.baidu.speech.SECRET_KEY" android:value="${百度语音申请的secretkey}"/>
<service android:name="com.baidu.speech.VoiceRecognitionService" android:exported="false" />
```

#### dcloud_properties.xml配置

```xml
<feature name="Speech" value="io.dcloud.feature.speech.SpeechFeatureImpl">
    <module name="baidu" value="io.dcloud.feature.speech.BaiduSpeechEngine"/>
</feature>
```

---

### 讯飞语音

#### 需要添加的文件

| 路径 | 文件名 |
|---|---|
| SDK\libs | `speech-release.aar`, `speech_ifly-release.aar` |

#### AndroidManifest.xml配置

**权限同百度语音。**

**application节点下：**
```xml
<meta-data android:name="IFLY_APPKEY" android:value="${讯飞语音申请的appid}" />
```

#### dcloud_properties.xml配置

```xml
<feature name="Speech" value="io.dcloud.feature.speech.SpeechFeatureImpl">
    <module name="iFly" value="io.dcloud.feature.speech.IflySpeechEngine"/>
</feature>
```

---

### 相关模块

- [其他模块及国际化配置](other-modules.md) — audio-mp3aac 录制MP3格式音频
- [第三方 SDK 依赖说明](third-party-dependencies.md) — 百度语音版本信息

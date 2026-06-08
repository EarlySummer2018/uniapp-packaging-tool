# 其他模块及国际化配置（Android）

> **适用版本**：HBuilderX 5.0+
> **平台**：Android
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/

## 14. 其他模块及国际化配置

### VideoPlayer（视频播放）

| 路径 | 文件 |
|---|---|
| SDK/libs | `media-release.aar`, `weex_videoplayer-release.aar` |

**dcloud_properties.xml：**
```xml
<feature name="VideoPlayer" value="io.dcloud.media.MediaFeatureImpl"/>
```

---

### LivePusher（直播推流）

| 路径 | 文件 |
|---|---|
| SDK/libs | `weex_livepusher-release.aar` |

**AndroidManifest.xml权限：**
```xml
<uses-permission android:name="android.permission.INTERNET" />
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
<uses-permission android:name="android.permission.ACCESS_WIFI_STATE" />
<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" />
<uses-permission android:name="android.permission.READ_EXTERNAL_STORAGE" />
<uses-permission android:name="android.permission.RECORD_AUDIO" />
<uses-permission android:name="android.permission.MODIFY_AUDIO_SETTINGS" />
<uses-permission android:name="android.permission.BLUETOOTH" />
<uses-permission android:name="android.permission.CAMERA" />
<uses-permission android:name="android.permission.READ_PHONE_STATE" />
<uses-feature android:name="android.hardware.Camera"/>
<uses-feature android:name="android.hardware.camera.autofocus" />
```

**dcloud_properties.xml：**
```xml
<feature name="LivePusher" value="io.dcloud.media.live.LiveMediaFeatureImpl"/>
```

---

### Barcode（扫码）

**AndroidManifest.xml权限：**
```xml
<uses-permission android:name="android.permission.CAMERA" />
<uses-feature android:name="android.hardware.camera"/>
<uses-feature android:name="android.hardware.camera.autofocus"/>
<uses-permission android:name="android.permission.VIBRATE"/>
<uses-permission android:name="android.permission.FLASHLIGHT"/>
```

**dcloud_properties.xml：**
```xml
<feature name="Barcode" value="io.dcloud.feature.barcode2.BarcodeFeatureImpl"/>
```

---

### Bluetooth（低功耗蓝牙）

| 路径 | 文件 |
|---|---|
| SDK/libs | `Bluetooth-release.aar` |

**AndroidManifest.xml权限：**
```xml
<uses-permission android:name="android.permission.ACCESS_COARSE_LOCATION" />
<uses-permission android:name="android.permission.ACCESS_FINE_LOCATION" />
<uses-permission android:name="android.permission.BLUETOOTH_ADMIN" />
<uses-permission android:name="android.permission.BLUETOOTH" />
```

> targetSdkVersion 31及以上需追加：
> ```xml
> <uses-permission android:name="android.permission.BLUETOOTH_SCAN" />
> <uses-permission android:name="android.permission.BLUETOOTH_CONNECT" />
> ```

**dcloud_properties.xml：**
```xml
<feature name="Bluetooth" value="io.dcloud.feature.bluetooth.BluetoothFeature"/>
```

---

### Camera（相机/相册）

**权限：**
```xml
<uses-permission android:name="android.permission.CAMERA" />
```

**dcloud_properties.xml：**
```xml
<feature name="Camera" value="io.dcloud.js.camera.CameraFeatureImpl"/>
```

---

### iBeacon

| 路径 | 文件 |
|---|---|
| SDK/libs | `iBeacon-release.aar` |

**权限同Bluetooth模块（含targetSdkVersion 31+追加权限）。**

**dcloud_properties.xml：**
```xml
<feature name="iBeacon" value="io.dcloud.feature.iBeacon.WxBluetoothFeatureImpl"/>
```

---

### Contact（通讯录）

| 路径 | 文件 |
|---|---|
| SDK/libs | `contacts-release.aar` |

**权限：**
```xml
<uses-permission android:name="android.permission.GET_ACCOUNTS"/>
<uses-permission android:name="android.permission.WRITE_CONTACTS"/>
<uses-permission android:name="android.permission.READ_CONTACTS"/>
```

**dcloud_properties.xml：**
```xml
<feature name="Contacts" value="io.dcloud.feature.contacts.ContactsFeatureImpl"></feature>
```

---

### Fingerprint（指纹识别）

| 路径 | 文件 |
|---|---|
| SDK/libs | `fingerprint-release.aar` |

**权限：**
```xml
<uses-permission android:name="android.permission.USE_FINGERPRINT"/>
```

**dcloud_properties.xml：**
```xml
<feature name="Fingerprint" value="io.dcloud.feature.fingerprint.FingerPrintsImpl"/>
```

---

### Messaging（短彩邮件消息）

| 路径 | 文件 |
|---|---|
| SDK/libs | `messaging-release.aar` |

**权限：**
```xml
<uses-permission android:name="android.permission.RECEIVE_SMS"/>
<uses-permission android:name="android.permission.SEND_SMS"/>
<uses-permission android:name="android.permission.WRITE_SMS"/>
<uses-permission android:name="android.permission.READ_SMS"/>
```

**dcloud_properties.xml：**
```xml
<feature name="Messaging" value="io.dcloud.adapter.messaging.MessagingPluginImpl" />
```

---

### Record（录音）

**权限：**
```xml
<uses-permission android:name="android.permission.RECORD_AUDIO" />
<uses-permission android:name="android.permission.MODIFY_AUDIO_SETTINGS" />
```

---

### SQLite（数据库）

| 路径 | 文件 |
|---|---|
| SDK/libs | `sqlite-release.aar` |

**dcloud_properties.xml：**
```xml
<feature name="Sqlite" value="io.dcloud.feature.sqlite.DataBaseFeature"/>
```

---

### gcanvas

| 路径 | 文件 |
|---|---|
| SDK/libs | `weex_gcanvas-release.aar` |

---

### 拓展模块

- `audio-mp3aac-release.aar` — 录制音频时需要录制MP3格式时使用，拷贝到libs即可，不需额外配置。

---

### 隐私与政策提示框配置

#### 一级弹窗

| 字符串键名 | 字符串键值 |
|---|---|
| dcloud_privacy_prompt_title | 提示框标题，默认"服务协议和隐私政策" |
| dcloud_privacy_prompt_accept_button_text | 接受按钮文本，默认"同意" |
| dcloud_privacy_prompt_refuse_button_text | 拒绝按钮文本，默认不显示 |
| dcloud_privacy_prompt_message | 提示框内容，支持richtext |

#### 二级弹窗

| 字符串键名 | 字符串键值 |
|---|---|
| dcloud_second_privacy_prompt_title | 二级弹窗标题，默认不显示 |
| dcloud_second_privacy_prompt_accept_button_text | 确认按钮，默认"确定" |
| dcloud_second_privacy_prompt_refuse_button_text | 拒绝按钮，默认不显示 |
| dcloud_second_privacy_prompt_message | 内容，支持richtext |

> 默认不显示二级弹窗，配置后点击一级弹窗拒绝按钮时才会弹出。

---

### 国际化配置字符串

详见原文档，包括：
- html input(type=file) 选择页面国际化
- 图片选择器国际化字符串（多图）
- 应用启动时引导用户允许权限的提示语

---

### 相关模块

- [第三方 SDK 依赖说明](third-party-dependencies.md) — 默认集成依赖库信息
- [Speech 语音输入](speech.md) — audio-mp3aac拓展模块用于录制MP3
- [FAQ](../faq.md) — FAQ包含各类适配注意事项

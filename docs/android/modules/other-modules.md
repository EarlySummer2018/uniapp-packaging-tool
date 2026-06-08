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

### manifest.json 勾选说明

这些模块的勾选状态主要来自 `manifest.json` 的 `app-plus.modules`，不能只读取 `app-plus.distribute.sdkConfigs`。完整结构可参考 DCloud 文档：[App manifest.json 完整配置](https://uniapp.dcloud.net.cn/collocation/manifest-app.html#full-manifest)。

| manifest 模块名 | 打包处理 |
|---|---|
| `VideoPlayer` | 复制 `media-release.aar`、`weex_videoplayer-release.aar`，注册 `VideoPlayer` feature |
| `LivePusher` | 复制 `weex_livepusher-release.aar`，注册 `LivePusher` feature，追加相机/录音/网络等权限 |
| `Barcode` | 注册 `Barcode` feature，追加扫码所需相机/震动/闪光灯权限 |
| `Bluetooth` | 复制 `Bluetooth-release.aar`，注册 `Bluetooth` feature，追加蓝牙权限 |
| `iBeacon` | 复制 `iBeacon-release.aar`，注册 `iBeacon` feature，追加蓝牙权限 |
| `Contacts` / `Contact` | 复制 `contacts-release.aar`，注册 `Contacts` feature，追加通讯录权限 |
| `Fingerprint` | 复制 `fingerprint-release.aar`，注册 `Fingerprint` feature，追加指纹权限 |
| `Messaging` | 复制 `messaging-release.aar`，注册 `Messaging` feature，追加短信权限 |
| `Record` | 追加录音权限 |
| `SQLite` / `Sqlite` | 复制 `sqlite-release.aar`，注册 `Sqlite` feature |
| `gcanvas` / `GCanvas` | 复制 `weex_gcanvas-release.aar` |
| `Webview-x5` / `X5Webview` | 复制 X5 WebView AAR，注册 `X5Webview` feature |

# Geolocation 定位（Android）

> **适用版本**：HBuilderX 5.0+
> **平台**：Android
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/

## 1. Geolocation（定位）

**离线打包地图模块与定位模块可以分别配置，不需要单独依赖地图模块。**

### 百度定位

#### 需要拷贝的文件

| 路径 | 文件 |
|---|---|
| SDK\libs | `baidu-libs-release.aar`, `geolocation-baidu-release.aar` |

#### application节点下配置

```xml
<meta-data android:name="com.baidu.lbsapi.API_KEY" android:value="%appkey_android%"></meta-data>
<service android:name="com.baidu.location.f" android:enabled="true" android:process=":remote"></service>
```

---

### 高德定位

#### 需要拷贝的文件

| 路径 | 文件 |
|---|---|
| SDK\libs | `geolocation-amap-release.aar` |

高德 SDK 通过 gradle 集成。

#### 通过gradle集成高德定位SDK

```groovy
android {
    xxxxxxxx
    defaultConfig {
        xxxxxxxx
    }
}
dependencies {
    xxxxxxxx
    implementation('com.amap.api:location:6.4.5')
}
```

**注意事项：**
- 版本号通过离线SDK中的demo获取相对应版本
- 本地集成的高德定位SDK需要删除相关aar文件，否则会导致sdk冲突
- 高德定位与高德地图SDK集成冲突，需要注意如果集成地图无须再配置定位

#### AndroidManifest.xml文件需要修改的项

**需要在application节点前添加权限**

```xml
<uses-permission android:name="android.permission.ACCESS_COARSE_LOCATION"/>
<uses-permission android:name="android.permission.ACCESS_FINE_LOCATION"/>
<uses-permission android:name="android.permission.ACCESS_WIFI_STATE"/>
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE"/>
<uses-permission android:name="android.permission.CHANGE_WIFI_STATE"/>
<uses-permission android:name="android.permission.READ_PHONE_STATE"/>
<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE"/>
<uses-permission android:name="android.permission.INTERNET"/>
<uses-permission android:name="android.permission.MOUNT_UNMOUNT_FILESYSTEMS"/>
<uses-permission android:name="android.permission.READ_LOGS"/>
<uses-permission android:name="android.permission.WRITE_SETTINGS"/>
<uses-permission android:name="android.permission.ACCESS_BACKGROUND_LOCATION"/>
<uses-permission android:name="android.permission.FOREGROUND_SERVICE"/>
```

**application节点下配置**

```xml
<meta-data android:name="com.amap.api.v2.apikey" android:value="%用户申请的APPkey%"></meta-data>
<service android:name="com.amap.api.location.APSService"></service>
```

---

### 系统定位

#### 需要拷贝的文件

**最新SDK使用系统定位已不需要引入任何文件**

#### AndroidManifest.xml文件需要修改的项

**需要在application节点前添加权限**

```xml
<uses-permission android:name="android.permission.ACCESS_COARSE_LOCATION"/>
<uses-permission android:name="android.permission.ACCESS_FINE_LOCATION"/>
<uses-permission android:name="android.permission.ACCESS_WIFI_STATE"/>
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE"/>
<uses-permission android:name="android.permission.CHANGE_WIFI_STATE"/>
<uses-permission android:name="android.permission.READ_PHONE_STATE"/>
<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE"/>
<uses-permission android:name="android.permission.INTERNET"/>
<uses-permission android:name="android.permission.MOUNT_UNMOUNT_FILESYSTEMS"/>
<uses-permission android:name="android.permission.READ_LOGS"/>
<uses-permission android:name="android.permission.WRITE_SETTINGS"/>
```

---

### 腾讯定位

腾讯定位依赖于UTS基础模块，请先集成[UTS基础模块](uts-base-module.md)。

#### 需要拷贝的文件

| 路径 | 文件 |
|---|---|
| SDK\libs | `uni-getLocation-tencent-uni1-release.aar` |

#### 通过gradle集成腾讯定位SDK

```groovy
android {
    xxxxxxxx
    defaultConfig {
        xxxxxxxx
    }
}
dependencies {
    xxxxxxxx
    implementation('com.tencent.map.geolocation:TencentLocationSdk-openplatform:xxx')
}
```

> **注意**：xxx是腾讯定位版本号

**application节点下配置**

```xml
<meta-data android:name="TencentMapSDK" android:value="您申请的腾讯定位App Key"/>
```

---

### 相关模块

- [Map 地图](map.md) — 地图模块配置（百度/高德/谷歌）
- [UTS 基础模块](uts-base-module.md) — 腾讯定位依赖此模块

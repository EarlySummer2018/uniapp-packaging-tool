# Map 地图（Android）

> **适用版本**：HBuilderX 5.0+
> **平台**：Android
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/

## 5. Map（地图）

> **开发者需要修改使用的地图插件时，需要修改dcloud_properties.xml文件的features节点下Maps节点value属性的配置，高德地图和百度地图的配置只能保留一个。**

### 百度地图

#### 需要拷贝的文件

| 路径 | 文件 |
|---|---|
| SDK\libs | `baidu-libs-release.aar`, `map-baidu-release.aar` |

> 百度地图暂时不支持 nvue map 标签

#### AndroidManifest.xml配置

**权限：**
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

**application节点下：**
```xml
<meta-data android:name="com.baidu.lbsapi.API_KEY" android:value="%appkey_android%"></meta-data>
<service android:name="com.baidu.location.f" android:enabled="true" android:process=":remote"></service>
```

#### dcloud_properties.xml配置

**features节点：**
```xml
<feature name="Maps" value="io.dcloud.js.map.JsMapPluginImpl"></feature>
```
**services节点：**
```xml
<service name="Maps" value="io.dcloud.js.map.MapInitImpl" />
```

---

### 高德地图

#### 需要拷贝的文件

| 页面类型 | 路径 | 文件 |
|---|---|---|
| nvue页面 | SDK\libs | `weex_amap-release.aar` |
| vue页面 | SDK\libs | `map-amap-release.aar` |

高德 SDK 通过 gradle 集成。

#### 通过gradle集成高德地图SDK

```groovy
dependencies {
    implementation('com.amap.api:3dmap:xxx')
    implementation('com.amap.api:search:xxx')
}
```

> xxx是版本号，通过离线SDK中的demo获取。本地集成的需删除相关aar文件否则冲突。高德定位与高德地图SDK集成冲突，集成地图无须再配定位。

#### AndroidManifest.xml配置

**权限同百度地图（不含最后两项额外权限）。**

**application节点下：**
```xml
<meta-data android:name="com.amap.api.v2.apikey" android:value="%appkey_android%"/>
<service android:name="com.amap.api.location.APSService"></service>
```

> 高德地图使用的appkey与包名及签名文件存在对应关系，填写错误会导致地图无法正常使用。

#### dcloud_properties.xml配置

```xml
<feature name="Maps" value="io.dcloud.js.map.amap.JsMapPluginImpl"></feature>
```

> 高德地图不需要添加services节点。

---

### 谷歌地图

#### 需要拷贝的文件

| 路径 | 文件 |
|---|---|
| SDK\libs | `weex_google-map-release.aar` |

> 谷歌地图仅支持 nvue map 标签

#### app目录build.gradle添加依赖

```groovy
implementation 'com.google.android.gms:play-services-maps:18.0.1'
```

#### AndroidManifest.xml配置

**权限：**
```xml
<uses-permission android:name="android.permission.ACCESS_COARSE_LOCATION" />
<uses-permission android:name="android.permission.ACCESS_FINE_LOCATION" />
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
<uses-permission android:name="android.permission.INTERNET" />
<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" />
<uses-permission android:name="android.permission.ACCESS_LOCATION_EXTRA_COMMANDS" />
```

**application节点下：**
```xml
<meta-data android:name="com.google.android.geo.API_KEY" android:value="%api_key%" />
```

> api_key在[谷歌开发者](https://mapsplatform.google.com/)开通。谷歌地图不需要修改dcloud_properties.xml文件。

---

### 相关模块

- [Geolocation 定位](geolocation.md) — 地图与定位可分别配置，高德地图含定位功能
- [第三方 SDK 依赖说明](third-party-dependencies.md) — 百度/高德地图版本信息

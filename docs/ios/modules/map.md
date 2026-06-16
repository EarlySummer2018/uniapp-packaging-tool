# Map（地图）（iOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：iOS (iPhone/iPad)
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/map.html

---

iOS 地图模块支持**百度地图**（vue 页面）、**高德地图**（nvue 页面）和**谷歌地图**。

> **HBuilderX 5.13+ 推荐使用本地 Pod 集成地图模块。**
> - 百度地图使用 `Map-Baidu`
> - 高德地图使用 `Map-Gaode`
> - Google 地图使用 `Map-Google`
>
> 如只使用定位能力，可选择 `Geolocation`、`Geolocation-Baidu` 或 `Geolocation-Gaode`；手动集成时再参考下方依赖表。

> **注意**：工程里只能有一个地图，其他地图功能需要删除 Info.plist 里的对应 key 和库文件，请根据 [功能模块与依赖关系对照表](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/common?id=%e5%a6%82%e4%bd%95%e9%85%8d%e7%bd%ae%e6%a8%a1%e5%9d%97%e4%b8%89%e6%96%b9sdk) 配置

## 4.1 百度地图（仅 vue 页面支持）

### 添加依赖资源及文件

| 依赖库 | 系统库 | 依赖资源 |
|--------|--------|----------|
| `BaiduMapAPI_Utils.framework`、`BaiduMapAPI_Base.framework`、`BaiduMapAPI_Search.framework`、`BaiduMapAPI_Map.framework`、`BMKLocationKit.framework`、`liblibMap.a`、`libbmapimp.a`、`libBaiduKeyVerify.a`、`libssl.a`、`libcrypto.a` | `libc++.tbd`、`libsqlite3.0.tbd`、`libz.tbd`、`QuartzCore.framework`、`CoreGraphics.framework`、`CoreTelephony.framework`、`Accelerate.framework`、`SystemConfiguration.framework`、`Security.framework`、`MapKit.framework`、`OpenGLES.framework`、`CoreLocation.framework` | `mapapi.bundle` |

### 账号配置

1. 申请 AppKey，如果没有 AppKey 将会导致地图显示不出，参考 [百度地图 AppKey 申请章节](http://ask.dcloud.net.cn/article/29)

2. 打开 Info.plist 文件找到 `baidu` 项，如果没有则添加该项，在下图中红色区域输入申请的 AppKey。注意 Info.plist 中 Bundle identifier 要和输入的安全码一致

```xml
<key>baidu</key>
<dict>
    <key>appkey</key>
    <string>%在此处输入申请的AppKey%</string>
</dict>
```

3. 在工程的 Info.plist 添加 `NSLocationAlwaysAndWhenInUseUsageDescription` 和 `NSLocationWhenInUseUsageDescription` key，并填写获取权限描述信息：

```xml
<key>NSLocationWhenInUseUsageDescription</key>
<string>我们需要使用您的位置信息来显示当前位置</string>
<key>NSLocationAlwaysAndWhenInUseUsageDescription</key>
<string>我们需要使用您的位置信息来提供持续导航服务</string>
```

### 常见问题解决

1. 如下图只能看见栅格图，可能的原因：AppKey 配置不对、Bundle identifier 和安全码不一致、百度地图缓存导致（可删除 App 重新安装）

2. 提示 AppKey 校验错误时，在 Xcode 控制台搜索 `baidu maponGetPermissionState` 查看错误码，对比百度开发平台错误信息

## 4.2 高德地图（仅 nvue 页面支持）

### 添加依赖资源及文件

| 依赖库 | 系统库 | 依赖资源 |
|--------|--------|----------|
| `liblibMap.a`、`libAMapImp.a` | `MapKit.framework`、`AMapSearchKit.framework`、`MAMapKit.framework`、`CoreLocation.framework`、`AMapFoundationKit.framework`、`libc++.tbd`、`GLKit.framework` | `AMap.bundle`、`userPosition@2x.png` |

> 注：`userPosition@2x.png` 为显示带方向的用户位置的图标，可替换为自己的设计

### 账号配置

1. 在 [高德地图官网](http://lbs.amap.com/api/ios-sdk/guide/create-project/get-key)申请 AppKey

2. 在工程的 Info.plist 添加 `amap` 节点，添加 AppKey 信息：

```xml
<key>amap</key>
<dict>
    <key>appkey</key>
    <string>%在此处输入申请的AppKey%</string>
</dict>
```

3. 在工程的 Info.plist 添加 `NSLocationAlwaysAndWhenInUseUsageDescription` 和 `NSLocationWhenInUseUsageDescription` key，并填写获取权限描述信息：

```xml
<key>NSLocationWhenInUseUsageDescription</key>
<string>我们需要使用您的位置信息来显示当前位置</string>
<key>NSLocationAlwaysAndWhenInUseUsageDescription</key>
<string>我们需要使用您的位置信息来提供持续导航服务</string>
```

## 4.3 uni 项目的 nvue 页面中使用地图组件（目前只支持高德地图）

### 添加依赖资源及文件

| 依赖库 | 系统库 | 依赖资源 |
|--------|--------|----------|
| `libDCUniMap.a`、`libDCUniAmap.a`、`Masonry.framework`、`AMapSearchKit.framework`、`MAMapKit.framework`、`AMapFoundationKit.framework` | `MapKit.framework`、`CoreLocation.framework`、`libc++.tbd`、`GLKit.framework` | `AMap.bundle`、`userPosition@2x.png` |

> 注：`userPosition@2x.png` 为显示带方向的用户位置的图标，可替换为自己的设计

### 账号配置

1. 在 [高德地图官网](http://lbs.amap.com/api/ios-sdk/guide/create-project/get-key)申请 AppKey

2. 在工程的 Info.plist 添加 `amap` 节点，添加 AppKey 信息：

```xml
<key>amap</key>
<dict>
    <key>appkey</key>
    <string>%在此处输入申请的AppKey%</string>
</dict>
```

3. 在工程的 Info.plist 添加 `NSLocationAlwaysAndWhenInUseUsageDescription` 和 `NSLocationWhenInUseUsageDescription` key，并填写获取权限描述信息：

```xml
<key>NSLocationWhenInUseUsageDescription</key>
<string>我们需要使用您的位置信息来显示当前位置</string>
<key>NSLocationAlwaysAndWhenInUseUsageDescription</key>
<string>我们需要使用您的位置信息来提供持续导航服务</string>
```

## 4.4 谷歌地图

### 添加依赖资源及文件

| 依赖库 | 系统库 | 依赖资源 |
|--------|--------|----------|
| `libDCUniMap.a`、`libDCUniGoogleMap.a`、`GoogleMapsBase.framework`、`GoogleMaps.framework`、`GoogleMapsCore.framework`、`liblibMap.a` | `Accelerate.framework`、`CoreData.framework`、`CoreGraphics.framework`、`CoreImage.framework`、`CoreLocation.framework`、`CoreTelephony.framework`、`CoreText.framework`、`GLKit.framework`、`ImageIO.framework`、`libc++.tbd`、`libz.tbd`、`Metal.framework`、`OpenGLES.framework`、`QuartzCore.framework`、`SystemConfiguration.framework` | `GoogleMaps.bundle` |

### 账号配置

1. 在 [谷歌地图官网](https://developers.google.com/maps)申请 APIKey

2. 在工程的 Info.plist 添加 `googleMap` 节点，添加 APIKey 信息：

```xml
<key>googleMap</key>
<dict>
    <key>apikey</key>
    <string>%在此处输入申请的APIKey%</string>
</dict>
```

3. 在工程的 Info.plist 添加 `NSLocationAlwaysAndWhenInUseUsageDescription` 和 `NSLocationWhenInUseUsageDescription` key，并填写获取权限描述信息：

```xml
<key>NSLocationWhenInUseUsageDescription</key>
<string>我们需要使用您的位置信息来显示当前位置</string>
<key>NSLocationAlwaysAndWhenInUseUsageDescription</key>
<string>我们需要使用您的位置信息来提供持续导航服务</string>
```

---

## 交叉引用

- 上一篇：[Oauth（登录鉴权）](oauth.md)
- 下一篇：[Speech（语音输入）](speech.md)
- 相关模块：[Geolocation（定位）](geolocation.md)

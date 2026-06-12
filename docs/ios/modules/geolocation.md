# Geolocation（定位）（iOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：iOS (iPhone/iPad)
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/
>
> **最后更新**：2024年7月

---

iOS 平台支持**三种定位方案**：百度定位、高德定位、系统定位。根据项目需求选择合适的方案。

## 一、百度定位配置

### 需要引入的依赖库

| 类别 | 内容 |
|------|------|
| **第三方库** | `libBaiduLocationPlugin.a`<br>`libBaiduKeyVerify.a`<br>`liblibGeolocation.a`<br>`libssl.a`<br>`libcrypto.a`<br>`BaiduMapAPI_Utils.framework`<br>`BaiduMapAPI_Base.framework`<br>`BaiduMapAPI_Search.framework`<br>`BMKLocationKit.framework` |
| **系统库** | `libc++.tbd`<br>`libsqlite3.0.tbd`<br>`SystemConfiguration.framework`<br>`Security.framework`<br>`CoreLocation.framework`<br>`CoreTelephony.framework` |

### Info.plist 配置

**步骤1：申请 AppKey**

读取用户在 manifest.json 中配置的 AppKey。

**步骤2：在 Info.plist 文件中找到 `baidu` 项，添加 Dictionary 类型的配置：**

```xml
<key>baidu</key>
<dict>
    <key>appkey</key>
    <string>%在此处输入申请的AppKey%</string>
</dict>
```

> **重要提示**：Info.plist 中的 Bundle identifier 必须与申请安全码时填写的一致

### 隐私权限配置（Info.plist）

需要在 Info.plist 中添加以下隐私权限声明：

| 权限 Key | 类型 | 说明 |
|----------|------|------|
| `Privacy - Location Usage Description` | String | 使用定位说明 |
| `Privacy - Location Always and When In Use Usage Description` | String | 始终及使用时定位说明 |
| `Privacy - Location Always Usage Description` | String | 始终定位说明 |
| `Privacy - Location When In Use Usage Description` | String | 使用时定位说明 |

---

## 二、高德定位配置

### 需要引入的依赖库

| 类别 | 内容 |
|------|------|
| **第三方库** | `libAMapLocationPlugin.a`<br>`lilibGeolocation.a`<br>`AMapFoundationKit.framework`<br>`AMapLocationKit.framework` |
| **系统库** | `libc++.tbd`<br>`libz.tbd`<br>`ExternalAccessory.framework`<br>`GLKit.framework`<br>`Security.framework`<br>`CoreTelephony.framework`<br>`SystemConfiguration.framework` |

### Info.plist 配置

**步骤1：申请 AppKey**

读取用户在 manifest.json 中配置的 AppKey。

**步骤2：在 Info.plist 文件中找到 `amap` 项，添加 Dictionary 类型的配置：**

```xml
<key>amap</key>
<dict>
    <key>appkey</key>
    <string>%在此处输入申请的AppKey%</string>
</dict>
```

### 隐私权限配置（Info.plist）

与百度定位相同的四项隐私权限声明（见上表）

---

## 三、系统定位配置（最轻量）

### 需要引入的依赖库

| 类别 | 内容 |
|------|------|
| **第三方库** | `liblibGeolocation.a` |
| **系统库** | `Foundation.framework`<br>`CoreLocation.framework` |

### 隐私权限配置（Info.plist）

与上述相同的四项隐私权限声明

---

## 交叉引用

- 上一篇：[UTS 内置模块](uts-builtin-modules.md)
- 下一篇：[Payment（支付）](payment.md)
- 相关模块：[Map（地图）](map.md)（百度地图/高德地图可复用 AppKey）

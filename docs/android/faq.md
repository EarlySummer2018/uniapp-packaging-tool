# Android 注意事项（FAQ）

> **适用版本**：HBuilderX 5.0+
> **平台**：Android
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/

## Android 注意事项

### 1. minSdkVersion设置的大于21时，应用启动白屏

需要在AndroidManifest.xml的application节点添加 `android:extractNativeLibs="true"`：

```xml
<application
    android:extractNativeLibs="true">
</application>
```

### 2. HBuilderX 4.41版，离线打包报 error: style attribute 'android:attr/windowOptOutEdgeToEdgeEnforcement' not found

从4.41版起，为了适配Android 15，需要配置：
- **compileSdk**: 35
- **buildToolsVersion**: 35.0.0
- **Gradle**: 8.11.1 版
- **Android Gradle 插件**: 8.7.3 版

### 3. 当设置targetSdkVersion为34时，上架 Google Play 应用白屏

如果targetSdkVersion设置为34时，需要在build.gradle的android节点下新增以下内容：

```gradle
packagingOptions {
    jniLibs {
        useLegacyPackaging true
    }
}
```

### 4. 上架 Google Play 提示：SoLoader SDK 版本有问题

需要在主模块的build.gradle的dependencies节点中添加SoLoader的依赖：

```gradle
implementation "com.facebook.soloader:soloader:0.10.5"
```

### 5. 离线SDK集成实人认证时报错：lib/*/libc++_shared.so

离线SDK集成实人认证如果出现该报错时，需要在module的build.gradle的android节点下添加如下内容：

```gradle
packagingOptions {
    pickFirst 'lib/*/libc++_shared.so'
}
```

### 6. 解决上架 Google Play 审核报 DCloud SDK 包含从未知来源下载或安装应用的问题

HBuilder X 3.8.7-alpha开始，离线打包将安装功能独立成单独的aar —— **install-apk-release.aar**，上架谷歌市场不能包含此库，非谷歌市场可酌情考量。

不包含此库，调用plus.runtime.install将无法安装apk文件。

其余上架谷歌市场注意事项可参考[文档](https://uniapp.dcloud.net.cn/tutorial/android-gp.html)。

### 7. 离线打包存在多个以uni-jsframework开头的文件

离线打包时为减少集成难度，默认会将所有框架都包含在内，如果需要去除其余框架，可参考[文档](https://nativesupport.dcloud.net.cn/AppDocs/FAQ/jsframeworkdeclare.html)配置。

### 8. 适配暗黑模式

适配暗黑模式新增了webkit依赖库，需要将如下配置添加到build.gradle中：

```gradle
dependencies {
    implementation 'androidx.webkit:webkit:1.3.0'
}
```

为适配暗黑模式，需要在AndroidManifest.xml中PandoraEntryActivity对应的android:configChanges中添加uiMode：

```xml
<activity
    android:name="io.dcloud.PandoraEntryActivity"
    android:launchMode="singleTask"
    android:configChanges="orientation|keyboardHidden|screenSize|mcc|mnc|fontScale|keyboard|smallestScreenSize|screenLayout|screenSize|uiMode"
    android:hardwareAccelerated="true"
    android:permission="com.miui.securitycenter.permission.AppPermissionsEditor"
    android:screenOrientation="user"
    android:theme="@style/DCloudTheme"
    android:windowSoftInputMode="adjustResize">
    <intent-filter>
        <category android:name="android.intent.category.DEFAULT" />
        <category android:name="android.intent.category.BROWSABLE" />
        <action android:name="android.intent.action.VIEW" />
        <data android:scheme=" " />
    </intent-filter>
</activity>
```

### 9. 离线打包设置隐私协议状态

如果离线打包需要自定义隐私协议，为了不影响SDK正常使用，需要用户在同意或拒绝隐私状态时同步到SDK：

- **SDK.setAgreePrivacy(Context context, boolean isAllow)** - 设置SDK隐私协议的状态（支持版本：3.3.1+）
- **SDK.isAgreePrivacy(Context context)** - 获取SDK隐私协议的状态（支持版本：3.3.1+）

### 10. Android 12 适配

离线打包如果将targetSdkVersion设置为31时，在Android 12设备上安装可能会报如下错误：

```
adb: failed to install XXX.apk: Failure [INSTALL_PARSE_FAILED_MANIFEST_MALFORMED: Failed parse during installPackageLI: /data/base.apk (at Binary XML file line #173): 
XXX.XXX.XXX.TestActivity: Targeting S+ (version 31 and above) requires that an explicit value for android:exported be defined when intent filters are present]
```

**解决方案**：Android 12 中要求包含 intent-filter 的 activity、service 或 receiver 必须显示声明 android:exported 属性：

```xml
<activity
    android:name="XXX.XXX.XXX.TestActivity"
    android:exported="true">
    <intent-filter>
        ......
    </intent-filter>
</activity>
```

> **注意**：Android系统默认包含 intent-filter 的组件android:exported默认值为true，所以建议将android:exported设置为true

### 11. 应用启动白屏或者提示打包时未添加ui模块

大多数是因为build.gradle中配置了混淆。如果需要使用proguard混淆代码，需确保不要混淆SDK的代码。混淆配置和混淆文件可以参考SDK中的UniPlugin-Hello-AS项目。

### 12. 编译报错 style attribute 'android:attr/forceDarkAllowed' not found.

需要将 compileSdkVersion 设置为 **29 或以上**。

### 13. breakpad 配置

离线SDK新增加了breakpad-build-release.aar，直接将这个库拷贝到libs目录下即可。详情可参考[文档](https://nativesupport.dcloud.net.cn/AppDocs/usesdk/android.html)。

### 14. gallery 冲突问题

gallery-dmcBig-release.aar相应代码被加入到lib.5plus.base-release.aar，使用时请删除gallery-dmcBig-release.aar库。

### 15. 离线打包编译报错

如果离线打包编译时提示以下错误：

```
Execution failed for task ':hbuilder:checkDebugDuplicateClasses'.
> 1 exception was raised by workers:
java.lang.IllegalStateException: java.lang.IllegalStateException: Worker finished without being first started
```

**解决方案**：

1. 将项目根目录下的build.gradle中的gradle插件版本升级：

```gradle
buildscript {
    repositories {
        jcenter()
        google()
    }
    dependencies {
        classpath 'com.android.tools.build:gradle:4.1.1'
    }
}
```

2. 修改项目根目录 gradle/gradle-wrapper.properties 下的gradle的版本到6.5：

```properties
distributionBase=GRADLE_USER_HOME
distributionPath=wrapper/dists
zipStoreBase=GRADLE_USER_HOME
zipStorePath=wrapper/dists
distributionUrl=https\://services.gradle.org/distributions/gradle-6.5-bin.zip
```

### 16. 离线打包无法调起应用安装界面

如果离线打包调用plus.runtime.install无法调起安装界面，需要在Androidmanifest.xml中添加：

**添加权限**（manifest节点下）：

```xml
<uses-permission android:name="android.permission.REQUEST_INSTALL_PACKAGES"/>
```

**添加provider节点**（application节点下）：

```xml
<provider
    android:name="io.dcloud.common.util.DCloud_FileProvider"
    android:authorities="${apk.applicationId}.dc.fileprovider"
    android:exported="false"
    android:grantUriPermissions="true">
    <meta-data
        android:name="android.support.FILE_PROVIDER_PATHS"
        android:resource="@xml/dcloud_file_provider" />
</provider>
```

> ${apk.applicationId}须替换成应用的包名。

### 17. 离线打包Android 10上无法启动相机

在application节点下添加provider节点（同第16条）：

```xml
<provider
    android:name="io.dcloud.common.util.DCloud_FileProvider"
    android:authorities="${apk.applicationId}.dc.fileprovider"
    android:exported="false"
    android:grantUriPermissions="true">
    <meta-data
        android:name="android.support.FILE_PROVIDER_PATHS"
        android:resource="@xml/dcloud_file_provider" />
</provider>
```

### 18. uni-AD业务状态异常

如果出现uni-AD业务状态异常提醒，请删除掉未申请的平台的相关配置和aar。例如广告后台添加了穿山甲广告，但没有添加360和广点通的广告，请删除掉广点通和360的相关配置和aar。

### 19. x5配置

如果需要使用x5内核，将webview-x5-release.aar拷贝到libs目录下，直接运行即可。

uni-app将webview-x5-release.aar和weex_webview-x5-release.aar拷贝到libs目录下即可。

### 20. 推送上传谷歌市场注意事项

参考文档：[Android离线SDK解决使用UniPush和个推推送违反谷歌应用商店（GooglePlay）个人和敏感信息政策的问题](https://ask.dcloud.net.cn/article/36495)

### 21. 高德地图上传谷歌市场注意事项

如需上传谷歌市场，将原来的amap-libs-release.aar替换成amap-gp-libs-release.aar即可。

### 22. uni-app离线打包注意事项

参考文档：[uni-app离线打包Android平台注意事项](https://ask.dcloud.net.cn/article/35139)

### 23. 重写Application

如果集成离线SDK时需要重写application，**必须继承自DCloudApplication**，否则会导致SDK中业务逻辑无法正常运行：

```xml
<application 
    android:name="io.dcloud.test.TestApplication" 
    android:icon="@drawable/icon" 
    android:label="@string/app_name" 
    tools:replace="android:name">
</application>
```

### 24. 添加so库

如果需要集成的第三方sdk存在so库文件，只需添加 **armeabi-v7a、arm64-v8a、x86** 三个文件夹即可，如果添加其他文件夹会导致在部分手机上无法运行。

### 25. 打包aab运行白屏

按以下配置修改：

1. 原生项目主app的AndroidManifest.xml中，application节点配置 `android:extractNativeLibs="true"`
2. 原生项目根目录 gradle.properties 配置 `android.bundle.enableUncompressedNativeLibs=false`
3. 重新编译打包

### 26. 适配Android 13 文件权限

为了兼容Android 13 新的权限要求，需要在AndroidManifest.xml 中新增下面的权限声明：

```xml
<uses-permission android:name="android.permission.READ_MEDIA_IMAGES" />
<uses-permission android:name="android.permission.READ_MEDIA_VIDEO" />
```

---

### 相关模块

- [Geolocation 定位](modules/geolocation.md) — 定位模块配置
- [Map 地图](modules/map.md) — 地图模块配置（含高德谷歌市场适配）
- [Push 消息推送](modules/push.md) — 推送模块配置（含谷歌市场适配）
- [Payment 支付](modules/payment.md) — 支付模块配置
- [Share 分享](modules/share.md) — 分享模块配置
- [Oauth 登录鉴权](modules/oauth.md) — 登录鉴权配置
- [Speech 语音输入](modules/speech.md) — 语音输入配置
- [Statistic 统计](modules/statistic.md) — 统计模块配置
- [FacialRecognitionVerify 实人认证](modules/facial-recognition-verify.md) — 实人认证配置
- [uni-AD 广告](modules/uni-ad.md) — 广告模块配置
- [其他模块及国际化配置](modules/other-modules.md) — 其他功能模块及国际化
- [第三方 SDK 依赖说明](modules/third-party-dependencies.md) — SDK依赖版本信息

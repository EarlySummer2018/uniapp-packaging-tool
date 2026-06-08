# 启动图配置

## Android 启动图配置

### 用户在 manifest.json 中选择了通用启动图

对于通用启动图，Android 端主要依赖原生工程中的应用图标和资源名称，无需修改 `styles.xml` 中的 `windowBackground` 为图片资源。

**步骤：**

1. **配置应用图标和名称**  
   - 在 `manifest.json` 中正确填写 `app-plus -> distribute -> android -> icon` 和 `name`。  
   - 在离线工程的 `AndroidManifest.xml` 中，确保 `application` 节点的 `android:icon` 指向你的应用图标资源（通常是 `@drawable/icon` 或 `@mipmap/ic_launcher`）。  
   - 确保 `application` 节点的 `android:label` 指向你的应用名称资源（通常是 `@string/app_name`）。

2. **无需放置 Splash 图片**  
   - 通用启动图模式下，不需要在 `res/drawable` 目录下放置 `splash.png` 或 `splash.9.png`。  
   - 如果之前为了自定义启动图放置了这些文件，且希望切换回通用模式，建议移除或重命名这些文件，避免冲突（尽管通常 SDK 会优先读取通用逻辑，但清理冗余文件是好习惯）。

3. **确认 Activity 主题**  
   - 检查 `AndroidManifest.xml` 中启动 Activity（如 `MainActivity`）的主题。  
   - 如果使用默认主题（如 `Theme.AppCompat.Light.NoActionBar`），通常即可正常显示通用启动图。  
   - **注意**：不要像配置自定义全屏启动图那样将 `windowBackground` 设置为某个图片资源，否则可能会覆盖掉 SDK 默认的通用启动图渲染逻辑，导致显示异常或黑屏。保持默认或仅设置无 ActionBar 即可。

4. **关闭启动页控制（可选）**  
   - 在 `manifest.json` 的 `app-plus -> splashscreen` 中配置关闭策略：
     ```json
     "splashscreen": {
         "alwaysShowBeforeRender": false,
         "autoclose": false
     }
     ```
   - 然后在首页 JS 中调用 `plus.navigator.closeSplashscreen()` 关闭启动图。

---

### 用户在 manifest.json 中选择了自定义启动图

> 用户如果在 `manifest.json` 配置的是 `.png`，则需要将图片重命名为 `splash.png`。  
> 用户如果在 `manifest.json` 配置的是 `.9.png`，则需要将图片重命名为 `splash.9.png`。  
> 下面以 `splash.9.png` 为例，介绍如何配置自定义全屏启动图。

#### 1. Android 平台配置（核心）

在离线工程中，`.9.png` 需要被系统识别为可拉伸背景，并应用于启动 Activity 的主题中。

**第一步：准备图片（读取用户配置本地项目路径项目中的 manifest.json 中配置的启动图）**  
- 将你的 `.9.png` 图片命名为 `splash.9.png`（或者自定义名称如 `unipack_splash.9.png`）。  
- **重要**：确保图片四边有正确的黑线标记（左上角定义拉伸区域，右下角定义内容显示区域）。

- **推荐做法：多分辨率适配**  
  为了在不同屏幕密度的设备上获得最佳显示效果，建议将适配好的 `.9.png` 文件放入对应的 `drawable-dpi` 文件夹中。典型的目录结构如下：

  ```
  android/
  └── res/
      ├── drawable-hdpi/       // 对应 hdpi (480x762) - 可选，若未提供可省略
      │   └── splash.9.png
      ├── drawable-xhdpi/      // 对应 xhdpi (720x1242)
      │   └── splash.9.png     // 放入 720P 的 .9.png
      ├── drawable-xxhdpi/     // 对应 xxhdpi (1080x1882)
      │   └── splash.9.png     // 放入 1080P 的 .9.png
      └── ...
  ```

  > **提示**：uni-app SDK 默认会查找 `res/drawable` 下的 `splash.9.png` 或根据密度自动匹配。为了保险起见，建议在 `drawable-xhdpi` 和 `drawable-xxhdpi` 中都放入对应分辨率的 `.9.png` 文件。

- 如果项目中没有对应密度的文件夹，可以只将图片放入 `res/drawable` 目录，系统会自动缩放，但为了最佳效果，强烈建议按上述结构放置。

**第二步：修改 styles.xml**  
打开 `android/res/values/styles.xml`，创建一个继承自无 ActionBar 主题的新样式，并将 `windowBackground` 设置为你的 `.9.png` 资源。

```xml
<style name="AppTheme.Splash" parent="Theme.AppCompat.Light.NoActionBar">
    <!-- 隐藏标题栏 -->
    <item name="windowNoTitle">true</item>
    <!-- 隐藏 Action Bar -->
    <item name="windowActionBar">false</item>
    <!-- 去除内容覆盖层 -->
    <item name="android:windowContentOverlay">@null</item>
    <!-- 设置为全屏模式 -->
    <item name="android:windowFullscreen">true</item>
    <!-- 设置启动背景为 .9.png 图片 -->
    <!-- 假设图片名为 splash.9.png，资源名即为 splash -->
    <item name="android:windowBackground">@drawable/splash</item>
</style>
```

**第三步：修改 AndroidManifest.xml**  
找到应用的启动 Activity（通常是 `MainActivity` 或 `LaunchActivity`），将其 `theme` 属性指向刚才创建的 `AppTheme.Splash`。

```xml
<activity
    android:name=".MainActivity"
    android:screenOrientation="portrait"
    android:configChanges="orientation|keyboardHidden|navigation"
    android:exported="true"
    android:theme="@style/TranslucentTheme"> <!-- 应用自定义全屏主题 -->

    <intent-filter>
        <action android:name="android.intent.action.MAIN" />
        <category android:name="android.intent.category.LAUNCHER" />
    </intent-filter>
</activity>
```

**第四步：关闭启动页（可选，推荐）**  
为了实现“和云打包一样”的体验，通常希望等待首页渲染完成后再关闭启动图，避免白屏闪烁。

- 在 `manifest.json` 的 `app-plus -> splashscreen` 中配置：
  ```json
  "splashscreen": {
      "alwaysShowBeforeRender": false,
      "autoclose": false
  }
  ```
- 在首页 Vue 文件的 `onLoad` 或 `mounted` 生命周期中调用关闭接口：
  ```js
  // #ifdef APP-PLUS
  plus.navigator.closeSplashscreen();
  // #endif
  ```

---
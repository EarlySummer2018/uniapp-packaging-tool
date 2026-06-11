### 配置Appkey
打开info.plist，创建key为dcloud_appkey，Value选择String类型，值读取用户配置的：dcloud appkey

**注意：**需要将 「应用标识」以及 导入资源教程 中「control.xml 中对应的 appid」 修改为正确的值，否则运行时会还是会提示 appkey 错误；

### 配置应用标识(Bundle Identifier)
Bundle Identifier为苹果的AppID，必须与应用发布时配置的Profile关联的AppID一致；

### 配置应用名称
Display Name 为应用在App Store中显示的名称，推荐与manifest.json中name值一致。

### 配置应用版本名称
Version为应用版本号，在App Store中显示的版本号，推荐与manifest.json中versionName值一致；

### 配置应用版本号
Build为编译版本号，App Store判断升级使用，推荐与manifest.json中versionCode值一致。

### 配置应用的图标
点击project->target->General->App Icons and Launch Images->App Icons Source项右侧小箭头

对应的尺寸图标读取并复制manifest.json中的 app-plus.distribute.icons.ios 对象，格式如下：
```json
"ios" : {
  "appstore" : "unpackage/res/icons/1024x1024.png",
  "ipad" : {
      "app" : "unpackage/res/icons/76x76.png",
      "app@2x" : "unpackage/res/icons/152x152.png",
      "notification" : "unpackage/res/icons/20x20.png",
      "notification@2x" : "unpackage/res/icons/40x40.png",
      "proapp@2x" : "unpackage/res/icons/167x167.png",
      "settings" : "unpackage/res/icons/29x29.png",
      "settings@2x" : "unpackage/res/icons/58x58.png",
      "spotlight" : "unpackage/res/icons/40x40.png",
      "spotlight@2x" : "unpackage/res/icons/80x80.png"
  },
  "iphone" : {
      "app@2x" : "unpackage/res/icons/120x120.png",
      "app@3x" : "unpackage/res/icons/180x180.png",
      "notification@2x" : "unpackage/res/icons/40x40.png",
      "notification@3x" : "unpackage/res/icons/60x60.png",
      "settings@2x" : "unpackage/res/icons/58x58.png",
      "settings@3x" : "unpackage/res/icons/87x87.png",
      "spotlight@2x" : "unpackage/res/icons/80x80.png",
      "spotlight@3x" : "unpackage/res/icons/120x120.png"
  }
}
```

### 配置应用启动界面
根据manifest.json中的app-plus.distribute.splashscreen.iosStyle值，配置应用启动界面。
- 如果值为`default`，则使用自定义启动界面。
- 如果值为`common`，则使用通用启动图。
- 如果值为`storyboard`，则使用自定义启动界面。

如果值为`storyboard`，则需要读取用户配置的 zip 并解压，获取里面*.storyboard(*：表示任意文件名)文件。
解压后里面会包含*.storyboard文件的和*.storyboard引用的图片文件。
将*.storyboard文件和图片文件添加到 Xcode 项目中，并配置 Launch Images Source：Use Asset Catalog、Launch Screen File：LaunchScreen。
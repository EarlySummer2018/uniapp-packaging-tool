# UTS 内置模块（iOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：iOS (iPhone/iPad)
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/

---

UTS（Uni Type Script）是 DCloud 推出的跨平台开发语言，iOS 端支持通过 UTS 插件扩展原生能力。

## UTS 基础模块

### 需要引入的系统框架

| 框架 | 说明 |
|------|------|
| Foundation.framework | 基础框架 |
| UIKit.framework | UI框架 |
| CoreLocation.framework | 定位（如使用位置相关API） |
| Photos.framework | 相册（如使用图片选择器） |
| AssetsLibrary.framework | 资产库（旧版相册访问） |

### CocoaPods 依赖

```ruby
# UTS 运行时依赖
pod 'UTSPlugin', :path => './SDK/libs/UTSPlugin.podspec'  # 本地路径
```

### 需要拷贝的文件

| 路径 | 文件 |
|------|------|
| SDK/libs | `utsplugin.framework` 或 `libutsplugin.a` |
| SDK/libs | 各个 UTS 内置模块对应的 .framework 文件 |

## UTS 内置模块列表

| 模块名称 | 功能说明 | 依赖的系统框架 |
|---------|---------|--------------|
| uni-getSystemInfo | 获取系统信息 | UIDevice, UIScreen |
| uni-getDeviceInfo | 获取设备信息 | UIDevice |
| uni-getNetworkType | 获取网络类型 | NetworkExtension |
| uni-storage | 本地存储 | Foundation |
| uni-chooseMedia | 选择媒体文件 | Photos, UIImagePickerController |
| uni-installApk | 安装应用（iOS 不适用） | - |
| uni-prompt | 弹窗提示 | UIKit |
| uni-privacy | 隐私管理 | Foundation |
| uni-exit | 退出应用 | UIApplication |
| uni-openAppAuthorizeSetting | 打开授权设置 | UIApplication |
| uni-getAppBaseInfo | 获取应用基础信息 | Bundle |
| uni-createRequestPermissionListener | 权限监听 | Foundation |
| uni-getAccessibilityInfo | 无障碍信息 | UIAccessibility |
| uni-getAppAuthorizeSetting | 应用授权状态 | Foundation |
| uni-getSystemSetting | 系统设置 | Foundation |

## Swift 代码示例（UTS 插件开发）

```swift
import Foundation
import UIKit

// 示例：自定义 UTS 模块
@objc(UTSCustomModule)
class UTSCustomModule: NSObject {
    
    @objc static func require(_ module: String!) -> Any! {
        // 模块导出逻辑
        return nil
    }
    
    @objc func getDeviceInfo(_ callback: @escaping ([String: Any]) -> Void) {
        DispatchQueue.main.async {
            let device = UIDevice.current
            let info: [String: Any] = [
                "model": device.model,
                "systemName": device.systemName,
                "systemVersion": device.systemVersion,
                "name": device.name,
                "identifierForVendor": device.identifierForVendor?.uuidString ?? ""
            ]
            callback(info)
        }
    }
    
    @objc func showAlert(_ title: String, _ message: String) {
        guard let viewController = self.getCurrentVC() else { return }
        
        let alert = UIAlertController(title: title, message: message, preferredStyle: .alert)
        alert.addAction(UIAlertAction(title: "确定", style: .default))
        viewController.present(alert, animated: true)
    }
    
    private func getCurrentVC() -> UIViewController? {
        guard let windowScene = UIApplication.shared.connectedScenes.first as? UIWindowScene,
              let rootViewController = windowScene.windows.first?.rootViewController else {
            return nil
        }
        return self.getTopVC(from: rootViewController)
    }
    
    private func getTopVC(from vc: UIViewController) -> UIViewController? {
        if let presentedVC = vc.presentedViewController {
            return getTopVC(from: presentedVC)
        }
        if let nav = vc as? UINavigationController {
            return getTopVC(from: nav.visibleViewController ?? nav)
        }
        if let tab = vc as? UITabBarController {
            return getTopVC(from: tab.selectedViewController ?? tab)
        }
        return vc
    }
}
```

## ⚠️ UTS 开发注意事项

1. **Swift/Objective-C 混编**：UTS 插件可以使用 Swift 或 Objective-C 编写，但需要配置 Bridging Header
2. **内存管理**：注意循环引用问题，合理使用 weak/unowned 引用
3. **线程安全**：涉及 UI 操作必须在主线程执行
4. **版本兼容**：UTS 插件需要适配多个 iOS 版本，使用 @available 检查 API 可用性
5. **调试技巧**：使用 NSLog 或 os.log 输出调试信息，配合 Console.app 查看

---

## 交叉引用

- 上一篇：[UIWebview 配置](uiwebview.md)
- 下一篇：[Geolocation（定位）](geolocation.md)
- 相关模块：所有模块均可通过 UTS 插件扩展原生能力

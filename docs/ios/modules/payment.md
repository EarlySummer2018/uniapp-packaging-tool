# Payment（支付）（iOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：iOS (iPhone/iPad)
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/
>
> **最后更新**：2026-05-29
>
> ⚠️ **说明**：官方文档页面曾返回502错误，以下为基于 Android 版本的参考配置，实际使用时请以官方最新文档为准。
>
> **功能概述**: Payment 模块提供统一的移动支付能力，支持支付宝、微信支付、苹果应用内购(IAP)、Apple Pay、PayPal、Stripe 等主流支付平台。通过统一的 API 接口，开发者可以快速集成多种支付方式。

---

## 一、feature.plist 支付平台参数配置

Payment 模块需要在 `feature.plist` 中声明支持的支付平台。根据业务需求选择对应的支付模块：

| 支付方式 | module name | class | 适用场景 |
|---------|------------|-------|---------|
| **支付宝** | AliPay | io.dcloud.feature.payment.alipay.AliPay | 国内主流支付 |
| **微信支付** | Payment-Weixin | io.dcloud.feature.payment.weixin.WeiXinPay | 国内主流支付 |
| **苹果应用内购** | Payment-IAP | io.dcloud.feature.payment.iap.IapFeature | 虚拟商品/订阅 |
| **Apple Pay** | Payment-ApplePay | io.dcloud.feature.payment.applepay.ApplePayFeature | 实体商品/线下支付 |
| **PayPal** | Payment-PayPal | io.dcloud.feature.payment.paypal.PayPalFeature | 海外市场 |

### feature.plist 完整配置示例

```xml
<features>
    <!-- Payment 主模块 -->
    <feature name="Payment" value="io.dcloud.feature.payment.PaymentFeatureImpl">
        <!-- 支付宝支付 -->
        <module name="AliPay" value="io.dcloud.feature.payment.alipay.AliPay"/>
        <!-- 微信支付 -->
        <module name="Payment-Weixin" value="io.dcloud.feature.payment.weixin.WeiXinPay"/>
        <!-- 苹果应用内购 IAP -->
        <module name="Payment-IAP" value="io.dcloud.feature.payment.iap.IapFeature"/>
        <!-- Apple Pay（可选） -->
        <module name="Payment-ApplePay" value="io.dcloud.feature.payment.applepay.ApplePayFeature"/>
        <!-- PayPal（可选，海外市场） -->
        <module name="Payment-PayPal" value="io.dcloud.feature.payment.paypal.PayPalFeature"/>
        <!-- Stripe（可选，海外市场） -->
        <module name="Payment-Stripe" value="io.dcloud.feature.payment.stripe.StripeFeature"/>
    </feature>
</features>
```

---

## 二、支付宝 AlipaySDK 集成

### 2.1 系统依赖库（Link Binary With Libraries）

| 库文件 | 类型 | 说明 |
|-------|------|------|
| UIKit.framework | System | UI 基础框架 |
| Foundation.framework | System | 基础框架 |
| SystemConfiguration.framework | System | 网络配置检测 |
| CoreTelephony.framework | System | 电话信息（安全校验） |
| QuartzCore.framework | System | 图形渲染引擎 |
| CoreGraphics.framework | System | 图形绘制基础 |
| Security.framework | System | 安全服务（签名验证） |
| CoreMotion.framework | System | 运动传感器（安全校验） |
| libc++.tbd | System | C++ 运行时支持 |

### 2.2 CocoaPods 依赖

```ruby
# Podfile
platform :ios, '12.0'

target 'YourApp' do
  # 支付宝 SDK（官方推荐版本）
  pod 'AlipaySDK-iOS', '~> 15.8.10'
end
```

### 2.3 Info.plist 配置

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <!-- ==================== 支付宝 URL Scheme 配置 ==================== -->
    <key>CFBundleURLTypes</key>
    <array>
        <dict>
            <key>CFBundleURLSchemes</key>
            <array>
                <!-- 格式：ap + 您的支付宝 AppID -->
                <!-- 示例：ap2019082567890123 -->
                <string>ap%您的支付宝AppID%</string>
            </array>
            <key>CFBundleURLName</key>
            <string>alipay</string>
        </dict>
    </array>

    <!-- 白名单查询（iOS 9+ 必需） -->
    <key>LSApplicationQueriesSchemes</key>
    <array>
        <string>alipay</string>
        <string>alipays</string>
    </array>

    <!-- 网络安全配置（开发阶段可开启，生产环境建议关闭） -->
    <key>NSAppTransportSecurity</key>
    <dict>
        <key>NSAllowsArbitraryLoads</key>
        <true/>
    </dict>
</dict>
</plist>
```

### 2.4 工程配置注意事项

1. **Other Linker Flags**: 添加 `-ObjC`
2. **Bitcode**: 设置为 NO（AlipaySDK 暂不支持 Bitcode）
3. **Deployment Target**: iOS 12.0 或更高版本

---

## 三、微信支付 WXApi 集成

### 3.1 SDK 版本选择

| SDK 版本 | 支持平台 | 推荐场景 | CocoaPods |
|---------|---------|---------|-----------|
| WechatOpenSDK 1.9.2 | 微信支付 + 分享 + 登录 | 通用方案 | `pod 'WechatOpenSDK', '1.9.2'` |
| WechatOpenSDK-XCShell 2.0.4 | 微信支付 + 分享 + 登录（XCFramework） | 推荐（M1/M2 Mac 友好） | `pod 'WechatOpenSDK-XCShell', '2.0.4'` |
| WechatOpenSDK_MiniProg | 小程序跳转 | 特殊需求 | 单独引入 |

> **推荐使用 XCShell 版本**：兼容 Xcode 15+ 和 Apple Silicon Mac，避免编译警告。

### 3.2 系统依赖库

| 库文件 | 说明 |
|-------|------|
| UIKit.framework | UI 框架 |
| Foundation.framework | 基础框架 |
| CoreTelephony.framework | 电话信息（用于安全校验） |
| Security.framework | 安全服务 |
| libc++.tbd | C++ 运行时 |
| CoreGraphics.framework | 图形处理（分享图片时需要） |
| WebKit.framework | 浏览器组件（小程序场景） |

### 3.3 Info.plist 配置（6 步完成）

```xml
<!-- Step 1: 微信 URL Scheme -->
<key>CFBundleURLTypes</key>
<array>
    <dict>
        <key>CFBundleURLSchemes</key>
        <array>
            <!-- 替换为您在微信开放平台注册的 AppID -->
            <!-- 示例：wxd930ea5d5a258f4f -->
            <string>%您的微信AppID%</string>
        </array>
        <key>CFBundleURLName</key>
        <string>wechat</string>
    </dict>
</array>

<!-- Step 2: 白名单查询（必需） -->
<key>LSApplicationQueriesSchemes</key>
<array>
    <string>weixin</string>
    <string>weixinULAPI</string>
    <string>weixinuniversallink</string>
</array>

<!-- Step 3: Universal Links（强烈推荐，iOS 9+） -->
<key>com.apple.developer.associated-domains</key>
<array>
    <!-- 替换为您的域名，需配置 apple-app-site-association 文件 -->
    <string>applinks:%您的域名%</string>
</array>

<!-- Step 4: 网络安全配置 -->
<key>NSAppTransportSecurity</key>
<dict>
    <key>NSAllowsArbitraryLoads</key>
    <true/>
</dict>

<!-- Step 5: 相册权限（如果涉及分享图片到朋友圈） -->
<key>NSPhotoLibraryUsageDescription</key>
<string>需要访问相册以选择分享的图片</string>

<!-- Step 6: 相机权限（如果涉及扫码等场景） -->
<key>NSCameraUsageDescription</key>
<string>需要使用相机进行扫码</string>
```

### 3.4 Universal Links 配置详解

**为什么需要 Universal Links？**
- iOS 9+ 系统限制了 URL Scheme 的调用方式
- 微信支付回调必须通过 Universal Links 才能稳定工作
- 避免 iOS 13+ 的弹窗确认提示

**配置步骤**:

1. 在 [微信开放平台](https://open.weixin.qq.com) 注册应用并获取 AppID
2. 在服务器根目录或 `.well-known` 目录部署 `apple-app-site-association` 文件：

```json
{
    "applinks": {
        "apps": [],
        "details": [
            {
                "appID": "TEAM_ID.com.yourcompany.app",
                "paths": ["/app/*", "/wxapi/*"]
            }
        ]
    }
}
```

3. 在 Xcode → Signing & Capabilities → Associated Domains 中添加域名：
   ```
   applinks:yourdomain.com
   ```

### 3.5 Objective-C 回调代码

```objc
// AppDelegate.m
#import "WXApi.h"
#import "WXApiObject.h"

@interface AppDelegate () <WXApiDelegate>
@end

@implementation AppDelegate

- (BOOL)application:(UIApplication *)application didFinishLaunchingWithOptions:(NSDictionary *)launchOptions {
    
    // ====== 初始化微信 SDK ======
    // 参数1: 微信开放平台注册的 AppID
    // 参数2: Universal Link 地址（必须与 associated-domains 匹配）
    [WXApi registerApp:@"wx%您的AppID%" universalLink:@"https://%您的域名%/app/"];
    
    return YES;
}

// ====== 处理微信回调（iOS 9+ 方法）======
- (BOOL)application:(UIApplication *)app openURL:(NSURL *)url options:(NSDictionary<NSString *,id> *)options {
    return [WXApi handleOpenURL:url delegate:self];
}

#pragma mark - WXApiDelegate 必需方法

// ====== 微信回调处理 ======
- (void)onResp:(BaseResp *)resp {
    if ([resp isKindOfClass:[PayResp class]]) {
        PayResp *payResp = (PayResp *)resp;
        
        switch (payResp.errCode) {
            case WXSuccess:
                NSLog(@"✅ 微信支付成功");
                // TODO: 通知前端支付成功，同时向服务端验证订单
                break;
                
            case WXErrCodeCommon:
                NSLog(@"❌ 微信支付错误：%@", payResp.errStr);
                break;
                
            case WXErrCodeUserCancel:
                NSLog(@"⚠️ 用户取消支付");
                break;
                
            case WXErrCodeSentFail:
                NSLog(@"❌ 发送失败");
                break;
                
            case WXErrCodeAuthDeny:
                NSLog(@"❌ 授权被拒绝");
                break;
                
            case WXErrCodeUnsupport:
                NSLog(@"❌ 微信不支持");
                break;
                
            default:
                NSLog(@"❌ 未知错误码：%d", payResp.errCode);
                break;
        }
    }
}

@end
```

---

## 四、苹果应用内购 IAP（In-App Purchase）

> **重要**: 销售虚拟商品（游戏道具、会员订阅、解锁功能等）**必须**使用 IAP，禁止使用其他支付方式，否则会被 App Store 拒绝审核。

### 4.1 系统框架

| 框架 | 说明 |
|------|------|
| StoreKit.framework | 应用内购核心框架（必需） |

### 4.2 工程配置

1. **Xcode → Signing & Capabilities → + Capability**: 添加 **In-App Purchase**
2. 在 [App Store Connect](https://appstoreconnect.apple.com) 创建产品和订阅
3. 配置沙盒测试账号（Settings → Sandbox → Testers）

### 4.3 Info.plist 配置

```xml
<!-- IAP 通常无需额外 Info.plist 配置 -->
<!-- 但建议添加网络权限说明 -->
<key>NSAppTransportSecurity</key>
<dict>
    <key>NSAllowsArbitraryLoads</key>
    <true/>
</dict>
```

### 4.4 StoreKit 代码示例（Swift）

```swift
import StoreKit
import UIKit

class IAPManager: NSObject, SKProductsRequestDelegate, SKPaymentTransactionObserver {
    
    static let shared = IAPManager()
    private var productsRequest: SKProductsRequest?
    private var products: [SKProduct] = []
    
    private override init() {
        super.init()
        // 监听交易状态
        SKPaymentQueue.default().add(self)
    }
    
    deinit {
        SKPaymentQueue.default().remove(self)
    }
    
    // MARK: - 获取产品列表
    
    func fetchProducts(productIds: Set<String>) {
        let request = SKProductsRequest(productIdentifiers: productIds)
        request.delegate = self
        self.productsRequest = request
        request.start()
    }
    
    func productsRequest(_ request: SKProductsRequest, didReceive response: SKProductsResponse) {
        self.products = response.products
        
        for product in response.products {
            print("📦 产品名称: \(product.localizedTitle)")
            print("💰 价格: \(product.price)")
            print("🆔 Product ID: \(product.productIdentifier)")
        }
        
        if !response.invalidProductIdentifiers.isEmpty {
            print("⚠️ 无效的产品ID: \(response.invalidProductIdentifiers)")
        }
    }
    
    // MARK: - 发起购买
    
    func purchaseProduct(productId: String) {
        guard let product = products.first(where: { $0.productIdentifier == productId }) else {
            print("❌ 未找到产品")
            return
        }
        
        let payment = SKPayment(product: product)
        SKPaymentQueue.default().add(payment)
    }
    
    // MARK: - SKPaymentTransactionObserver
    
    func paymentQueue(_ queue: SKPaymentQueue, updatedTransactions transactions: [SKPaymentTransaction]) {
        for transaction in transactions {
            switch transaction.transactionState {
            case .purchased:
                print("✅ 购买成功: \(transaction.payment.productIdentifier)")
                completeTransaction(transaction)
                
            case .failed:
                if let error = transaction.error as? SKError {
                    print("❌ 购买失败: \(error.code.rawValue) - \(error.localizedDescription)")
                    
                    if error.code == .paymentCancelled {
                        print("⚠️ 用户取消购买")
                    }
                }
                queue.finishTransaction(transaction)
                
            case .restored:
                print("🔄 恢复购买: \(transaction.payment.productIdentifier)")
                queue.finishTransaction(transaction)
                
            case .purchasing:
                print("⏳ 正在处理...")
                
            default:
                break
            }
        }
    }
    
    private func completeTransaction(_ transaction: SKPaymentTransaction) {
        // TODO: 向服务端验证收据（Receipt Validation）
        verifyReceipt(transaction: transaction)
        
        SKPaymentQueue.default().finishTransaction(transaction)
    }
    
    private func verifyReceipt(transaction: SKPaymentTransaction) {
        guard let receiptURL = Bundle.main.appStoreReceiptURL,
              let receipt = try? Data(contentsOf: receiptURL) else {
            print("❌ 无法获取收据")
            return
        }
        
        // 将收据发送到你的服务器进行验证
        // 服务器端应调用 Apple 的 verifyReceipt 接口
        print("📝 收据数据长度: \(receipt.count) bytes")
        
        // TODO: 实现 HTTP 请求将 receipt 发送到后端
    }
    
    // MARK: - 恢复购买（针对非消耗型产品）
    
    func restorePurchases() {
        SKPaymentQueue.default().restoreCompletedTransactions()
    }
}
```

---

## 五、PayPal 支付（海外市场）

> **适用场景**: 面向欧美市场的应用，支持信用卡、借记卡、PayPal余额等多种支付方式。

### 5.1 前提条件

- **最低系统版本**: iOS 13.0+
- **Xcode 版本**: 14.0 或更高
- **PayPal 开发者账号**: 注册地址 https://developer.paypal.com/

### 5.2 CocoaPods 依赖

```ruby
# PayPal Checkout SDK（最新版）
pod 'PayPalCheckout', '~> 1.2.0'
```

### 5.3 系统依赖库

| 库文件 | 说明 |
|-------|------|
| SafariServices.framework | Safari 网页视图（Web 支付流程） |

### 5.4 Info.plist 配置

```xml
<key>CFBundleURLTypes</key>
<array>
    <dict>
        <key>CFBundleURLSchemes</key>
        <array>
            <!-- PayPal 返回的 URL Scheme，通常由 SDK 自动生成 -->
            <string>%PayPal-Return-URL-Scheme%</string>
        </array>
    </dict>
</array>

<key>LSApplicationQueriesSchemes</key>
<array>
    <string>paypal</string>
    <string>paypalsandbox</string>
</array>
```

### 5.5 PayPal 初始化代码（Swift）

```swift
import PayPalCheckout

@main
class AppDelegate: UIResponder, UIApplicationDelegate {
    
    func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        
        // 配置 PayPal SDK
        // 注意：clientID 从 PayPal Developer Dashboard 获取
        // sandbox 用于测试环境，production 用于正式环境
        Checkout.config(
            clientID: "%您的PayPal客户端ID%",
            environment: .sandbox  // 正式环境改为 .production
        )
        
        return true
    }
}
```

---

## 六、Stripe 支付（海外市场）

> **适用场景**: 全球化应用，支持 135+ 种货币，提供完整的支付解决方案。

### 6.1 前提条件

- **最低系统版本**: iOS 13.0+
- **Stripe 账号**: 注册地址 https://dashboard.stripe.com/register

### 6.2 CocoaPods 依赖（8 个核心 xcframework）

```ruby
# Stripe iOS SDK（包含所有核心模块）
pod 'Stripe', '~> 24.0.0'

# 如果只需要部分功能，可单独引入以下子模块：
# pod 'StripeCore', '~> 24.0.0'
# pod 'StripePayments', '~> 24.0.0'
# pod 'StripePaymentsUI', '~> 24.0.0'
# pod 'StripeApplePay', '~> 24.0.0'
# pod 'StripeFinancialConnections', '~> 24.0.0'
# pod 'StripeIdentity', '~> 24.0.0'
# pod 'StripeTerminal', '~> 24.0.0'  # 硬件读卡器（线下场景）
```

### 6.3 Stripe SDK 包含的核心组件

| 组件 | 功能 | 是否必需 |
|------|------|---------|
| StripeCore | 核心 HTTP 网络层和模型 | ✅ 是 |
| StripePayments | 支付创建和处理逻辑 | ✅ 是 |
| StripePaymentsUI | 预构建的支付表单 UI | ✅ 是 |
| StripeApplePay | Apple Pay 集成 | 可选 |
| StripeFinancialConnections | 银行账户关联（美国） | 可选 |
| StripeIdentity | 身份验证（KYC） | 可选 |
| StripeTerminal | 线下硬件支付终端 | 可选 |

### 6.4 系统依赖库

| 库文件 | 说明 |
|-------|------|
| PassKit.framework | Apple Pay 支持（可选） |
| Security.framework | 加密和安全服务 |
| Foundation.framework | 基础框架 |
| UIKit.framework | UI 组件 |

### 6.5 Info.plist 配置

```xml
<!-- Apple Pay 支持（如果启用） -->
<key>CFBundleURLTypes</key>
<array>
    <dict>
        <key>CFBundleURLSchemes</key>
        <array>
            <string>%Stripe-Merchant-ID%</string>
        </array>
    </dict>
</array>

<!-- 网络权限 -->
<key>NSAppTransportSecurity</key>
<dict>
    <key>NSAllowsArbitraryLoads</key>
    <true/>
</dict>
```

### 6.6 Stripe 初始化代码（Swift）

```swift
import Stripe
import StripePaymentsUI

@main
class AppDelegate: UIResponder, UIApplicationDelegate {
    
    func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        
        // 使用你的 Stripe 可发布密钥（测试用 test_，生产用 live_）
        STPAPIClient.shared.publishableKey = "%您的Stripe-Publishable-Key%"
        
        return true
    }
}
```

---

## 七、AppDelegate.m 回调处理代码（完整版）

> **重要**: 以下两个方法是处理所有支付回调的**必需代码**，请确保正确实现。

```objc
// ============================================================
//  AppDelegate.m - 支付回调统一处理
// ============================================================

#import "AppDelegate.h"
#import <AlipaySDK/AlipaySDK.h>           // 支付宝
#import "WXApi.h"                          // 微信
#import "WXApiObject.h"                    // 微信对象

@interface AppDelegate () <WXApiDelegate>
@end

@implementation AppDelegate

- (BOOL)application:(UIApplication *)application didFinishLaunchingWithOptions:(NSDictionary *)launchOptions {
    
    // ====== 1. 初始化微信 SDK ======
    [WXApi registerApp:@"wx%您的微信AppID%" 
          universalLink:@"https://%您的域名%/app/"];
    
    return YES;
}


// ============================================================
//  必需方法 1: 处理第三方 App 回调（iOS 9+）
//  支付宝、微信等都会通过此方法返回结果
// ============================================================
- (BOOL)application:(UIApplication *)app 
            openURL:(NSURL *)url 
            options:(NSDictionary<UIApplicationOpenURLOptionsKey,id> *)options {
    
    NSString *scheme = url.scheme;
    NSString *host = url.host;
    
    NSLog(@"📱 收到回调 - Scheme: %@, Host: %@", scheme, host);
    
    
    // ====== 处理支付宝回调 ======
    if ([scheme hasPrefix:@"ap"] || [url.absoluteString containsString:@"safepay"]) {
        [[AlipaySDK defaultService] processOrderWithPaymentResult:url standbyCallback:^(NSDictionary *resultDic) {
            
            NSInteger resultStatus = [resultDic[@"resultStatus"] integerValue];
            NSString *memo = resultDic[@"memo"] ?: @"";
            
            switch (resultStatus) {
                case 9000:
                    NSLog(@"✅ 支付宝支付成功");
                    // TODO: 发送通知给前端 + 服务端验证订单
                    break;
                    
                case 6001:
                    NSLog(@"⚠️ 支付宝：用户取消支付");
                    break;
                    
                case 6002:
                    NSLog(@"❌ 支付宝：网络连接出错");
                    break;
                    
                case 4000:
                    NSLog(@"❌ 支付宝：订单支付失败");
                    break;
                    
                case 5000:
                    NSLog(@"❌ 支付宝：重复请求");
                    break;
                    
                case 6000:
                    NSLog(@"❌ 支付宝：用户未支付（中途退出）");
                    break;
                    
                default:
                    NSLog(@"❌ 支付宝：未知状态 %ld - %@", resultStatus, memo);
                    break;
            }
        }];
        return YES;
    }
    
    
    // ====== 处理微信回调 ======
    if ([scheme isEqualToString:@"wx%您的微信AppID%"] || 
        [host isEqualToString:@"oauth"] || 
        [host isEqualToString:@"pay"]) {
        
        BOOL success = [WXApi handleOpenURL:url delegate:self];
        if (!success) {
            NSLog(@"❌ 微信回调处理失败");
        }
        return success;
    }
    
    
    // ====== 其他支付方式回调（PayPal、Stripe 等）=====
    // 可在此处扩展
    
    
    NSLog(@"⚠️ 未识别的回调来源: %@", url.absoluteString);
    return NO;
}

// ============================================================
//  必需方法 2: WXApiDelegate - 微信支付结果回调
// ============================================================
#pragma mark - WXApiDelegate

- (void)onResp:(BaseResp *)resp {
    
    // 仅处理支付响应
    if (![resp isKindOfClass:[PayResp class]]) {
        return;
    }
    
    PayResp *payResp = (PayResp *)resp;
    NSInteger errCode = payResp.errCode;
    NSString *errStr = payResp.errStr ?: @"";
    
    NSLog(@"📲 微信支付回调 - errCode: %ld, errStr: %@", (long)errCode, errStr);
    
    switch (errCode) {
        case WXSuccess:               // 0
            NSLog(@"✅ 微信支付成功");
            // ⚠️ 重要：必须向服务端验证支付结果！
            // 不要仅依赖客户端回调判断支付成功
            break;
            
        case WXErrCodeCommon:         // -1
            NSLog(@"❌ 微信支付一般错误: %@", errStr);
            break;
            
        case WXErrCodeUserCancel:     // -2
            NSLog(@"⚠️ 用户取消支付");
            break;
            
        case WXErrCodeSentFail:       // -3
            NSLog(@"❌ 发送失败");
            break;
            
        case WXErrCodeAuthDeny:       // -4
            NSLog(@"❌ 授权拒绝");
            break;
            
        case WXErrCodeUnsupport:      // -5
            NSLog(@"❌ 微信不支持");
            break;
            
        default:
            NSLog(@"❌ 未知错误码: %ld", (long)errCode);
            break;
    }
}

@end
```

---

## 八、关键注意事项汇总

### 8.1 安全性（最高优先级）⚠️

| 要点 | 说明 | 后果 |
|------|------|------|
| **签名必须在服务端生成** | 订单签名的私钥绝对不能放在客户端代码中 | ❌ 应用被拒 + 资金损失风险 |
| **支付结果必须服务端验证** | 不能仅信任客户端回调结果 | ❌ 可能被篡改导致资金损失 |
| **HTTPS 强制要求** | 所有支付相关请求必须使用 HTTPS | ❌ 中间人攻击风险 |
| **日志脱敏** | 生产环境不要打印完整的订单号、签名等信息 | ❌ 信息泄露风险 |

### 8.2 Apple 审核规范

| 规则 | 详细说明 |
|------|---------|
| **IAP 强制要求** | 虚拟商品必须使用 In-App Purchase，否则直接拒绝 |
| **Apple Pay 限制** | 只能用于实体商品和服务，虚拟商品禁用 |
| **测试账号** | 提交审核时必须提供有效的沙盒测试账号 |
| **支付截图** | 需要提供完整的支付流程截图作为审核材料 |
| **恢复购买按钮** | 非消耗型产品必须提供「恢复购买」入口 |

### 8.3 回调处理的常见陷阱

| 问题 | 解决方案 |
|------|---------|
| **支付宝回调丢失** | 确保 URL Scheme 配置正确（格式：`ap` + AppID） |
| **微信无法拉起** | 检查 Universal Links 是否生效（用 Apple 验证工具检测） |
| **iOS 13+ 弹窗问题** | 必须配置 Universal Links，不能只用 URL Scheme |
| **多次回调重复处理** | 服务端做幂等性校验，避免重复发货 |
| **后台被杀后回调丢失** | 使用 `processOrderWithPaymentResult:` 的 standbyCallback |

### 8.4 版本兼容性

| SDK | 最低 iOS 版本 | Xcode 要求 | Bitcode |
|-----|--------------|-----------|---------|
| AlipaySDK 15.8.x | iOS 12.0+ | Xcode 13+ | ❌ 不支持 |
| WechatOpenSDK 1.9.2 | iOS 9.0+ | Xcode 11+ | ⚠️ 部分支持 |
| WechatOpenSDK-XCShell 2.0.4 | iOS 11.0+ | Xcode 14+ | ✅ 支持 |
| PayPalCheckout 1.2.0 | iOS 13.0+ | Xcode 14+ | ✅ 支持 |
| Stripe 24.0.0 | iOS 13.0+ | Xcode 14+ | ✅ 支持 |
| StoreKit (IAP) | iOS 13.0+ | Xcode 11+ | ✅ 支持 |

### 8.5 调试技巧

1. **支付宝沙盒测试**: 使用支付宝提供的沙盒环境，无需真实扣款
2. **微信沙盒测试**: 微信开放平台提供测试白名单机制
3. **IAP 沙盒测试**: 使用 App Store Connect 的 Sandbox Tester 账号
4. **StoreKit 测试文件**: Xcode → File → New → StoreKit Configuration File，可本地模拟 IAP 流程
5. **日志开关**: 开发阶段开启详细日志，发布前务必关闭

### 8.6 常见错误排查

| 错误现象 | 可能原因 | 解决方案 |
|---------|---------|---------|
| 支付宝返回 4000 | 订单参数错误 | 检查 orderString 格式和签名 |
| 微信返回 -1 | 签名错误或 AppID 不匹配 | 核对服务端签名算法和 AppID |
| IAP 产品无效 | Product ID 未在 App Store Connect 创建 | 检查 Products 配置是否生效 |
| Apple Pay 无法使用 | 未添加 Capability 或 Merchant ID 错误 | Xcode → Signing & Capabilities 检查 |
| PayPal 网页打不开 | Client ID 无效或环境配置错误 | 确认 Sandbox/Live 环境对应正确的 Client ID |
| Stripe 卡片输入异常 | Publishable Key 错误 | 检查 Dashboard 密钥配置 |

---

## 交叉引用

- 上一篇：[Geolocation（定位）](geolocation.md)
- 下一篇：[FAQ - iOS 注意事项](../faq.md)
- 相关模块：[Share（分享）](share.md)（微信/QQ/Facebook 可复用 SDK）、[Oauth（登录鉴权）](oauth.md)、[第三方 SDK 依赖说明](third-party-dependencies.md)

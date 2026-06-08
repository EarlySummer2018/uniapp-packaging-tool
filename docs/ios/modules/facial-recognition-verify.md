# FacialRecognitionVerify（实人认证）（iOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：iOS (iPhone/iPad)
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/

---

iOS 实人认证模块用于身份验证场景（如金融开户、实名认证等）。

## 需要引入的系统框架

| 框架 | 说明 |
|------|------|
| AVFoundation.framework | 人脸检测与图像采集 |
| CoreGraphics.framework | 图形绘制 |
| CoreImage.framework | 图像处理 |
| Vision.framework | 苹果视觉框架（人脸识别） |
| UIKit.framework | UI组件 |
| Foundation.framework | 基础框架 |
| Security.framework | 安全加密 |
| libc++.tbd | C++ 运行时 |

## Info.plist 配置

```xml
<!-- 相机权限 -->
<key>NSCameraUsageDescription</key>
<string>我们需要使用摄像头进行人脸识别验证</string>

<!-- 网络权限 -->
<key>NSAppTransportSecurity</key>
<dict>
    <key>NSAllowsArbitraryLoads</key>
    <true/>
</dict>
```

## CocoaPods 依赖

```ruby
pod 'DCFaceRecognitionVerify', '~> 1.x.x'  # DCloud实人认证SDK（具体版本以官方为准）
```

## 需要拷贝的文件

| 路径 | 文件 |
|------|------|
| SDK/libs | `DCFaceRecognitionVerify.framework` 或相关静态库 |

## Objective-C 代码示例

```objc
#import <DCFaceRecognitionVerify/DCFaceRecognitionVerify.h>

// 发起实人认证
- (void)startFaceVerification {
    DCVerifyConfig *config = [[DCVerifyConfig alloc] init];
    config.verifyToken = @"从服务器获取的verifyToken";
    
    DCFaceRecognitionVerify *verifier = [[DCFaceRecognitionVerify alloc] init];
    [verifier startVerify:config completion:^(BOOL success, NSDictionary *result, NSError *error) {
        if (success) {
            NSLog(@"认证成功：%@", result);
        } else {
            NSLog(@"认证失败：%@", error.localizedDescription);
        }
    }];
}
```

## dcloud_properties.xml 配置

```xml
<feature name="FacialRecognitionVerify" value="io.dcloud.feature.face.FaceRecognitionVerifyFeatureImpl"/>
```

## ⚠️ 实人认证注意事项

1. **实名备案**：使用实人认证功能需要进行企业实名认证
2. **安全合规**：人脸数据属于敏感信息，需符合《个人信息保护法》要求
3. **活体检测**：建议开启活体检测功能防止照片攻击
4. **网络环境**：认证过程需要联网，且对网络质量有要求
5. **真机测试**：模拟器不支持相机调用，必须使用真机测试

---

## 交叉引用

- 上一篇：[Statistic（统计）](statistic.md)
- 下一篇：[uni-AD（广告）](uni-ad.md)
- 相关模块：[LivePusher（直播推流）](livepusher.md)（同样需要相机权限）

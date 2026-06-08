# 常见问题 FAQ 与对比参考（HarmonyOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：HarmonyOS Next
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/harmonyModuleConfig/

---

## ❓ 常见问题 FAQ

### Q1: ohpm 包下载失败怎么办？

**A:** 可能的原因及解决方案：

1. **网络问题**
   ```bash
   # 检查网络连接
   ping repo.harmony.cn
   
   # 如果访问不通，可以尝试配置代理或切换网络
   ```

2. **ohpm 源配置问题**
   ```bash
   # 查看当前配置的源
   ohpm config get registry
   
   # 设置为官方源
   ohpm config set registry https://repo.harmony.cn/ohpm/
   ```

3. **权限问题**
   ```bash
   # 确保 ohpm 有写入权限
   chmod -R 755 ./oh_modules
   
   # 或者尝试清理缓存后重装
   rm -rf ./oh_modules
   ohpm install
   ```

4. **版本不存在**
   - 确认版本号是否正确
   - 查看 ohpm 仓库确认是否有该版本
   - 尝试使用最新版本

---

### Q2: 注册 Provider 时报错 "module not found"

**A:** 这个错误通常是由于以下原因造成的：

1. **依赖未正确安装**
   ```bash
   # 重新安装依赖
   rm -rf oh_modules
   ohpm install
   
   # 检查 oh_modules 目录是否存在对应的包
   ls oh_modules/@uni_modules/
   ```

2. **导入路径错误**
   ```typescript
   // 错误示例：路径拼写错误
   import { UniOAuthHuaweiProviderImpl } from "@uni_modules/uni-oauth-huawe";  // 少了 i
   
   // 正确示例
   import { UniOAuthHuaweiProviderImpl } from "@uni_modules/uni-oauth-huawei";
   ```

3. **IDE 缓存问题**
   - 在 DevEco Studio 中执行：`File → Invalidate Caches / Restart`
   - 清理并重新构建项目：`Build → Clean Project` → `Build → Rebuild Project`

---

### Q3: 腾讯地图 Key 无效或地图无法显示

**A:** 请按以下步骤排查：

1. **检查 Key 配置**
   ```json
   // 确认 module.json5 中的配置
   {
     "metadata": [
       {
         "name": "TENCENT_MAP_KEY",
         "value": "ABZBZ-XXXXXXXX-XXXXXXXX"  // 确保没有多余的空格或引号
       }
     ]
   }
   ```

2. **检查域名白名单设置**
   - 登录[腾讯位置服务控制台](https://lbs.qq.com/console/mykey.html)
   - 编辑你的 Key
   - **确保"域名白名单"字段为空**或设置为 `*`
   - 保存设置后等待几分钟生效

3. **检查网络连接**
   - 地图使用 Web 方案渲染，需要网络连接
   - 确保设备可以访问外网
   - 检查是否有防火墙或代理阻止了地图资源加载

4. **Key 类型不匹配**
   - 确认申请的是**Web端（JSAPI）**类型的 Key
   - 其他类型（如 Android SDK、iOS SDK）的 Key 不适用于鸿蒙端

---

### Q4: 支付宝支付调用后无反应或报错

**A:** 常见原因及解决方案：

1. **缺少 orderInfo 参数**
   ```typescript
   // 错误：缺少必要参数
   uni.requestPayment({
     provider: 'alipay',
     success: () => {}
   });
   
   // 正确：必须提供 orderInfo（由服务端生成）
   uni.requestPayment({
     provider: 'alipay',
     orderInfo: '服务端生成的支付参数字符串',
     success: () => {}
   });
   ```

2. **orderInfo 格式错误**
   - orderInfo 必须是完整的、符合支付宝规范的字符串
   - 通常由服务端调用支付宝 SDK 生成
   - 不要尝试在客户端手动拼接

3. **应用签名问题**
   - 确保应用签名与支付宝开放平台配置的一致
   - 开发阶段使用 debug 签名，发布时使用 release 签名
   - 可以在支付宝开放平台的"应用信息"中查看绑定的签名

4. **沙箱 vs 生产环境**
   - 开发阶段使用沙箱环境进行测试
   - 上线前切换到生产环境
   - 两者的 AppID 和网关地址不同

---

### Q5: 实人认证调用失败或超时

**A:** 排查步骤：

1. **certifyId 无效或过期**
   - certifyId 有时效性（通常几分钟），过期后需要重新获取
   - 每个 certifyId 只能使用一次
   - 确保在获取 certifyId 后立即调用认证接口

2. **metaInfo 未正确传递**
   ```typescript
   // 确保先获取 metaInfo 并传给服务端
   const metaInfo = await uni.getFacialRecognitionMetaInfo();
   const certifyId = await serverApi.getCertifyId({ metaInfo });
   
   // 然后再启动认证
   uni.startFacialRecognitionVerify({ certifyId });
   ```

3. **相机权限问题**
   - 确保应用有相机权限
   - 在 `module.json5` 中声明权限：
   ```json
   {
     "requestPermissions": [
       {
         "name": "oh.permission.CAMERA",
         "reason": "$string:camera_permission_reason"
       }
     ]
   }
   ```

4. **设备不支持**
   - 部分旧设备可能不支持人脸识别
   - 模拟器通常不支持实人认证
   - 建议在真机上测试

---

### Q6: 编译报错 "Cannot find module"

**A:** 解决方案：

1. **检查 oh_modules 目录**
   ```bash
   # 确认模块是否已安装
   ls -la oh_modules/@uni_modules/
   
   # 如果不存在，重新安装
   ohpm install
   ```

2. **检查 package.json5 路径**
   - 确保 `oh-package.json5` 位于项目根目录
   - 不是 `package.json`（那是 npm 用的）

3. **DevEco Studio 同步问题**
   - 点击菜单：`File → Sync Project with Ohpm Files`
   - 或点击工具栏的同步按钮

4. **清除缓存重建**
   ```bash
   # 删除构建缓存
   rm -rf build/
   rm -rf .preview/
   
   # 重新构建
   hvigorw clean
   hvigorw assembleHap
   ```

---

## 🔄 与其他平台对比

### 平台架构差异对比

| 特性 | Android | iOS | 鸿蒙 (HarmonyOS) |
|------|---------|-----|------------------|
| **包管理工具** | Gradle / Maven | CocoaPods | ohpm |
| **配置文件** | build.gradle, AndroidManifest.xml | Podfile, Info.plist, plist | oh-package.json5, module.json5, index.generated.ets |
| **编程语言** | Java/Kotlin | Objective-C/Swift | ArkTS (TypeScript 扩展) |
| **模块注册方式** | dcloud_properties.xml | Info.plist / Podfile | index.generated.ets |
| **依赖声明** | implementation / compile | pod 'xxx', :path | dependencies in json5 |

### 模块配置方式对比

#### OAuth（登录鉴权）

| 平台 | 配置方式 | 支持的登录方式 |
|------|---------|---------------|
| **Android** | dcloud_properties.xml + aar 文件 | 微信、QQ、微博、小米、Google、Facebook、一键登录等 10+ 种 |
| **iOS** | Info.plist + 框架引入 | 微信、QQ、微博、Apple、Google、Facebook 等 7 种 |
| **鸿蒙** | ohpm + registerUniProvider | **仅支持华为登录** |

**鸿蒙特色**：
- 使用 ohpm 包管理，无需手动拷贝 aar 文件
- 通过 TypeScript 代码注册 Provider，更现代化
- 目前仅支持华为登录，期待后续增加更多登录方式

#### Map（地图）

| 平台 | 配置方式 | 支持的地图 |
|------|---------|----------|
| **Android** | build.gradle + AndroidManifest.xml | 高德、百度、谷歌 |
| **iOS** | Info.plist + 框架引入 | 高德、百度、谷歌 |
| **鸿蒙** | module.json5 metadata | **仅支持腾讯地图（Web 方案）** |

**鸿蒙特色**：
- 采用 Web 方案渲染地图，配置最简单
- 只需在 metadata 中配置 Key 即可
- 性能和功能相比原生方案有一定差距

#### Payment（支付）

| 平台 | 配置方式 | 支持的支付方式 |
|------|---------|---------------|
| **Android** | dcloud_properties.xml + aar/gradle | 支付宝、微信、PayPal、Stripe、Google Pay |
| **iOS** | Info.plist + 框架引入 | 支付宝、微信、Apple Pay |
| **鸿蒙** | ohpm + registerUniProvider | **仅支持支付宝支付** |

**鸿蒙特色**：
- 配置简洁，只需两步（安装依赖 + 注册 Provider）
- 支付安全性依赖服务端实现
- 期待后续支持微信支付等其他支付方式

#### FacialRecognitionVerify（实人认证）

| 平台 | 配置方式 | 实现方式 |
|------|---------|---------|
| **Android** | dcloud_properties.xml + aar | DCloud 实人认证服务 |
| **iOS** | Info.plist + 框架引入 | DCloud 实人认证服务 |
| **鸿蒙** | ohpm + uni 全局方法扩展 | DCloud 实人认证服务 |

**鸿蒙特色**：
- 通过扩展 uni 全局对象的方式暴露 API
- 调用方式与其他平台保持一致
- 代码风格更加统一和现代化

### 鸿蒙配置的独特优势

1. **✅ 声明式依赖管理**
   - 使用 ohpm 统一管理依赖，类似于 npm
   - 版本锁定清晰，避免依赖冲突
   - 自动处理依赖关系

2. **✅ TypeScript 优先**
   - 配置代码使用 TypeScript/ArkTS 编写
   - 类型安全，编译时就能发现错误
   - IDE 支持完善，代码提示友好

3. **✅ 统一的注册入口**
   - 所有模块都在 `index.generated.ets` 中注册
   - 代码结构清晰，易于维护
   - 避免了分散在多个配置文件中的混乱

4. **✅ JSON5 配置格式**
   - 支持注释，可读性强
   - 语法宽松，尾逗号也不会报错
   - 比 JSON 更适合人工编辑

### 鸿蒙配置的当前局限

1. **⚠️ 支持的模块较少**
   - 目前只有 4 个模块可用
   - 常用的推送、统计、分享等模块尚未支持
   - 需要关注官方后续更新

2. **⚠️ 生态不够成熟**
   - 第三方 SDK 适配较少
   - 遇到问题时社区资源有限
   - 部分功能需要自行实现或等待官方支持

3. **⚠️ 调试工具待完善**
   - 相比 Android Studio 和 XCode，DevEco Studio 还在不断改进
   - 部分调试功能可能不如其他平台完善

---

## ⚠️ 重要提示

### 1. oh-package.json5 配置规范

- 所有需要通过 ohpm 安装的模块都必须在 `dependencies` 字段中声明
- 版本号建议使用精确版本（如 `1.0.1`），避免使用范围版本（如 `^1.0.1`）
- 添加依赖后务必执行 `ohpm install` 安装依赖包
- 如果遇到下载失败的问题，请检查网络连接和 ohpm 源配置

### 2. index.generated.ets 注册规范

- 该文件是 uni-app 鸿蒙端的模块入口文件
- 所有第三方模块的注册代码都应放在 `initUniExtApi()` 函数中
- Provider 类型的模块使用 `registerUniProvider()` 注册
- API 类型的模块直接挂载到 `uni` 对象上
- 确保导入路径正确，避免拼写错误

### 3. 腾讯地图 Key 申请要点

- **域名白名单必须留空**（这是最常见的问题）
- Key 与应用绑定，不同环境（开发/生产）建议使用不同的 Key
- 如果地图无法显示，首先检查 Key 是否正确配置

### 4. 当前支持程度说明

> 鸿蒙端目前支持的模块相对较少，后续会持续增加。

目前已支持的模块：
- ✅ OAuth（华为登录）
- ✅ Map（腾讯地图）
- ✅ Payment（支付宝支付）
- ✅ FacialRecognitionVerify（实人认证）

暂未支持的常用模块（可关注官方更新）：
- ❌ 微信登录/支付
- ❌ 微信分享
- ❌ 推送（Push）
- ❌ 统计（Statistic）
- ❌ 其他第三方登录（QQ、微博等）

---

## 📎 交叉引用

- [← 返回鸿蒙模块概览](./index.md)
- [OAuth 登录鉴权](./modules/oauth.md)
- [Map 地图模块](./modules/map.md)
- [Payment 支付模块](./modules/payment.md)
- [FacialRecognitionVerify 实人认证](./modules/facial-recognition-verify.md)

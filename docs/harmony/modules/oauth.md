# OAuth 登录鉴权（HarmonyOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：HarmonyOS Next
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/harmonyModuleConfig/

---

OAuth 模块提供了第三方登录的能力，目前在鸿蒙平台**仅支持华为登录**。

## 📌 模块概览

| 属性 | 说明 |
|------|------|
| **功能描述** | 集成华为账号登录能力 |
| **ohpm 包名** | `@uni_modules/uni-oauth-huawei` |
| **当前版本** | 1.0.1 |
| **注册方式** | `registerUniProvider` |
| **支持程度** | ✅ 完全支持 |

## 🔧 前置条件

在开始配置之前，请确保已完成以下准备工作：

1. **开发环境准备**
   - 已安装 DevEco Studio 4.0+
   - 已配置 HarmonyOS SDK（API 12+）
   - 已安装 ohpm 包管理工具

2. **华为开发者账号**
   - 已注册[华为开发者账号](https://developer.huawei.com/)
   - 已创建应用并获取 AppID
   - 已开通华为账号服务（Account Kit）

3. **项目基础配置**
   - 项目已集成 uni-app 鸿蒙运行时
   - `oh-package.json5` 文件已存在
   - `index.generated.ets` 入口文件已配置

## 📝 配置步骤

### 步骤一：添加 ohpm 依赖

在项目根目录下的 `oh-package.json5` 文件中，添加华为登录模块依赖：

```json
{
  "name": "your-project-name",
  "version": "1.0.0",
  "dependencies": {
    // 其他依赖...
    
    // 华为登录模块 - 版本号请根据实际情况调整
    "@uni_modules/uni-oauth-huawei": "1.0.1"
  }
}
```

> 💡 **提示**：添加依赖后，需要在项目目录下执行 `ohpm install` 命令来下载并安装依赖包。

### 步骤二：注册华为登录 Provider

在 uni_modules 的入口文件 `index.generated.ets` 中，注册华为登录 provider：

```typescript
// 导入 uni-app 运行时核心模块
// registerUniProvider: 用于注册第三方服务提供者
// uni: 全局 uni 对象，用于扩展 API
import { registerUniProvider, uni } from "@dcloudio/uni-app-runtime";

// 导入华为登录 Provider 实现
// UniOAuthHuaweiProviderImpl: 华为登录的具体实现类
import { UniOAuthHuaweiProviderImpl } from "@uni_modules/uni-oauth-huawei";

// 导出初始化函数，供外部调用
export function initUniModules() {
  // 调用扩展 API 初始化函数
  initUniExtApi();
}

// 初始化扩展 API 的函数
function initUniExtApi() {
  // 注册华为登录 Provider
  // 参数说明：
  //   - "oauth": 模块类型标识，表示这是登录鉴权模块
  //   - "huawei": 服务提供商标识，表示使用华为登录
  //   - new UniOAuthHuaweiProviderImpl(): 创建华为登录 Provider 实例
  registerUniProvider("oauth", "huawei", new UniOAuthHuaweiProviderImpl());
}
```

### 步骤三：配置 module.json5（如需要）

如果华为登录需要额外的权限或元数据配置，请在 `entry/src/main/module.json5` 中添加：

```json
{
  "module": {
    // ... 其他配置
    
    // 申请华为登录所需的权限
    "requestPermissions": [
      {
        "name": "oh.permission.INTERNET",
        "reason": "$string:internet_permission_reason",
        "usedScene": {
          "abilities": ["EntryAbility"],
          "when": "always"
        }
      }
    ]
  }
}
```

## ✅ 验证方法

完成上述配置后，可以通过以下方式验证华为登录是否配置成功：

1. **编译验证**
   ```bash
   # 在项目根目录执行编译命令
   hvigorw assembleHap --mode module -p product=default
   ```
   
   如果编译成功且无报错，说明依赖和注册代码正确。

2. **运行时验证**
   
   在页面中调用 uni.login 接口测试：
   ```typescript
   // 测试华为登录
   uni.login({
     provider: 'huawei',  // 指定使用华为登录
     success: (loginRes) => {
       console.log('华为登录成功:', loginRes);
       // 登录成功后的处理逻辑
     },
     fail: (err) => {
       console.error('华为登录失败:', err);
       // 错误处理逻辑
     }
   });
   ```

3. **日志检查**
   
   在 DevEco Studio 的 Log 面板中查看是否有以下关键日志：
   - `[UniOAuth] Huawei provider registered successfully`
   - `[HMS] Account service initialized`

## ⚠️ 注意事项

1. **签名配置**：鸿蒙应用必须正确配置签名文件，否则华为登录无法正常工作
2. **应用审核**：发布前确保已在华为应用市场完成应用审核
3. **用户授权**：首次登录会弹出华为账号授权界面，需引导用户完成授权

---

## 📎 交叉引用

- [← 返回鸿蒙模块概览](../index.md)
- [Map 地图模块](./map.md)
- [Payment 支付模块](./payment.md)
- [FacialRecognitionVerify 实人认证](./facial-recognition-verify.md)
- [常见问题 FAQ](../faq.md)

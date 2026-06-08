# Payment 支付（HarmonyOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：HarmonyOS Next
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/harmonyModuleConfig/

---

## 📌 模块概览

| 属性 | 说明 |
|------|------|
| **功能描述** | 集成支付宝支付能力 |
| **ohpm 包名** | `@uni_modules/uni-payment-alipay` |
| **当前版本** | 1.0.1 |
| **注册方式** | `registerUniProvider` |
| **支持程度** | ✅ 完全支持 |

## 🔧 前置条件

1. **支付宝开放平台账号**
   - 已注册[支付宝开放平台](https://open.alipay.com/)账号
   - 已创建移动应用并获取 AppID
   - 已完成应用签约（手机网站支付或 APP 支付）

2. **证书和密钥**
   - 已生成应用私钥（RSA2 格式）
   - 已上传公钥到支付宝开放平台
   - 已获取支付宝公钥

3. **服务器端配合**
   - 后端接口已实现支付宝订单创建和回调处理
   - 支付结果通知 URL 已配置

## 📝 配置步骤

### 步骤一：添加 ohpm 依赖

在 `oh-package.json5` 文件中添加支付宝支付模块依赖：

```json
{
  "name": "your-project-name",
  "version": "1.0.0",
  "dependencies": {
    // 其他依赖...
    
    // 支付宝支付模块 - 版本号请根据实际情况调整
    "@uni_modules/uni-payment-alipay": "1.0.1"
  }
}
```

执行安装命令：

```bash
ohpm install
```

### 步骤二：注册支付宝支付 Provider

在 `index.generated.ets` 文件中注册支付宝支付 provider：

```typescript
// 导入 uni-app 运行时核心模块
import { registerUniProvider, uni } from "@dcloudio/uni-app-runtime";

// 导入支付宝支付 Provider 实现
// UniPaymentAlipayProviderImpl: 支付宝支付的具体实现类
import { UniPaymentAlipayProviderImpl } from "@uni_modules/uni-payment-alipay";

// 导出初始化函数
export function initUniModules() {
  // 初始化扩展 API
  initUniExtApi();
}

// 初始化扩展 API
function initUniExtApi() {
  // 注册支付宝支付 Provider
  // 参数说明：
  //   - "payment": 模块类型标识，表示这是支付模块
  //   - "alipay": 服务提供商标识，表示使用支付宝支付
  //   - new UniPaymentAlipayProviderImpl(): 创建支付宝支付 Provider 实例
  registerUniProvider("payment", "alipay", new UniPaymentAlipayProviderImpl());
}
```

### 步骤三：配置支付参数（可选）

如果需要在客户端配置支付参数，可以在 `module.json5` 中添加：

```json
{
  "module": {
    "metadata": [
      {
        "name": "ALIPAY_APP_ID",
        "value": "你的支付宝AppID"
      }
    ]
  }
}
```

> ⚠️ **安全提示**：不建议将敏感信息（如私钥）硬编码在客户端。建议通过服务端接口动态获取支付参数。

## ✅ 验证方法

1. **编译验证**

   执行编译命令，确保无错误：
   ```bash
   hvigorw assembleHap --mode module -p product=default
   ```

2. **沙箱环境测试**
   
   支付宝提供了沙箱环境用于开发测试：
   ```typescript
   // 调用支付接口（使用沙箱环境）
   uni.requestPayment({
     provider: 'alipay',           // 指定支付方式为支付宝
     orderInfo: '从服务端获取的支付参数',  // 支付订单信息，由服务端生成
     
     success: function (res) {
       console.log('支付成功:', res);
       // 支付成功后的业务处理
       // 注意：最终支付状态应以服务端异步通知为准
     },
     
     fail: function (err) {
       console.log('支付失败:', err);
       // 支付失败或取消的处理
     }
   });
   ```

3. **真机调试**
   
   - 使用真机进行支付测试（模拟器可能不支持跳转支付宝）
   - 测试正常支付、取消支付、支付失败等场景
   - 验证支付结果回调是否正确触发

## ⚠️ 注意事项

1. **支付安全性**
   - `orderInfo` 参数必须由服务端生成，不要在客户端拼接
   - 不要在客户端存储支付宝私钥等敏感信息
   - 支付结果应以服务端异步通知为准，不能仅依赖客户端返回值

2. **应用签名**
   - 发布版应用必须使用正式签名
   - 确保支付宝开放平台配置的包名和签名与实际应用一致

3. **用户体验**
   - 支付过程中会跳转到支付宝 App 或 H5 页面
   - 需要处理好支付页面的返回逻辑
   - 建议在支付前展示订单详情供用户确认

---

## 📎 交叉引用

- [← 返回鸿蒙模块概览](../index.md)
- [OAuth 登录鉴权](./oauth.md)
- [Map 地图模块](./map.md)
- [FacialRecognitionVerify 实人认证](./facial-recognition-verify.md)
- [常见问题 FAQ](../faq.md)

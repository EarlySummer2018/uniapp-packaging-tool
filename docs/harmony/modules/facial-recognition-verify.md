# FacialRecognitionVerify 实人认证（HarmonyOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：HarmonyOS Next
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/harmonyModuleConfig/

---

## 📌 模块概览

| 属性 | 说明 |
|------|------|
| **功能描述** | 提供实人认证（人脸识别）能力，用于身份核验场景 |
| **ohpm 包名** | `@uni_modules/uni-facialrecognitionverify` |
| **当前版本** | 1.0.2 |
| **注册方式** | uni 全局方法扩展 |
| **支持程度** | ✅ 完全支持 |

## 🔧 前置条件

1. **DCloud 开发者账号**
   - 已在 DCloud 开发者中心开通实人认证服务
   - 已获取实人认证的业务参数

2. **应用场景确认**
   - 金融开户、实名认证等需要验证真实身份的场景
   - 符合相关法律法规要求（如《个人信息保护法》）

3. **用户授权**
   - 需要获得用户同意才能采集人脸信息
   - 需要在隐私政策中明确说明人脸信息的使用目的

## 📝 配置步骤

### 步骤一：添加 ohpm 依赖

在 `oh-package.json5` 文件中添加实人认证模块依赖：

```json
{
  "name": "your-project-name",
  "version": "1.0.0",
  "dependencies": {
    // 其他依赖...
    
    // 实人认证模块 - 版本号请根据实际情况调整
    "@uni_modules/uni-facialrecognitionverify": "1.0.2"
  }
}
```

执行安装命令：

```bash
ohpm install
```

### 步骤二：注册实人认证 API

与 OAuth 和 Payment 模块不同，实人认证模块是通过**扩展 uni 全局方法**的方式注册的。在 `index.generated.ets` 文件中进行如下配置：

```typescript
// 导入 uni-app 运行时核心模块
import { registerUniProvider, uni } from "@dcloudio/uni-app-runtime";

// 导入实人认证的核心方法
// startFacialRecognitionVerify: 启动实人认证流程的方法
// getFacialRecognitionMetaInfo: 获取实人认证所需的元信息（如设备指纹等）
import { 
  startFacialRecognitionVerify, 
  getFacialRecognitionMetaInfo 
} from '@uni_modules/uni-facialrecognitionverify';

// 导出初始化函数
export function initUniModules() {
  // 初始化扩展 API
  initUniExtApi();
}

// 初始化扩展 API
function initUniExtApi() {
  // 将实人认证方法挂载到 uni 全局对象上
  // 这样就可以通过 uni.startFacialRecognitionVerify() 调用了
  
  // 启动实人认证
  // 调用后会打开实人认证界面，引导用户完成人脸识别
  uni.startFacialRecognitionVerify = startFacialRecognitionVerify;
  
  // 获取认证元信息
  // 通常在调用 startFacialRecognitionVerify 之前先调用此方法
  // 获取到的信息需要传给服务端，用于后续的认证流程
  uni.getFacialRecognitionMetaInfo = getFacialRecognitionMetaInfo;
}
```

## ✅ 验证方法

1. **编译验证**
   
   编译项目，确保实人认证模块正确集成。

2. **功能测试**
   
   在页面中调用实人认证接口：
   ```typescript
   async function verifyIdentity() {
     try {
       // 第一步：获取认证元信息
       const metaInfo = await new Promise((resolve, reject) => {
         uni.getFacialRecognitionMetaInfo({
           success: (res) => resolve(res),
           fail: (err) => reject(err)
         });
       });
       
       console.log('获取元信息成功:', metaInfo);
       
       // 第二步：将 metaInfo 发送给你的服务器
       // 服务器会结合 metaInfo 和其他参数生成 certifyId
       const certifyId = await requestCertifyIdFromServer(metaInfo);
       
       // 第三步：启动实人认证
       uni.startFacialRecognitionVerify({
         // 认证 ID，由服务端生成
         certifyId: certifyId,
         
         // 成功回调
         success: (res) => {
           console.log('实人认证成功:', res);
           // 认证成功后的业务处理
           // 建议再次向服务端确认认证结果
         },
         
         // 失败回调
         fail: (err) => {
           console.error('实人认证失败:', err);
           
           // 根据错误码进行相应处理
           if (err.errCode === '10001') {
             // 用户取消认证
             console.log('用户取消了认证');
           } else if (err.errCode === '10002') {
             // 认证超时
             console.log('认证过程超时');
           } else {
             // 其他错误
             console.log('认证出错:', err.errMsg);
           }
         }
       });
       
     } catch (error) {
       console.error('实人认证流程出错:', error);
     }
   }
   
   // 辅助函数：向服务端请求 certifyId
   async function requestCertifyIdFromServer(metaInfo: any): Promise<string> {
     // 这里应该是实际的 HTTP 请求
     // 示例代码，需要替换为你的实际接口
     const response = await fetch('/api/get-certify-id', {
       method: 'POST',
       headers: {
         'Content-Type': 'application/json'
       },
       body: JSON.stringify({
         metaInfo: metaInfo,
         // 其他必要的参数...
       })
     });
     
     const data = await response.json();
     return data.certifyId;
   }
   ```

3. **完整流程测试**
   
   - 测试正常的认证流程
   - 测试用户中途取消的场景
   - 测试网络异常的情况
   - 测试光线不足等极端环境

## ⚠️ 注意事项

1. **合规要求**
   - 必须获得用户的明确授权才能采集人脸信息
   - 人脸信息的采集、存储、使用需符合《个人信息保护法》等法律法规
   - 建议在隐私政策中单独说明人脸识别功能的使用

2. **安全性**
   - `certifyId` 必须由服务端生成，不要在客户端伪造
   - 认证结果应以服务端的异步通知为准
   - 不要缓存或持久化人脸图像数据

3. **用户体验**
   - 认证过程需要良好的光照条件
   - 引导用户正对摄像头，保持适当距离
   - 认证过程通常需要 3-5 秒，请做好等待提示

4. **错误处理**
   - 做好各种异常情况的处理（取消、超时、网络错误等）
   - 提供清晰的错误提示和重试机制
   - 记录认证失败的日志以便排查问题

---

## 📎 交叉引用

- [← 返回鸿蒙模块概览](../index.md)
- [OAuth 登录鉴权](./oauth.md)
- [Map 地图模块](./map.md)
- [Payment 支付模块](./payment.md)
- [常见问题 FAQ](../faq.md)

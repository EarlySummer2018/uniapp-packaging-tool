# Map 地图（HarmonyOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：HarmonyOS Next
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/harmonyModuleConfig/

---

## 📌 模块概览

| 属性 | 说明 |
|------|------|
| **功能描述** | 集成腾讯地图显示和交互能力 |
| **ohpm 包名** | 内置模块（无需额外安装） |
| **当前版本** | 随 uni-app 运行时版本 |
| **注册方式** | metadata 配置 |
| **支持程度** | ✅ 完全支持 |

## 🔧 前置条件

1. **腾讯地图开发者账号**
   - 已注册[腾讯位置服务](https://lbs.qq.com/)开发者账号
   - 已创建应用并获取 Key

2. **Key 申请注意事项**
   - ⚠️ **重要**：申请腾讯地图 Key 时，**域名白名单必须留空**
   - 这是因为鸿蒙端使用 Web 端方案渲染地图
   - 如果填写了域名白名单，可能导致地图无法正常加载

## 📝 配置步骤

### 步骤一：申请腾讯地图 Key

1. 访问[腾讯位置服务控制台](https://lbs.qq.com/console/mykey.html)
2. 点击"创建新密钥"
3. 填写应用名称和相关信息
4. **关键步骤**：在"域名白名单"输入框中**留空**或填入 `*`
5. 提交后获取到 Key（格式类似：`XXXXX-XXXXX-XXXXX-XXXXX-XXXXX`）

### 步骤二：配置 module.json5

在 `entry/src/main/module.json5` 文件的 `module` 节点下，添加腾讯地图 Key 的元数据配置：

```json
{
  "module": {
    // 模块名称
    "name": "entry",
    
    // 模块类型
    "type": "entry",
    
    // ... 其他基础配置
    
    // 元数据配置区域
    // 用于存储应用的配置信息，如 API Key、密钥等
    "metadata": [
      // 其他元数据配置...
      
      // 腾讯地图 Key 配置
      // name: 固定为 "TENCENT_MAP_KEY"，系统通过此名称读取地图 Key
      // value: 替换为你在腾讯位置服务申请的真实 Key
      {
        "name": "TENCENT_MAP_KEY",
        "value": "你的腾讯地图Key"
      }
    ],
    
    // ... 其他配置
  }
}
```

**完整示例**：

```json
{
  "module": {
    "name": "entry",
    "type": "entry",
    "description": "$string:module_desc",
    "mainElement": "EntryAbility",
    "deviceTypes": [
      "phone",
      "tablet"
    ],
    "deliveryWithInstall": true,
    "installationFree": false,
    "pages": "$profile:main_pages",
    "metadata": [
      {
        "name": "TENCENT_MAP_KEY",
        "value": "ABZBZ-1234567890-ABCDEF"
      }
    ],
    "abilities": [
      {
        "name": "EntryAbility",
        "srcEntry": "./ets/entryability/EntryAbility",
        "description": "$string:EntryAbility_desc",
        "icon": "$media:icon",
        "label": "$string:EntryAbility_label",
        "startWindowIcon": "$media:startIcon",
        "startWindowBackground": "$color:start_window_background",
        "exported": true,
        "skills": [
          {
            "entities": [
              "entity.system.home"
            ],
            "actions": [
              "action.system.home"
            ]
          }
        ]
      }
    ]
  }
}
```

## ✅ 验证方法

1. **编译验证**
   
   确保项目可以正常编译，无配置错误。

2. **地图组件验证**
   
   在页面中使用 `<map>` 组件：
   ```vue
   <template>
     <view class="container">
       <!-- 地图组件 -->
       <map 
         :latitude="39.9042" 
         :longitude="116.4074"
         :scale="15"
         style="width: 100%; height: 400px;"
       ></map>
     </view>
   </template>
   ```

3. **API 调用验证**
   
   使用地图相关 API 进行测试：
   ```typescript
   // 打开地图选择位置
   uni.chooseLocation({
     success: function (res) {
       console.log('位置名称：' + res.name);
       console.log('详细地址：' + res.address);
       console.log('纬度：' + res.latitude);
       console.log('经度：' + res.longitude);
     }
   });
   ```

## ⚠️ 注意事项

1. **Web 方案限制**
   - 鸿蒙端地图采用 Web 渲染方案，性能略低于原生实现
   - 不支持部分高级地图功能（如 3D 地图、室内地图等）
   - 需要网络连接才能正常显示地图

2. **Key 安全性**
   - 不要将真实的 Key 提交到公开代码仓库
   - 建议使用环境变量或构建脚本动态注入 Key

3. **域名白名单**
   - 再次强调：申请 Key 时域名白名单必须为空
   - 如果已经填写了白名单，需要重新申请 Key 或修改白名单设置

---

## 📎 交叉引用

- [← 返回鸿蒙模块概览](../index.md)
- [OAuth 登录鉴权](./oauth.md)
- [Payment 支付模块](./payment.md)
- [FacialRecognitionVerify 实人认证](./facial-recognition-verify.md)
- [常见问题 FAQ](../faq.md)

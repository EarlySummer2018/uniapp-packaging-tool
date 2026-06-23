# 配置鸿蒙原生项目

> **适用版本**：HBuilderX 5.0+ / uni-app 5.0+
> **平台**：HarmonyOS
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usesdk/harmony.html

---

本文档介绍如何配置鸿蒙原生项目以支持 uni-app 运行时。

## 前置条件

- 已安装 DevEco Studio
- 已配置 HarmonyOS SDK
- 已创建空的鸿蒙项目

## 配置步骤

### 步骤一：创建空的鸿蒙项目

在 DevEco Studio 中创建一个新的空鸿蒙项目。

### 步骤二：修改 oh-package.json5 添加依赖

修改鸿蒙项目根目录下的 `oh-package.json5` 文件，在 dependencies 中添加 uni-app 运行时依赖：

```json
{
  "dependencies": {
    "@dcloudio/uni-app-runtime": "版本号"
  }
}
```

如下图所示，将版本号替换为实际需要的版本：

![](https://aka.doubaocdn.com/s/HQK71weOrw)

### 步骤三：同步依赖

点击 DevEco Studio 右上角的 **Sync Now** 按钮，并等待同步结束。

### 步骤四：修改 EntryAbility.ets 添加初始化逻辑

打开鸿蒙项目文件 `/entry/src/main/ets/entryability/EntryAbility.ets`，增加 uni-app SDK 初始化逻辑。

完整的 EntryAbility.ets 代码如下：

```typescript
import { UniEntryAbility } from "@dcloudio/uni-app-runtime";
import { initUniModules } from "../uni_modules/index.generated";
import BuildProfile from "BuildProfile";

initUniModules();

export default class EntryAbility extends UniEntryAbility {
  constructor() {
    super("HBuilder", {
      debug: BuildProfile.DEBUG,
    });
  }
}
```

## 代码说明

| 导入项 | 说明 |
|--------|------|
| `UniEntryAbility` | uni-app 提供的基类 Ability，继承自鸿蒙的 UIAbility |
| `initUniModules` | 初始化 uni_modules 模块的入口函数 |
| `BuildProfile` | 鸿蒙项目的构建配置，提供 DEBUG 等常量 |

## 注意事项

- 确保 `@dcloudio/uni-app-runtime` 的版本与 HBuilderX 版本匹配

## 相关文档

- [集成编译产物到项目内](./integration-guide.md) — 将 uni-app 编译产物集成到鸿蒙项目
- [模块配置总览](./index.md) — 各模块配置说明

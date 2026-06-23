# 集成编译产物到项目内

> **适用版本**：HBuilderX 5.0+ / uni-app 5.0+
> **平台**：HarmonyOS
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/importfeproject/harmony.html

---

本文档介绍如何将 uni-app 编译产物集成到鸿蒙原生项目内。

## 约定说明

本文档使用 `/resource` 指代鸿蒙资源输出目录。

## 集成步骤

### 步骤一：移动 uni_modules 入口文件

将 uni_modules 入口文件移动到 `/entry/src/main/ets/uni_modules/index.generated.ets`。

如果该目录不存在，需要手动创建：

```
移动 /resource/uni_modules/index.generated.ets
  到 鸿蒙项目 /entry/src/main/ets/uni_modules/index.generated.ets
```

![](https://aka.doubaocdn.com/s/08lb1weOrx)

### 步骤二：部署 UTS API 的 uni_modules

将 UTS API 对应的 uni_modules 文件部署到鸿蒙工程内：

```
移动 /resource/uni_modules 目录下的 UTS API 模块目录
  到 鸿蒙项目 /uni_modules 目录
```

![](https://aka.doubaocdn.com/s/7Ng21weOrx)

#### 关键概念：静态库 moduleName 与 packageName 区分

编译到鸿蒙时，每个 UTS API 的 uni_module 都会创建一个鸿蒙静态库。对于静态库有两个重要概念需要区分：

| 概念 | 说明 | 命名规则 |
|------|------|----------|
| **moduleName** | 静态库的模块名称 | 只允许大小写字母加下划线组成 |
| **packageName** | 静态库被 import 时的名称（类似 npm 包名） | 不允许使用大写字母 |

#### 命名转换示例

以 `uni-getBatteryInfo` 这个 uni_module 为例：

| 属性 | 值 |
|------|-----|
| uni_module 名称 | `uni-getBatteryInfo` |
| **packageName** | `@uni_modules/uni-getbatteryinfo` |
| **moduleName** | `uni_modules__uni_getbatteryinfo` |

**命名规则总结**：
- **packageName**：给 uni_module 名称前加上 `@uni_modules` 前缀，然后转为全小写
- **moduleName**：在 packageName 基础上生成，移除 `@` 符号，将 `/` 替换为两个下划线 `__`，将 `-` 替换为一个下划线 `_`

### 步骤三：修改 oh-package.json5 注册依赖

为所有本地 UTS API 的 uni_module 及其他三方依赖注册 packageName。

uni-app 编译器会自动在 `/resource/uni_modules` 目录下生成 `oh-package.json5` 文件，该文件包含了所有依赖的信息。可以直接将此文件合并到鸿蒙项目的 `oh-package.json5` 文件内。

以 `uni-getBatteryInfo` 为例，在 `oh-package.json5` 文件内 `dependencies` 字段下添加：

```json
{
  "dependencies": {
    "@uni_modules/uni-getbatteryinfo": "./uni_modules/uni-getBatteryInfo"
  }
}
```

### 步骤四：修改 build-profile.json5 注册模块

为所有本地 UTS API 的 uni_module 注册 moduleName。

uni-app 编译器会自动在 `/resource/uni_modules` 目录下生成 `build-profile.json5` 文件，该文件包含了所有模块的信息。可以直接将此文件合并到鸿蒙项目的 `build-profile.json5` 文件内。

以 `uni-getBatteryInfo` 为例，在 `build-profile.json5` 文件内 `modules` 数组内添加：

```json
{
  "modules": [
    {
      "name": "uni_modules__uni_getbatteryinfo",
      "srcPath": "./uni_modules/uni-getBatteryInfo"
    }
  ]
}
```

### 步骤五：拷贝小程序打包资源

将小程序打包出的资源（CLI 项目使用 `npm run build:app-harmony` 生成）拷贝到以下目录：

```
/entry/src/main/resources/resfile/apps/HBuilder
```

**注意**：是 `resfile` 不是 `rawfile`。如果此目录不存在，需手动创建。

![](https://aka.doubaocdn.com/s/VVup1weOrx)

## 完整流程图

```
┌─────────────────────────────────────────────────────────────┐
│                    uni-app 编译产物 (/resource)              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────────┐    ┌────────────────────────────────┐│
│  │ index.generated   │───▶│ /entry/src/main/ets/uni_modules/││
│  │ .ets              │    │   index.generated.ets           ││
│  └──────────────────┘    └────────────────────────────────┘│
│                                                             │
│  ┌──────────────────┐    ┌────────────────────────────────┐│
│  │ uni_modules/      │───▶│ /uni_modules/                   ││
│  │ (UTS API模块)     │    │ (各模块目录)                     ││
│  └──────────────────┘    └────────────────────────────────┘│
│                                                             │
│  ┌──────────────────┐    ┌────────────────────────────────┐│
│  │ oh-package.json5  │───▶│ 合并到根目录 oh-package.json5    ││
│  └──────────────────┘    └────────────────────────────────┘│
│                                                             │
│  ┌──────────────────┐    ┌────────────────────────────────┐│
│  │ build-profile     │───▶│ 合并到根目录 build-profile.json5 ││
│  │ .json5            │    │                                 ││
│  └──────────────────┘    └────────────────────────────────┘│
│                                                             │
│  ┌──────────────────┐    ┌────────────────────────────────┐│
│  │ 打包资源           │───▶│ /entry/src/main/resources/      ││
│  │ (小程序资源)       │    │   resfile/apps/HBuilder         ││
│  └──────────────────┘    └────────────────────────────────┘│
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## 注意事项

- 确保所有路径正确，特别是 `resfile` 和 `rawfile` 的区别
- 合并配置文件时注意 JSON5 格式，避免语法错误

## 相关文档

- [配置鸿蒙原生项目](./setup-guide.md) — 初始配置鸿蒙项目
- [模块配置总览](./index.md) — 各模块配置说明

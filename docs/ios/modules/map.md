# Map（地图）（iOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：iOS (iPhone/iPad)
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/

---

iOS 地图模块支持百度地图、高德地图和苹果原生地图（MapKit）。

## 4.1 百度地图

### 需要引入的系统框架

| 框架 | 说明 |
|------|------|
| CoreLocation.framework | 定位服务 |
| QuartzCore.framework | 图形渲染 |
| OpenGLES.framework | OpenGL ES 支持 |
| SystemConfiguration.framework | 系统配置 |
| Security.framework | 安全服务 |
| libsqlite3.tbd | SQLite 数据库 |
| libstdc++.tbd | C++ 标准库 |
| CoreTelephony.framework | 电话网络信息（定位所需） |

### Info.plist 配置

```xml
<!-- 定位权限 -->
<key>NSLocationWhenInUseUsageDescription</key>
<string>我们需要使用您的位置信息来显示当前位置</string>
<key>NSLocationAlwaysAndWhenInUseUsageDescription</key>
<string>我们需要使用您的位置信息来提供持续导航服务</string>
<key>NSLocationAlwaysUsageDescription</key>
<string>我们需要使用您的位置信息来提供持续导航服务</string>

<!-- 百度地图 API Key -->
<key>BaiduMapApiKey</key>
<string>%您的百度地图API Key%</string>

<!-- 后台定位（可选） -->
<key>UIBackgroundModes</key>
<array>
    <string>location</string>
</array>
```

### CocoaPods 依赖

```ruby
pod 'BaiduMapKit', '~> 7.x.x'  # 百度地图SDK
```

### 需要拷贝的文件

| 路径 | 文件 |
|------|------|
| SDK/libs | `BaiduMapAPI_Base.framework`, `BaiduMapAPI_Map.framework` 等 |

### Objective-C 代码初始化

```objc
#import <BaiduMapAPI_Base/BMKBaseComponent.h>
#import <BaiduMapAPI_Map/BMKMapComponent.h>

@interface AppDelegate () <BMKGeneralDelegate>
@property (nonatomic, strong) BMKMapManager *mapManager;
@end

- (BOOL)application:(UIApplication *)application didFinishLaunchingWithOptions:(NSDictionary *)launchOptions {
    
    // 初始化百度地图
    _mapManager = [[BMKMapManager alloc] init];
    BOOL ret = [_mapManager start:@"您的百度地图API Key" generalDelegate:self];
    if (!ret) {
        NSLog(@"百度地图启动失败");
    }
    
    return YES;
}
```

### dcloud_properties.xml 配置

```xml
<feature name="Maps" value="io.dcloud.js.map.JsMapPluginImpl"></feature>
<service name="Maps" value="io.dcloud.js.map.MapInitImpl" />
```

## 4.2 高德地图

### 需要引入的系统框架

| 框架 | 说明 |
|------|------|
| CoreLocation.framework | 定位服务 |
| QuartzCore.framework | 图形渲染 |
| OpenGLES.framework | OpenGL ES 支持 |
| SystemConfiguration.framework | 系统配置 |
| Security.framework | 安全服务 |
| CoreTelephony.framework | 电话网络信息 |
| libz.tbd | 压缩库 |
| libsqlite3.tbd | SQLite 数据库 |

### Info.plist 配置

```xml
<!-- 定位权限 -->
<key>NSLocationWhenInUseUsageDescription</key>
<string>我们需要使用您的位置信息来显示当前位置</string>
<key>NSLocationAlwaysAndWhenInUseUsageDescription</key>
<string>我们需要使用您的位置信息来提供持续导航服务</string>

<!-- 高德地图 Key -->
<key>AMapApiKey</key>
<string>%您的高德地图Key%</string>
```

### CocoaPods 依赖

```ruby
pod 'AMap3DMap', '~> 10.x.x'   # 3D地图
pod 'AMapSearch', '~> 9.x.x'    # 搜索功能
pod 'AMapLocation', '~> 2.x.x'  # 定位功能（可选）
```

### 需要拷贝的文件

| 路径 | 文件 |
|------|------|
| SDK/libs | `MAMapKit.framework`, `AMapSearchKit.framework` 等 |

### Objective-C 代码初始化

```objc
#import <AMapFoundationKit/AMapFoundationKit.h>

- (BOOL)application:(UIApplication *)application didFinishLaunchingWithOptions:(NSDictionary *)launchOptions {
    
    // 初始化高德地图
    [AMapServices sharedServices].apiKey = @"您的高德地图Key";
    
    return YES;
}
```

### dcloud_properties.xml 配置

```xml
<feature name="Maps" value="io.dcloud.js.map.amap.JsMapPluginImpl"></feature>
```

## 4.3 苹果原生地图（MapKit）

> **无需第三方SDK**，直接使用苹果自带的 MapKit 框架即可。

### 需要引入的系统框架

| 框架 | 说明 |
|------|------|
| MapKit.framework | 苹果地图框架 |
| CoreLocation.framework | 定位服务 |

### Info.plist 配置

```xml
<key>NSLocationWhenInUseUsageDescription</key>
<string>我们需要使用您的位置信息来显示当前位置</string>
```

### Swift 代码示例

```swift
import MapKit
import CoreLocation

class MapViewController: UIViewController, MKMapViewDelegate, CLLocationManagerDelegate {
    
    @IBOutlet weak var mapView: MKMapView!
    let locationManager = CLLocationManager()
    
    override func viewDidLoad() {
        super.viewDidLoad()
        
        mapView.delegate = self
        locationManager.delegate = self
        
        // 请求定位权限
        locationManager.requestWhenInUseAuthorization()
        
        // 显示用户位置
        mapView.showsUserLocation = true
    }
}
```

## ⚠️ iOS 地图注意事项

1. **权限申请**：iOS 需要在运行时动态申请定位权限（NSLocationWhenInUseUsageDescription）
2. **后台定位**：如需持续定位，需开启 location 后台模式并申请 Always 权限
3. **API Key 管理**：百度和高德的 API Key 与 Bundle Identifier 绑定，请确保一致
4. **审核要求**：App Store 审核时需要说明为何需要定位权限

---

## 交叉引用

- 上一篇：[Oauth（登录鉴权）](oauth.md)
- 下一篇：[Speech（语音输入）](speech.md)
- 相关模块：[Geolocation（定位）](geolocation.md)

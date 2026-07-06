import type { IosModuleConfigField, IosModuleConfigModule, IosModuleConfigReport, UniappManifestInfo } from './types'
import { iosMapProviderForModule, normalizeIosMapPageTypeValue } from './moduleFields'
import { cloneJson, ensureObjectPath, normalizeBooleanFieldValue } from './manifestObject'
import {
  cleanupIosPushSdkConfigs,
  clearIosMapPageTypeConfig,
  ensureIosGeolocationSdkConfig,
  ensureIosMapSdkConfig,
  ensureIosPushSdkConfig,
  ensureIosStatisticSdkConfig,
  ensureIosUnipushConfig,
  setIosAllowsArbitraryLoads,
  setIosBluetoothBackgroundModes,
  setIosGeolocationProviderValue,
  setIosMapProviderValue,
  setIosProviderValue
} from './iosManifestConfig'

export function applyIosModuleConfigToManifestInfo(
  info: UniappManifestInfo,
  ctx: {
    selectedNeedsIosConfig: { value: boolean }
    iosModuleConfigReport: { value: IosModuleConfigReport | null }
    isIosConfigModuleSelected: (mod: IosModuleConfigModule) => boolean
    iosFieldValue: (mod: IosModuleConfigModule, field: IosModuleConfigField) => string
    buildIosPrivacyDescriptionPayload: () => Record<string, string>
  }
): UniappManifestInfo {
  if (!ctx.selectedNeedsIosConfig.value) return info
  const manifestValue = cloneJson(info.manifestValue || null)
  if (manifestValue) {
    applyIosGeolocationConfigToManifestValue(manifestValue, ctx)
    applyIosMapConfigToManifestValue(manifestValue, ctx)
    applyIosPushConfigToManifestValue(manifestValue, ctx)
    applyIosBluetoothConfigToManifestValue(manifestValue, ctx)
    applyIosVideoPlayerConfigToManifestValue(manifestValue, ctx)
    applyIosStatisticConfigToManifestValue(manifestValue, ctx)
  }
  return {
    ...info,
    manifestValue,
    iosPrivacyDescriptions: {
      ...(info.iosPrivacyDescriptions || {}),
      ...ctx.buildIosPrivacyDescriptionPayload()
    }
  }
}

function selectedIosModules(ctx: {
  iosModuleConfigReport: { value: IosModuleConfigReport | null }
  isIosConfigModuleSelected: (mod: IosModuleConfigModule) => boolean
}, templateKey: string) {
  return (ctx.iosModuleConfigReport.value?.modules || [])
    .filter(mod => mod.templateKey === templateKey && ctx.isIosConfigModuleSelected(mod))
}

function applyIosGeolocationConfigToManifestValue(manifestValue: Record<string, any>, ctx: any) {
  const geolocationModules = selectedIosModules(ctx, 'geolocation')
  if (!geolocationModules.length) return
  const sdkConfigs = ensureObjectPath(manifestValue, ['app-plus', 'distribute', 'sdkConfigs'])
  const geolocationConfig = ensureIosGeolocationSdkConfig(sdkConfigs)
  for (const mod of geolocationModules) {
    for (const field of mod.fields) {
      if (field.key.startsWith('privacy.')) continue
      const value = ctx.iosFieldValue(mod, field).trim()
      if (!value) continue
      if (field.key === 'baidu.appkey_ios') {
        setIosGeolocationProviderValue(geolocationConfig, 'baidu', ['baidu', 'bd'], 'appkey_ios', value)
      } else if (field.key === 'amap.appkey_ios') {
        setIosGeolocationProviderValue(geolocationConfig, 'amap', ['amap', 'gaode'], 'appkey_ios', value)
      }
    }
  }
}

function applyIosMapConfigToManifestValue(manifestValue: Record<string, any>, ctx: any) {
  const mapModules = selectedIosModules(ctx, 'map')
  if (!mapModules.length) return
  const sdkConfigs = ensureObjectPath(manifestValue, ['app-plus', 'distribute', 'sdkConfigs'])
  const mapConfig = ensureIosMapSdkConfig(sdkConfigs)
  delete mapConfig.__platform__
  for (const mod of mapModules) {
    const provider = iosMapProviderForModule(mod)
    if (provider === 'google') clearIosMapPageTypeConfig(mapConfig)
    for (const field of mod.fields) {
      if (field.key.startsWith('privacy.')) continue
      const value = ctx.iosFieldValue(mod, field).trim()
      if (field.key === 'baidu.appkey_ios') {
        if (value) setIosMapProviderValue(mapConfig, 'baidu', ['baidu', 'bd'], 'appkey_ios', value)
      } else if (field.key === 'amap.appkey_ios') {
        if (value) setIosMapProviderValue(mapConfig, 'amap', ['amap', 'gaode'], 'appkey_ios', value)
      } else if (field.key === 'google.apikey_ios') {
        if (value) setIosMapProviderValue(mapConfig, 'google', ['google', 'googleMap'], 'apikey_ios', value)
      } else if (field.key === 'MAP_PAGE_TYPE') {
        mapConfig.pageType = normalizeIosMapPageTypeValue(provider, value)
      }
    }
  }
}

function applyIosPushConfigToManifestValue(manifestValue: Record<string, any>, ctx: any) {
  const pushModule = selectedIosModules(ctx, 'push')[0]
  if (!pushModule) return
  const sdkConfigs = ensureObjectPath(manifestValue, ['app-plus', 'distribute', 'sdkConfigs'])
  const pushConfig = ensureIosPushSdkConfig(sdkConfigs)
  cleanupIosPushSdkConfigs(sdkConfigs, pushConfig)
  const unipushConfig = ensureIosUnipushConfig(pushConfig)
  for (const field of pushModule.fields) {
    const value = ctx.iosFieldValue(pushModule, field).trim()
    if (!value || field.key === 'pushProvider') continue
    if (field.key === 'unipush.appid') unipushConfig.appid = value
    else if (field.key === 'unipush.appkey') unipushConfig.appkey = value
    else if (field.key === 'unipush.appsecret') unipushConfig.appsecret = value
  }
  if (!('__platform__' in unipushConfig)) unipushConfig.__platform__ = ['ios']
  if (!('version' in unipushConfig)) unipushConfig.version = '2'
}

function applyIosBluetoothConfigToManifestValue(manifestValue: Record<string, any>, ctx: any) {
  const bluetoothModule = selectedIosModules(ctx, 'bluetooth')[0]
  if (!bluetoothModule) return
  const backgroundField = bluetoothModule.fields.find((field: IosModuleConfigField) => field.key === 'backgroundBluetooth')
  if (!backgroundField) return
  const enabled = normalizeBooleanFieldValue(ctx.iosFieldValue(bluetoothModule, backgroundField))
  const iosConfig = ensureObjectPath(manifestValue, ['app-plus', 'distribute', 'ios'])
  setIosBluetoothBackgroundModes(iosConfig, enabled)
}

function applyIosVideoPlayerConfigToManifestValue(manifestValue: Record<string, any>, ctx: any) {
  const videoPlayerModule = selectedIosModules(ctx, 'video_player')[0]
  if (!videoPlayerModule) return
  const atsField = videoPlayerModule.fields.find((field: IosModuleConfigField) => field.key === 'allowArbitraryLoads')
  if (!atsField) return
  const enabled = normalizeBooleanFieldValue(ctx.iosFieldValue(videoPlayerModule, atsField))
  const iosConfig = ensureObjectPath(manifestValue, ['app-plus', 'distribute', 'ios'])
  setIosAllowsArbitraryLoads(iosConfig, enabled)
}

function applyIosStatisticConfigToManifestValue(manifestValue: Record<string, any>, ctx: any) {
  const statisticModules = selectedIosModules(ctx, 'statistic')
  if (!statisticModules.length) return
  const sdkConfigs = ensureObjectPath(manifestValue, ['app-plus', 'distribute', 'sdkConfigs'])
  const statisticConfig = ensureIosStatisticSdkConfig(sdkConfigs)
  for (const mod of statisticModules) {
    for (const field of mod.fields) {
      const value = ctx.iosFieldValue(mod, field).trim()
      if (field.key === 'UMENG_APPKEY') {
        if (value) setIosProviderValue(statisticConfig, 'umeng', ['umeng', 'umeng-ios'], 'appkey_ios', value)
      } else if (field.key === 'UMENG_CHANNEL') {
        if (value) setIosProviderValue(statisticConfig, 'umeng', ['umeng', 'umeng-ios'], 'channelid_ios', value)
      }
    }
  }
}

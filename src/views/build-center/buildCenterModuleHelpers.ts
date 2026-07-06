import type {
  AndroidModuleConfigField,
  AndroidModuleConfigModule,
  IosModuleConfigField,
  IosModuleConfigModule,
  IosModuleConfigReport,
  ModuleStatusTone
} from './types'
import { isIosPrivacyField } from './moduleKeys'

export function createModuleStatusHelpers(ctx: {
  androidFieldValue: (mod: AndroidModuleConfigModule, field: AndroidModuleConfigField) => string
  iosFieldValue: (mod: IosModuleConfigModule, field: IosModuleConfigField) => string
}) {
  function configFieldFilled(mod: AndroidModuleConfigModule, field: AndroidModuleConfigField) {
    return ctx.androidFieldValue(mod, field).trim().length > 0
  }

  function configModuleMissingRequiredCount(mod: AndroidModuleConfigModule) {
    return mod.fields.filter(field => field.required && !configFieldFilled(mod, field)).length
  }

  function configModuleFilledCount(mod: AndroidModuleConfigModule) {
    return mod.fields.filter(field => configFieldFilled(mod, field)).length
  }

  function configModuleStatusTone(mod: AndroidModuleConfigModule): ModuleStatusTone {
    if (!mod.fields.length) return 'success'
    if (configModuleMissingRequiredCount(mod) === 0) return 'success'
    return configModuleFilledCount(mod) > 0 ? 'warning' : 'error'
  }

  function configModuleStatusLabel(mod: AndroidModuleConfigModule) {
    const missing = configModuleMissingRequiredCount(mod)
    if (!mod.fields.length) return '已选'
    if (missing === 0) return '已配置'
    if (configModuleFilledCount(mod) > 0) return '部分配置'
    return '需配置'
  }

  function preferredAndroidConfigModule(modules: AndroidModuleConfigModule[]) {
    return modules.find(mod => configModuleStatusTone(mod) === 'error')
      || modules.find(mod => configModuleStatusTone(mod) === 'warning')
      || modules[0]
  }

  function iosConfigFieldFilled(mod: IosModuleConfigModule, field: IosModuleConfigField) {
    return ctx.iosFieldValue(mod, field).trim().length > 0
  }

  function iosConfigModuleMissingRequiredCount(mod: IosModuleConfigModule) {
    return mod.fields.filter(field => field.required && !iosConfigFieldFilled(mod, field)).length
  }

  function iosConfigModuleFilledCount(mod: IosModuleConfigModule) {
    return mod.fields.filter(field => iosConfigFieldFilled(mod, field)).length
  }

  function iosConfigModuleStatusTone(mod: IosModuleConfigModule): ModuleStatusTone {
    if (!mod.fields.length) return 'success'
    if (iosConfigModuleMissingRequiredCount(mod) === 0) return 'success'
    return iosConfigModuleFilledCount(mod) > 0 ? 'warning' : 'error'
  }

  function iosConfigModuleStatusLabel(mod: IosModuleConfigModule) {
    const missing = iosConfigModuleMissingRequiredCount(mod)
    if (!mod.fields.length) return '已选'
    if (missing === 0) return '已配置'
    if (iosConfigModuleFilledCount(mod) > 0) return '部分配置'
    return '需配置'
  }

  function preferredIosConfigModule(modules: IosModuleConfigModule[]) {
    return modules.find(mod => iosConfigModuleStatusTone(mod) === 'error')
      || modules.find(mod => iosConfigModuleStatusTone(mod) === 'warning')
      || modules[0]
  }

  function androidConfigModuleStatusType(mod: AndroidModuleConfigModule) {
    return configModuleStatusTone(mod)
  }

  function iosConfigModuleStatusType(mod: IosModuleConfigModule) {
    return iosConfigModuleStatusTone(mod)
  }

  function fieldStatusType(mod: AndroidModuleConfigModule, field: AndroidModuleConfigField) {
    const value = ctx.androidFieldValue(mod, field).trim()
    if (!value && field.required) return 'error'
    if (field.valueSource === 'manifest' && value) return 'success'
    if (value) return 'info'
    return 'default'
  }

  function fieldStatusLabel(mod: AndroidModuleConfigModule, field: AndroidModuleConfigField) {
    const value = ctx.androidFieldValue(mod, field).trim()
    if (!value && field.required) return '必填'
    if (!value) return '可选'
    if (field.valueSource === 'manifest') return 'manifest'
    if (field.valueSource === 'default') return '默认'
    return '已填写'
  }

  function iosFieldStatusType(mod: IosModuleConfigModule, field: IosModuleConfigField) {
    const value = ctx.iosFieldValue(mod, field).trim()
    if (!value && field.required) return 'error'
    if (field.valueSource === 'manifest' && value) return 'success'
    if (field.valueSource === 'default' && value) return 'default'
    if (value) return 'info'
    return 'default'
  }

  function iosFieldStatusLabel(mod: IosModuleConfigModule, field: IosModuleConfigField) {
    const value = ctx.iosFieldValue(mod, field).trim()
    if (!value && field.required) return '必填'
    if (!value) return '可选'
    if (field.valueSource === 'manifest') return 'manifest'
    if (field.valueSource === 'default') return '默认'
    return '已填写'
  }

  return {
    androidConfigModuleStatusType,
    configModuleStatusLabel,
    fieldStatusLabel,
    fieldStatusType,
    iosConfigModuleStatusLabel,
    iosConfigModuleStatusType,
    iosFieldStatusLabel,
    iosFieldStatusType,
    preferredAndroidConfigModule,
    preferredIosConfigModule
  }
}

export function isIosLocalPodField(field: IosModuleConfigField) {
  return field.key === 'LOCAL_POD'
}

export function isIosInlineConfigField(field: IosModuleConfigField) {
  return !isIosLocalPodField(field) && !isIosPrivacyField(field)
}

export function stripIosLocalPodFields(report: IosModuleConfigReport): IosModuleConfigReport {
  return {
    modules: report.modules.map(mod => ({
      ...mod,
      fields: mod.fields.filter(field => !isIosLocalPodField(field))
    }))
  }
}

export function iosPrivacyModuleLabel(mod: IosModuleConfigModule) {
  const labels: Record<string, string> = {
    barcode: '扫码',
    bluetooth: '蓝牙',
    camera: '相机',
    contacts: '通讯录',
    face_id: 'Face ID',
    face_recognition: '实人认证',
    fingerprint: '指纹/面容识别',
    geolocation: '定位',
    ibeacon: 'iBeacon',
    livepusher: 'livePusher',
    map: '地图',
    record: '录音',
    speech: '语音识别'
  }
  return labels[mod.templateKey] || mod.name
}

export function iosPrivacyPermissionLabel(plistKey: string, fallback: string) {
  const labels: Record<string, string> = {
    NSBluetoothAlwaysUsageDescription: '蓝牙权限',
    NSBluetoothPeripheralUsageDescription: '蓝牙权限',
    NSCameraUsageDescription: '相机权限',
    NSContactsUsageDescription: '通讯录权限',
    NSFaceIDUsageDescription: 'Face ID 权限',
    NSLocationAlwaysAndWhenInUseUsageDescription: '始终和使用期间定位权限',
    NSLocationAlwaysUsageDescription: '始终定位权限',
    NSLocationWhenInUseUsageDescription: '使用期间定位权限',
    NSMicrophoneUsageDescription: '麦克风权限',
    NSPhotoLibraryAddUsageDescription: '保存到相册权限',
    NSPhotoLibraryUsageDescription: '相册权限',
    NSSpeechRecognitionUsageDescription: '语音识别权限'
  }
  return labels[plistKey] || fallback.replace(/说明$/, '').replace(/权限$/, '权限')
}

import type {
  AndroidModuleConfigField,
  AndroidModuleConfigModule,
  IosModuleConfigField,
  IosModuleConfigModule
} from './types'
import { normalizeManifestConfigKey } from './manifestObject'

function normalizedFieldText(value: string | null | undefined): string {
  return String(value ?? '').trim()
}

export function androidFieldType(field: AndroidModuleConfigField): string {
  return field.fieldType || field.field_type || 'text'
}

export function iosFieldType(field: IosModuleConfigField): string {
  return field.fieldType || field.field_type || 'text'
}

export function isFileField(field: AndroidModuleConfigField): boolean {
  return androidFieldType(field) === 'file'
}

export function isSelectField(field: AndroidModuleConfigField): boolean {
  return androidFieldType(field) === 'select'
}

export function isIosSelectField(field: IosModuleConfigField): boolean {
  return iosFieldType(field) === 'select'
}

export function selectFieldOptions(mod: AndroidModuleConfigModule, field: AndroidModuleConfigField) {
  if (mod.templateKey === 'map' && field.key === 'MAP_PAGE_TYPE') {
    const provider = mapProviderForModule(mod)
    return [
      { label: 'vue', value: 'vue', disabled: provider === 'google' },
      { label: 'nvue', value: 'nvue', disabled: provider === 'baidu' }
    ]
  }
  return []
}

export function mapProviderForModule(mod: AndroidModuleConfigModule) {
  if (mod.fields.some(field => field.key === 'BAIDU_MAP_AK')) return 'baidu'
  if (mod.fields.some(field => field.key === 'AMAP_KEY')) return 'amap'
  if (mod.fields.some(field => field.key === 'GOOGLE_MAPS_API_KEY')) return 'google'
  if (mod.fields.some(field => field.key === 'TENCENT_MAP_KEY')) return 'tencent'
  return 'amap'
}

export function normalizeIosFieldValue(mod: IosModuleConfigModule, field: IosModuleConfigField, value: string | null | undefined): string {
  const rawValue = String(value ?? '')
  const normalized = normalizedFieldText(rawValue)
  if (field.key === 'pushProvider') return normalizeIosPushProviderValue(normalized)
  if (mod.templateKey === 'map' && field.key === 'MAP_PAGE_TYPE') {
    return normalizeIosMapPageTypeValue(iosMapProviderForModule(mod), normalized)
  }
  return rawValue
}

export function normalizeIosPushProviderValue(value: string | null | undefined): string {
  const normalized = normalizedFieldText(value)
  return normalized === 'unipush' ? normalized : 'unipush'
}

export function normalizeIosMapPageTypeValue(provider: string, value: string | null | undefined): string {
  const normalized = normalizeManifestConfigKey(String(value ?? ''))
  if (provider === 'baidu') return 'vue'
  if (provider === 'amap') return 'nvue'
  return normalized === 'nvue' ? 'nvue' : 'vue'
}

export function iosMapProviderForModule(mod: IosModuleConfigModule) {
  if (mod.fields.some(field => field.key === 'baidu.appkey_ios')) return 'baidu'
  if (mod.fields.some(field => field.key === 'amap.appkey_ios')) return 'amap'
  if (mod.fields.some(field => field.key === 'google.apikey_ios')) return 'google'
  return 'amap'
}

export function iosSelectFieldOptions(mod: IosModuleConfigModule | null, field: IosModuleConfigField) {
  if (field.key === 'pushProvider') {
    return [
      { label: 'uniPush', value: 'unipush' },
      { label: '个推推送', value: 'getui', disabled: true },
      { label: 'Google Cloud Message', value: 'gcm', disabled: true }
    ]
  }
  if (
    field.key === 'backgroundBluetooth'
    || field.key === 'allowArbitraryLoads'
    || field.key === 'customComponentMode'
  ) {
    return [
      { label: '否', value: 'false' },
      { label: '是', value: 'true' }
    ]
  }
  if (mod?.templateKey === 'map' && field.key === 'MAP_PAGE_TYPE') {
    const provider = iosMapProviderForModule(mod)
    return [
      { label: 'vue', value: 'vue', disabled: provider === 'amap' },
      { label: 'nvue', value: 'nvue', disabled: provider === 'baidu' }
    ]
  }
  return []
}

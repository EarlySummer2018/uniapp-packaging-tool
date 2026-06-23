import type {
  AndroidModuleConfigField,
  AndroidModuleConfigModule,
  DetectedModule,
  HarmonyModuleConfigField,
  HarmonyModuleConfigModule,
  IosModuleConfigField,
  IosModuleConfigModule,
  NonIosPlatform,
  Platform
} from './types'

export function platformProjectName(platform: Platform) {
  if (platform === 'android') return '安卓'
  if (platform === 'ios') return '苹果'
  return '鸿蒙'
}

export function generateProjectKind(platform: NonIosPlatform) {
  if (platform === 'android') return 'generateAndroidProject' as const
  return 'generateHarmonyProject' as const
}

export function generateProjectCommand(platform: NonIosPlatform) {
  if (platform === 'android') return 'generate_android_project'
  return 'generate_harmony_project'
}

export function formatPlatforms(platforms: string[]) {
  return platforms.filter(platform => platform && platform !== 'all').join(' / ')
}

export function formatModuleWithPlatforms(mod: { name: string; platforms: string[] }) {
  const platforms = formatPlatforms(mod.platforms)
  return platforms ? `${mod.name}(${platforms})` : mod.name
}

export function moduleKeyParts(name: string, category: string, platforms: string[], source: string) {
  const sourceName = source || 'manifest.json'
  return [name, category, platforms.join('|'), sourceName].join('::')
}

export function manifestModuleKey(mod: DetectedModule) {
  return moduleKeyParts(mod.name, mod.category, mod.platforms, mod.source)
}

export function androidConfigModuleKey(mod: AndroidModuleConfigModule) {
  return moduleKeyParts(mod.name, mod.category, mod.platforms, mod.source)
}

export function iosConfigModuleKey(mod: IosModuleConfigModule) {
  return moduleKeyParts(mod.name, mod.category, mod.platforms, mod.source)
}

export function harmonyConfigModuleKey(mod: HarmonyModuleConfigModule) {
  return moduleKeyParts(mod.name, mod.category, mod.platforms, mod.source)
}

export function androidModuleFieldValueKey(mod: AndroidModuleConfigModule, field: AndroidModuleConfigField) {
  return `${mod.templateKey}.${field.key}`
}

export function iosModuleFieldValueKey(mod: IosModuleConfigModule, field: IosModuleConfigField) {
  if (isIosPrivacyField(field)) return field.key
  return `${mod.templateKey}.${field.key}`
}

export function harmonyModuleFieldValueKey(mod: HarmonyModuleConfigModule, field: HarmonyModuleConfigField) {
  return `${mod.templateKey}.${field.key}`
}

export function isIosPrivacyField(field: IosModuleConfigField) {
  return field.key.startsWith('privacy.')
}

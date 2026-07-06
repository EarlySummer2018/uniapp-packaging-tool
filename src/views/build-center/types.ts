export type Platform = 'android' | 'ios' | 'harmony'
export type NonIosPlatform = Exclude<Platform, 'ios'>
export type WorkbenchStatusFilter = 'all' | 'missing' | 'required' | 'configured'

export interface UtsBuiltinModule {
  name: string
  localAar: string
  onlineDeps: string[]
  dependsOn: string[]
  androidDir?: string | null
  iosDir?: string | null
}

export interface UtsCustomPlugin {
  id: string
  androidDir?: string | null
  iosDir?: string | null
  androidDeps: string[]
  iosFrameworks: string[]
  iosSystemFrameworks?: string[]
  iosPlists?: Record<string, string>
  iosProvider?: string | null
  iosDependenciesPods?: Record<string, string>
}

export interface DetectedModule {
  name: string
  category: string
  platforms: string[]
  configured: boolean
  requiredKeys: string[]
  source: string
}

export interface AndroidManifestConfig {
  packageName?: string | null
  minSdkVersion?: number | null
  targetSdkVersion?: number | null
  compileSdkVersion?: number | null
  permissions?: string[]
  excludePermissions?: string[]
  schemes?: string[]
  abiFilters?: string[]
}

export interface PlatformPackages {
  androidPackage?: string | null
  iosBundleId?: string | null
  harmonyBundle?: string | null
}

export interface SplashscreenConfig {
  androidStyle?: string | null
  android: Record<string, string>
  iosStyle?: string | null
  iosStoryboard?: string | null
  useOriginalMsgbox?: boolean | null
}

export interface UniappManifestInfo {
  appName?: string | null
  appId?: string | null
  versionName?: string | null
  versionCode?: number | null
  hbuilderxVersion?: string | null
  androidIcons?: { android: Record<string, string> } | null
  iosIcons?: { ios: Record<string, string> } | null
  iosPrivacyDescriptions?: Record<string, string>
  splashscreen?: SplashscreenConfig | null
  manifestValue?: Record<string, any> | null
  manifestPath: string
  projectRoot: string
  android: AndroidManifestConfig
  packageNames: PlatformPackages
  detectedModules: DetectedModule[]
  warnings: string[]
}

export interface AndroidModuleConfigField {
  key: string
  label: string
  required: boolean
  secret: boolean
  value?: string | null
  valueSource?: string | null
  placeholder: string
  fieldType?: string
  field_type?: string
}

export interface IosModuleConfigField {
  key: string
  label: string
  required: boolean
  secret: boolean
  value?: string | null
  valueSource?: string | null
  placeholder: string
  fieldType?: string
  field_type?: string
}

export interface HarmonyModuleConfigField {
  key: string
  label: string
  required: boolean
  secret: boolean
  value?: string | null
  valueSource?: string | null
  placeholder: string
  fieldType?: string
  field_type?: string
}

export interface AndroidModuleConfigModule {
  name: string
  templateKey: string
  category: string
  platforms: string[]
  source: string
  fields: AndroidModuleConfigField[]
}

export interface AndroidModuleMissingConfig {
  moduleName: string
  key: string
  label: string
}

export interface AndroidModuleConfigReport {
  modules: AndroidModuleConfigModule[]
  missingRequired: AndroidModuleMissingConfig[]
  allConfigured: boolean
}

export interface IosModuleConfigModule {
  name: string
  templateKey: string
  category: string
  platforms: string[]
  source: string
  fields: IosModuleConfigField[]
}

export interface IosModuleConfigReport {
  modules: IosModuleConfigModule[]
}

export interface HarmonyModuleConfigModule {
  name: string
  templateKey: string
  category: string
  platforms: string[]
  source: string
  fields: HarmonyModuleConfigField[]
}

export interface HarmonyModuleConfigReport {
  modules: HarmonyModuleConfigModule[]
}

export interface IosPrivacyDescriptionItem {
  key: string
  fieldKey: string
  label: string
  modules: string[]
  required: boolean
  placeholder: string
  value: string
  missing: boolean
}

export interface ResourceScanResult {
  appId: string
  appName?: string | null
  versionName?: string | null
  versionCode?: number | null
  hbuilderxVersion?: string | null
  sourcePath: string
  importedPath: string
  appResourcePath: string
  isZip: boolean
  manifestPath?: string | null
  splashscreen?: SplashscreenConfig | null
  detectedModules: DetectedModule[]
  uts: {
    hasUtsPlugins: boolean
    hasAndroidUtsPlugins?: boolean
    hasIosUtsPlugins?: boolean
    builtinModules: UtsBuiltinModule[]
    customPlugins: UtsCustomPlugin[]
  }
  warnings: string[]
}

export interface BuildArtifact {
  platform: Platform
  path: string
  fileName: string
  sizeBytes: number
  buildId: string
  cloudRunUrl?: string | null
}

export type BuildExecutionMode = 'auto' | 'local' | 'github'

export interface BuildRecord {
  id: string
  project_id: string
  project_name: string
  platform: Platform
  status: string
  artifact_path?: string | null
  artifact_size_mb?: number | null
  version_name: string
  version_code: number
  build_mode: string
  build_source?: string | null
  cloud_run_url?: string | null
  duration_secs: number
  started_at: string
  finished_at?: string | null
  error_message?: string | null
  log_path?: string | null
  resource_path?: string | null
}

export type ModuleStatusTone = 'default' | 'success' | 'warning' | 'error'

export interface WorkbenchField<TField = unknown> {
  key: string
  label: string
  required: boolean
  secret: boolean
  placeholder: string
  fieldType: string
  raw: TField
}

export interface WorkbenchModule<TModule = unknown, TField = unknown> {
  key: string
  name: string
  category: string
  platforms: string[]
  status: ModuleStatusTone
  statusLabel: string
  missingRequiredCount: number
  filledCount: number
  totalCount: number
  fields: WorkbenchField<TField>[]
  raw: TModule
}

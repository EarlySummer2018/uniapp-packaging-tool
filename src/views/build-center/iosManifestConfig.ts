import {
  cloneJson,
  collectStringValues,
  findFirstObjectEntry,
  isPlainRecord,
  uniqueNonEmptyStrings
} from './manifestObject'

export function clearIosMapPageTypeConfig(mapConfig: Record<string, any>) {
  for (const key of ['pageType', 'page_type', 'MAP_PAGE_TYPE', 'page']) {
    delete mapConfig[key]
  }
}

export function setIosBluetoothBackgroundModes(iosConfig: Record<string, any>, enabled: boolean) {
  const currentModes = collectStringValues(iosConfig.UIBackgroundModes)
    .filter(mode => mode !== 'bluetooth-central' && mode !== 'bluetooth-peripheral')
  if (enabled) {
    currentModes.push('bluetooth-central', 'bluetooth-peripheral')
  }
  iosConfig.UIBackgroundModes = uniqueNonEmptyStrings(currentModes)
}

export function setIosAllowsArbitraryLoads(iosConfig: Record<string, any>, enabled: boolean) {
  const ats = isPlainRecord(iosConfig.NSAppTransportSecurity)
    ? { ...iosConfig.NSAppTransportSecurity }
    : {}
  ats.NSAllowsArbitraryLoads = enabled
  iosConfig.NSAppTransportSecurity = ats
}

export function ensureIosOauthSdkConfig(sdkConfigs: Record<string, any>) {
  if (isPlainRecord(sdkConfigs.oauth)) return sdkConfigs.oauth
  const alias = findFirstObjectEntry(sdkConfigs, ['login', 'oauths'])
  const next = alias ? cloneJson(alias.value) : {}
  sdkConfigs.oauth = next
  return next
}

export function ensureIosShareSdkConfig(sdkConfigs: Record<string, any>) {
  if (isPlainRecord(sdkConfigs.share)) return sdkConfigs.share
  const alias = findFirstObjectEntry(sdkConfigs, ['shares'])
  const next = alias ? cloneJson(alias.value) : {}
  sdkConfigs.share = next
  return next
}

export function ensureIosPaymentSdkConfig(sdkConfigs: Record<string, any>) {
  if (isPlainRecord(sdkConfigs.payment)) return sdkConfigs.payment
  const alias = findFirstObjectEntry(sdkConfigs, ['pay'])
  const next = alias ? cloneJson(alias.value) : {}
  sdkConfigs.payment = next
  return next
}

export function ensureIosSpeechSdkConfig(sdkConfigs: Record<string, any>) {
  if (isPlainRecord(sdkConfigs.speech)) return sdkConfigs.speech
  const alias = findFirstObjectEntry(sdkConfigs, ['speechRecognition'])
  const next = alias ? cloneJson(alias.value) : {}
  sdkConfigs.speech = next
  return next
}

export function ensureIosStatisticSdkConfig(sdkConfigs: Record<string, any>) {
  if (isPlainRecord(sdkConfigs.statistic)) return sdkConfigs.statistic
  const alias = findFirstObjectEntry(sdkConfigs, ['statistics', 'statics'])
  const next = alias ? cloneJson(alias.value) : {}
  sdkConfigs.statistic = next
  return next
}

export function ensureIosGeolocationSdkConfig(sdkConfigs: Record<string, any>) {
  if (isPlainRecord(sdkConfigs.geolocation)) return sdkConfigs.geolocation
  const alias = findFirstObjectEntry(sdkConfigs, ['location', 'position'])
  const next = alias ? cloneJson(alias.value) : {}
  sdkConfigs.geolocation = next
  return next
}

export function ensureIosMapSdkConfig(sdkConfigs: Record<string, any>) {
  if (isPlainRecord(sdkConfigs.maps)) return sdkConfigs.maps
  const alias = findFirstObjectEntry(sdkConfigs, ['map'])
  const next = alias ? cloneJson(alias.value) : {}
  sdkConfigs.maps = next
  return next
}

export function ensureIosPushSdkConfig(sdkConfigs: Record<string, any>) {
  if (isPlainRecord(sdkConfigs.push)) return sdkConfigs.push
  const next = {}
  sdkConfigs.push = next
  return next
}

export function cleanupIosPushSdkConfigs(sdkConfigs: Record<string, any>, pushConfig: Record<string, any>) {
  const unipush = isPlainRecord(pushConfig.unipush) ? cloneJson(pushConfig.unipush) : {}
  for (const key of Object.keys(pushConfig)) {
    delete pushConfig[key]
  }
  pushConfig.unipush = unipush
  for (const key of ['unipush', 'getui', 'igetui', 'gcm', 'fcm', 'google', 'googleCloudMessage']) {
    delete sdkConfigs[key]
  }
}

export function ensureIosUnipushConfig(pushConfig: Record<string, any>) {
  if (isPlainRecord(pushConfig.unipush)) return pushConfig.unipush
  const alias = findFirstObjectEntry(pushConfig, ['unipush'])
  const next = alias ? cloneJson(alias.value) : {}
  pushConfig.unipush = next
  return next
}

export function setIosGeolocationProviderValue(
  geolocationConfig: Record<string, any>,
  canonicalProvider: string,
  aliases: string[],
  key: string,
  value: string
) {
  let provider = isPlainRecord(geolocationConfig[canonicalProvider])
    ? geolocationConfig[canonicalProvider]
    : null
  if (!provider) {
    const alias = findFirstObjectEntry(geolocationConfig, aliases)
    provider = alias ? cloneJson(alias.value) : {}
    geolocationConfig[canonicalProvider] = provider
  }
  provider[key] = value
  delete provider.__platform__
}

export function setIosMapProviderValue(
  mapConfig: Record<string, any>,
  canonicalProvider: string,
  aliases: string[],
  key: string,
  value: string
) {
  let provider = isPlainRecord(mapConfig[canonicalProvider])
    ? mapConfig[canonicalProvider]
    : null
  if (!provider) {
    const alias = findFirstObjectEntry(mapConfig, aliases)
    provider = alias ? cloneJson(alias.value) : {}
    mapConfig[canonicalProvider] = provider
  }
  provider[key] = value
  delete provider.__platform__
}

export function setIosProviderValue(
  config: Record<string, any>,
  canonicalProvider: string,
  aliases: string[],
  key: string,
  value: string
) {
  let provider = isPlainRecord(config[canonicalProvider])
    ? config[canonicalProvider]
    : null
  if (!provider) {
    const alias = findFirstObjectEntry(config, aliases)
    provider = alias ? cloneJson(alias.value) : {}
    config[canonicalProvider] = provider
  }
  provider[key] = value
  delete provider.__platform__
}

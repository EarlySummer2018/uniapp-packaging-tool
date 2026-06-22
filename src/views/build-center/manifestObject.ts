export function ensureObjectPath(root: Record<string, any>, keys: string[]) {
  let current = root
  for (const key of keys) {
    const entry = current[key]
    if (!isPlainRecord(entry)) current[key] = {}
    current = current[key]
  }
  return current
}

export function findFirstObjectEntry(root: Record<string, any>, keys: string[]) {
  for (const key of keys) {
    const direct = root[key]
    if (isPlainRecord(direct)) return { key, value: direct }
    const alias = normalizeManifestConfigKey(key)
    const matchedKey = Object.keys(root).find(candidate => normalizeManifestConfigKey(candidate) === alias)
    if (matchedKey && isPlainRecord(root[matchedKey])) return { key: matchedKey, value: root[matchedKey] }
  }
  return null
}

export function isPlainRecord(value: unknown): value is Record<string, any> {
  return !!value && typeof value === 'object' && !Array.isArray(value)
}

export function normalizeManifestConfigKey(value: string) {
  return value.replace(/[^a-z0-9]/gi, '').toLowerCase()
}

export function collectStringValues(value: unknown): string[] {
  if (Array.isArray(value)) return value.flatMap(item => collectStringValues(item))
  if (typeof value === 'string') {
    return value.split(',').map(item => item.trim()).filter(Boolean)
  }
  return []
}

export function uniqueNonEmptyStrings(values: string[]) {
  const seen = new Set<string>()
  const result: string[] = []
  for (const value of values) {
    const item = value.trim()
    if (!item || seen.has(item)) continue
    seen.add(item)
    result.push(item)
  }
  return result
}

export function normalizeBooleanFieldValue(value: string) {
  return ['1', 'true', 'yes', 'y', 'on', '是', '开启'].includes(value.trim().toLowerCase())
}

export function cloneJson<T>(value: T): T {
  return value == null ? value : JSON.parse(JSON.stringify(value))
}

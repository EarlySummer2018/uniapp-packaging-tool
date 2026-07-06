import { invoke } from '@tauri-apps/api/core'
import type { BuildArtifact, BuildRecord, Platform, UniappManifestInfo } from './types'
import { formatModuleWithPlatforms } from './moduleKeys'

export function createBuildCenterRecords(ctx: any) {
  function manifestLogLines(info: UniappManifestInfo, platform: Platform) {
    const moduleNames = info.detectedModules.map(mod => formatModuleWithPlatforms(mod))
    const lines = [
      `[info] 已读取 manifest.json: ${info.manifestPath}`,
      `[info] manifest 摘要: ${info.appName || '-'} / ${info.appId || '-'} / v${info.versionName || '-'} (${info.versionCode ?? '-'})`
    ]
    if (platform === 'android') {
      lines.push(`[info] Android SDK: min ${info.android.minSdkVersion ?? '-'}, target ${info.android.targetSdkVersion ?? '-'}, compile ${info.android.compileSdkVersion ?? '-'}`)
    }
    lines.push(`[info] manifest 模块 (${moduleNames.length}): ${moduleNames.length ? moduleNames.join(', ') : '无'}`)
    const report = ctx.androidModuleConfigReport.value
    const configurableModules = platform === 'android' ? report?.modules.filter((mod: any) => mod.fields.length > 0) || [] : []
    if (configurableModules.length) {
      let totalFields = 0
      let configuredFields = 0
      const missingRequired: string[] = []
      const missingOptional: string[] = []
      for (const mod of configurableModules) {
        for (const field of mod.fields) {
          totalFields += 1
          const value = ctx.androidFieldValue(mod, field).trim()
          if (value) configuredFields += 1
          else if (field.required) missingRequired.push(`${mod.name} / ${field.label}`)
          else missingOptional.push(`${mod.name} / ${field.label}`)
        }
      }
      lines.push(`[info] Android 模块配置: ${configurableModules.length} 个模块，${configuredFields}/${totalFields} 项已填写`)
      for (const item of missingRequired) lines.push(`[warn] 缺失 Android 必填配置: ${item}`)
      if (missingOptional.length) lines.push(`[info] 未填写可选配置: ${missingOptional.join('、')}`)
    }
    return lines
  }

  async function appendManifestLog(buildId: string, info: UniappManifestInfo, platform: Platform) {
    await ctx.buildStore.appendBuildLogLines(buildId, manifestLogLines(info, platform))
  }

  async function appendFinalLog(buildId: string, status: 'success' | 'failed', errorMessage?: string) {
    await ctx.buildStore.appendBuildLogLines(buildId, [
      status === 'success' ? '[success] 构建完成！' : `[error] 构建失败: ${errorMessage || '未知错误'}`
    ])
  }

  async function createBuildRecord(
    buildId: string,
    platform: Platform,
    startedAt: Date,
    recordProjectId: string = ctx.projectId.value,
    recordProjectName: string = ctx.getProjectName(),
    recordResourcePath: string | null = ctx.scanResult.value?.importedPath || null,
    buildSource: 'local' | 'github' = 'local'
  ) {
    const record: BuildRecord = {
      id: buildId,
      project_id: recordProjectId,
      project_name: recordProjectName,
      platform,
      status: 'building',
      artifact_path: null,
      artifact_size_mb: null,
      version_name: ctx.latestManifestInfo.value?.versionName || ctx.currentProject.value?.app.version || ctx.scanResult.value?.versionName || '-',
      version_code: ctx.latestManifestInfo.value?.versionCode || ctx.currentProject.value?.app.versionCode || ctx.scanResult.value?.versionCode || 1,
      build_mode: 'release',
      build_source: buildSource,
      cloud_run_url: null,
      duration_secs: 0,
      started_at: startedAt.toISOString(),
      finished_at: null,
      error_message: null,
      log_path: null,
      resource_path: recordResourcePath
    }
    try {
      await invoke('add_build_record', { record })
      await ctx.buildStore.appendBuildLogLines(buildId, [`[info] 开始构建 ${platform} 版本...`])
    } catch (e) {
      console.warn('Failed to create build history:', e)
    }
  }

  async function finalizeBuildRecord(
    buildId: string,
    status: 'success' | 'failed',
    startedAt: Date,
    artifact: BuildArtifact | null,
    errorMessage?: string,
    buildSource?: 'local' | 'github'
  ) {
    const finishedAt = new Date()
    try {
      const build = ctx.buildStore.getBuild(buildId)
      await invoke('update_build_record', {
        id: buildId,
        update: {
          status,
          artifact_path: artifact?.path || null,
          artifact_size_mb: artifact ? artifact.sizeBytes / 1024 / 1024 : null,
          finished_at: finishedAt.toISOString(),
          error_message: errorMessage || null,
          log_path: build?.logPath || null,
          build_source: buildSource || null,
          cloud_run_url: artifact?.cloudRunUrl || null,
          duration_secs: Math.max(1, Math.round((finishedAt.getTime() - startedAt.getTime()) / 1000))
        }
      })
    } catch (e) {
      console.warn('Failed to update build history:', e)
    }
  }

  async function cleanupBuildTemporaryFiles(logBuildId: string, cleanupBuildId: string | null, resourcePath: string | null): Promise<string[]> {
    try {
      const cleanupProjectId = ctx.buildStore.getBuild(logBuildId)?.projectId || ctx.projectId.value
      if (!cleanupProjectId) throw new Error('缺少项目 ID，无法清理临时文件')
      const result = await invoke<{ items: Array<{ label: string; path: string; status: string; message: string }> }>(
        'cleanup_build_temporary_files',
        { projectId: cleanupProjectId, buildId: cleanupBuildId, resourcePath }
      )
      const lines = result.items.map((item) => {
        const level = item.status === 'failed' ? 'warn' : 'info'
        return `[${level}] 清理${item.label}: ${item.message} (${item.path})`
      })
      if (lines.length) await appendCleanupLines(logBuildId, lines)
      return lines
    } catch (e: any) {
      const lines = [`[warn] 清理临时文件失败: ${String(e)}`]
      await appendCleanupLines(logBuildId, lines)
      return lines
    }
  }

  async function appendCleanupLines(buildId: string, lines: string[]) {
    if (!lines.length) return
    await ctx.buildStore.appendBuildLogLines(buildId, lines).catch(() => undefined)
  }

  return {
    appendCleanupLines,
    appendFinalLog,
    appendManifestLog,
    cleanupBuildTemporaryFiles,
    createBuildRecord,
    finalizeBuildRecord
  }
}

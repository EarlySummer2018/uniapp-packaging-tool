<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue'
import {
  NCollapse, NCollapseItem, NInput, NSwitch, NCheckbox,
  NRadioGroup, NRadioButton, NFormItem,
  NSpace, NText, NAlert, NDivider, NCard, NButton, NIcon,
  useMessage, NGrid, NGi, NSpin, NEmpty, NTag
} from 'naive-ui'
import {
  CheckmarkCircleOutline, NotificationsOutline, LocationOutline,
  ShareSocialOutline, LogInOutline, MapOutline, CardOutline,
  MicOutline, BarChartOutline, PersonOutline,
  GlobeOutline, VideocamOutline, PhonePortraitOutline,
  CodeWorkingOutline, MegaphoneOutline
} from '@vicons/ionicons5'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

const props = defineProps<{
  projectId: string
}>()

const message = useMessage()
const loading = ref(false)
const projectPath = ref('')
const projectPathLoading = ref(false)

interface VendorRow {
  name: string
  label: string
  enabled: boolean
  fields: { key: string; label: string; value: string }[]
}

const pushEnabled = ref(false)
const unipushAppid = ref('')
const unipushAppkey = ref('')
const unipushAppsecret = ref('')
const pushVendors = reactive<VendorRow[]>([
  { name: 'xiaomi', label: '小米', enabled: false, fields: [
    { key: 'XIAOMI_APP_ID', label: 'AppID', value: '' },
    { key: 'XIAOMI_APP_KEY', label: 'AppKey', value: '' }
  ]},
  { name: 'meizu', label: '魅族', enabled: false, fields: [
    { key: 'MEIZU_APP_ID', label: 'AppID', value: '' },
    { key: 'MEIZU_APP_KEY', label: 'AppKey', value: '' }
  ]},
  { name: 'huawei', label: '华为', enabled: false, fields: [
    { key: 'HUAWEI_APP_ID', label: 'AppID', value: '' }
  ]},
  { name: 'oppo', label: 'OPPO', enabled: false, fields: [
    { key: 'OPPO_APP_KEY', label: 'AppKey', value: '' },
    { key: 'OPPO_APP_SECRET', label: 'AppSecret', value: '' }
  ]},
  { name: 'vivo', label: 'vivo', enabled: false, fields: [
    { key: 'VIVO_APP_ID', label: 'AppID', value: '' },
    { key: 'VIVO_APP_KEY', label: 'AppKey', value: '' }
  ]},
  { name: 'honor', label: '荣耀', enabled: false, fields: [
    { key: 'HONOR_APP_ID', label: 'AppID', value: '' }
  ]}
])

const locationEngine = ref('system')
const baiduAk = ref('')
const amapKey = ref('')

const shareWeixin = reactive({ enabled: false, appid: '', secret: '' })
const shareQq = reactive({ enabled: false, appid: '' })
const shareSina = reactive({ enabled: false, appkey: '', secret: '', redirectUri: '' })

const mapEnabled = ref(false)
const mapEngine = ref('amap')
const mapAmapKey = ref('')
const tencentMapKey = ref('')
const baiduMapAk = ref('')
const googleMapsApiKey = ref('')

const loginEnabled = ref(false)
const loginWeixin = reactive({ enabled: false, appid: '', universalLinks: '' })
const loginQq = reactive({ enabled: false, appid: '', associatedDomains: '' })
const loginApple = reactive({ enabled: false, teamId: '', bundleId: '' })
const loginUniverify = reactive({ enabled: false, apiKey: '', apiSecret: '' })

const paymentEnabled = ref(false)
const paymentWeixin = reactive({ enabled: false, mchId: '', apiKey: '' })
const paymentAlipay = reactive({ enabled: false, appId: '', privateKey: '', publicKey: '' })
const paymentIapApple = reactive({ enabled: false, sharedSecret: '' })

const speechEnabled = ref(false)
const speechEngine = ref('xunfei')
const iflyAppid = ref('')
const bSpeechApiKey = ref('')
const bSpeechSecretKey = ref('')
const aliNlsAccessKeyId = ref('')
const aliNlsAccessKeySecret = ref('')

const statisticEnabled = ref(false)
const statisticProvider = ref('umeng')
const umengAppkey = ref('')
const umengChannel = ref('')
const mtaAppid = ref('')

const faceRecognitionEnabled = ref(false)
const faceProvider = ref('dcloud')
const dcloudLicense = ref('')
const bdFaceApiKey = ref('')
const bdFaceSecretKey = ref('')
const aliFaceAccessKeyId = ref('')
const aliFaceAccessKeySecret = ref('')

const uniAdEnabled = ref(false)
const csjAppId = ref('')
const gdtAppid = ref('')
const gromoreAppId = ref('')
const admobAppId = ref('')

const csjEnabled = computed({ get: () => csjAppId.value !== '', set: (val: boolean) => { if (!val) csjAppId.value = '' } })
const gdtEnabled = computed({ get: () => gdtAppid.value !== '', set: (val: boolean) => { if (!val) gdtAppid.value = '' } })
const gromoreEnabled = computed({ get: () => gromoreAppId.value !== '', set: (val: boolean) => { if (!val) gromoreAppId.value = '' } })
const admobEnabled = computed({ get: () => admobAppId.value !== '', set: (val: boolean) => { if (!val) admobAppId.value = '' } })

const x5Enabled = ref(false)

const livepusherEnabled = ref(false)
const livepusherLicenseUrl = ref('')
const livepusherLicenseKey = ref('')

const uiWebviewEnabled = ref(false)

function getProjectPath(): string | null {
  const trimmed = projectPath.value.trim()
  return trimmed || null
}

async function loadProjectPath() {
  projectPathLoading.value = true
  try {
    const project = await invoke<{ localPath?: string | null }>('get_project', {
      projectId: props.projectId
    })
    projectPath.value = project.localPath || ''
  } catch (e) {
    console.error('Failed to load project path:', e)
  } finally {
    projectPathLoading.value = false
  }
}

async function chooseProjectPath() {
  const selected = await open({
    directory: true,
    multiple: false,
    title: '选择 UniApp 本地项目目录',
  })
  if (typeof selected === 'string') {
    projectPath.value = selected
  }
}

async function saveProjectPath() {
  const trimmed = projectPath.value.trim()
  if (!trimmed) {
    message.warning('请输入本地项目路径')
    return
  }
  projectPathLoading.value = true
  try {
    await invoke('update_project', {
      projectId: props.projectId,
      updates: { localPath: trimmed }
    })
    message.success('本地项目路径已保存')
    await loadModuleConfig()
  } catch (e) {
    message.error(String(e))
  } finally {
    projectPathLoading.value = false
  }
}

async function loadModuleConfig() {
  const projectPath = getProjectPath()
  if (!projectPath) return

  loading.value = true
  try {
    const config = await invoke<any>('parse_project_modules', { projectPath })
    if (config.push) {
      pushEnabled.value = config.push.enabled || false
      if (config.push.unipush_appid) unipushAppid.value = config.push.unipush_appid
      if (config.push.unipush_appkey) unipushAppkey.value = config.push.unipush_appkey
      if (config.push.unipush_appsecret) unipushAppsecret.value = config.push.unipush_appsecret

      if (config.push.vendors && Array.isArray(config.push.vendors)) {
        for (const v of config.push.vendors) {
          const local = pushVendors.find(lv => lv.name === v.name)
          if (local) {
            local.enabled = v.enabled
            for (const f of local.fields) {
              if (v.config[f.key]) f.value = v.config[f.key]
            }
          }
        }
      }
    }
    if (config.geolocation) {
      locationEngine.value = config.geolocation.engine || 'system'
      if (config.geolocation.baidu_ak) baiduAk.value = config.geolocation.baidu_ak
      if (config.geolocation.amap_key) amapKey.value = config.geolocation.amap_key
    }
    if (config.share) {
      if (config.share.weixin) {
        shareWeixin.enabled = true
        if (config.share.weixin.WX_APPID) shareWeixin.appid = config.share.weixin.WX_APPID
        if (config.share.weixin.WX_SECRET) shareWeixin.secret = config.share.weixin.WX_SECRET
      }
      if (config.share.qq) {
        shareQq.enabled = true
        if (config.share.qq.QQ_APPID) shareQq.appid = config.share.qq.QQ_APPID
      }
      if (config.share.sina) {
        shareSina.enabled = true
        if (config.share.sina.SINA_APPKEY) shareSina.appkey = config.share.sina.SINA_APPKEY
        if (config.share.sina.SINA_SECRET) shareSina.secret = config.share.sina.SINA_SECRET
        if (config.share.sina.SINA_REDIRECT_URI) shareSina.redirectUri = config.share.sina.SINA_REDIRECT_URI
      }
    }

    if (config.map) {
      mapEnabled.value = config.map.enabled || false
      mapEngine.value = config.map.engine || 'amap'
      if (config.map.amap_key) mapAmapKey.value = config.map.amap_key
      if (config.map.tencent_map_key) tencentMapKey.value = config.map.tencent_map_key
      if (config.map.baidu_map_ak) baiduMapAk.value = config.map.baidu_map_ak
      if (config.map.google_maps_api_key) googleMapsApiKey.value = config.map.google_maps_api_key
    }

    if (config.login) {
      loginEnabled.value = config.login.enabled || false
      if (config.login.weixin) {
        loginWeixin.enabled = true
        if (config.login.weixin.appid) loginWeixin.appid = config.login.weixin.appid
        if (config.login.weixin.universal_links) loginWeixin.universalLinks = config.login.weixin.universal_links
      }
      if (config.login.qq) {
        loginQq.enabled = true
        if (config.login.qq.appid) loginQq.appid = config.login.qq.appid
        if (config.login.qq.associated_domains) loginQq.associatedDomains = config.login.qq.associated_domains
      }
      if (config.login.apple) {
        loginApple.enabled = true
        if (config.login.apple.team_id) loginApple.teamId = config.login.apple.team_id
        if (config.login.apple.bundle_id) loginApple.bundleId = config.login.apple.bundle_id
      }
      if (config.login.univerify) {
        loginUniverify.enabled = true
        if (config.login.univerify.api_key) loginUniverify.apiKey = config.login.univerify.api_key
        if (config.login.univerify.api_secret) loginUniverify.apiSecret = config.login.univerify.api_secret
      }
    }

    if (config.payment) {
      paymentEnabled.value = config.payment.enabled || false
      if (config.payment.weixin) {
        paymentWeixin.enabled = true
        if (config.payment.weixin.mch_id) paymentWeixin.mchId = config.payment.weixin.mch_id
        if (config.payment.weixin.api_key) paymentWeixin.apiKey = config.payment.weixin.api_key
      }
      if (config.payment.alipay) {
        paymentAlipay.enabled = true
        if (config.payment.alipay.app_id) paymentAlipay.appId = config.payment.alipay.app_id
        if (config.payment.alipay.private_key) paymentAlipay.privateKey = config.payment.alipay.private_key
        if (config.payment.alipay.public_key) paymentAlipay.publicKey = config.payment.alipay.public_key
      }
      if (config.payment.iap_apple) {
        paymentIapApple.enabled = true
        if (config.payment.iap_apple.shared_secret) paymentIapApple.sharedSecret = config.payment.iap_apple.shared_secret
      }
    }

    if (config.speech) {
      speechEnabled.value = config.speech.enabled || false
      speechEngine.value = config.speech.engine || 'xunfei'
      if (config.speech.ifly_appid) iflyAppid.value = config.speech.ifly_appid
      if (config.speech.baidu_api_key) bSpeechApiKey.value = config.speech.baidu_api_key
      if (config.speech.baidu_secret_key) bSpeechSecretKey.value = config.speech.baidu_secret_key
      if (config.speech.ali_nls_access_key_id) aliNlsAccessKeyId.value = config.speech.ali_nls_access_key_id
      if (config.speech.ali_nls_access_key_secret) aliNlsAccessKeySecret.value = config.speech.ali_nls_access_key_secret
    }

    if (config.statistic) {
      statisticEnabled.value = config.statistic.enabled || false
      statisticProvider.value = config.statistic.provider || 'umeng'
      if (config.statistic.umeng_appkey) umengAppkey.value = config.statistic.umeng_appkey
      if (config.statistic.umeng_channel) umengChannel.value = config.statistic.umeng_channel
      if (config.statistic.mta_appid) mtaAppid.value = config.statistic.mta_appid
    }

    if (config.face_recognition) {
      faceRecognitionEnabled.value = config.face_recognition.enabled || false
      faceProvider.value = config.face_recognition.provider || 'dcloud'
      if (config.face_recognition.dcloud_license) dcloudLicense.value = config.face_recognition.dcloud_license
      if (config.face_recognition.bd_api_key) bdFaceApiKey.value = config.face_recognition.bd_api_key
      if (config.face_recognition.bd_secret_key) bdFaceSecretKey.value = config.face_recognition.bd_secret_key
      if (config.face_recognition.ali_access_key_id) aliFaceAccessKeyId.value = config.face_recognition.ali_access_key_id
      if (config.face_recognition.ali_access_key_secret) aliFaceAccessKeySecret.value = config.face_recognition.ali_access_key_secret
    }

    if (config.uni_ad) {
      uniAdEnabled.value = config.uni_ad.enabled || false
      if (config.uni_ad.csj_app_id) csjAppId.value = config.uni_ad.csj_app_id
      if (config.uni_ad.gdt_appid) gdtAppid.value = config.uni_ad.gdt_appid
      if (config.uni_ad.gromore_app_id) gromoreAppId.value = config.uni_ad.gromore_app_id
      if (config.uni_ad.admob_app_id) admobAppId.value = config.uni_ad.admob_app_id
    }

    if (config.x5_tbs) {
      x5Enabled.value = config.x5_tbs.enabled || false
    }

    if (config.livepusher) {
      livepusherEnabled.value = config.livepusher.enabled || false
      if (config.livepusher.license_url) livepusherLicenseUrl.value = config.livepusher.license_url
      if (config.livepusher.license_key) livepusherLicenseKey.value = config.livepusher.license_key
    }

    if (config.ui_webview) {
      uiWebviewEnabled.value = config.ui_webview.enabled || false
    }
  } catch (e) {
    console.error('Failed to load module config:', e)
  } finally {
    loading.value = false
  }
}

async function saveModuleConfig() {
  const projectPath = getProjectPath()
  if (!projectPath) {
    message.warning('请先在项目基本信息中设置项目路径')
    return
  }

  try {
    message.loading('保存中...', { duration: 2000 })
    const config = {
      push: pushEnabled.value ? {
        enabled: true,
        unipush_appid: unipushAppid.value || undefined,
        unipush_appkey: unipushAppkey.value || undefined,
        unipush_appsecret: unipushAppsecret.value || undefined,
        vendors: pushVendors.filter(v => v.enabled).map(v => ({
          name: v.name,
          enabled: true,
          config: Object.fromEntries(v.fields.map(f => [f.key, f.value]))
        }))
      } : undefined,
      geolocation: locationEngine.value !== 'system' ? {
        enabled: true,
        engine: locationEngine.value,
        baidu_ak: baiduAk.value || undefined,
        amap_key: amapKey.value || undefined,
      } : undefined,
      share: (shareWeixin.enabled || shareQq.enabled || shareSina.enabled) ? {
        enabled: true,
        weixin: shareWeixin.enabled ? { WX_APPID: shareWeixin.appid, WX_SECRET: shareWeixin.secret } : undefined,
        qq: shareQq.enabled ? { QQ_APPID: shareQq.appid } : undefined,
        sina: shareSina.enabled ? { SINA_APPKEY: shareSina.appkey, SINA_SECRET: shareSina.secret, SINA_REDIRECT_URI: shareSina.redirectUri } : undefined,
      } : undefined,
      map: mapEnabled.value ? {
        enabled: true,
        engine: mapEngine.value,
        amap_key: mapAmapKey.value || undefined,
        tencent_map_key: tencentMapKey.value || undefined,
        baidu_map_ak: baiduMapAk.value || undefined,
        google_maps_api_key: googleMapsApiKey.value || undefined,
      } : undefined,
      login: loginEnabled.value ? {
        enabled: true,
        weixin: loginWeixin.enabled ? { appid: loginWeixin.appid, universal_links: loginWeixin.universalLinks } : undefined,
        qq: loginQq.enabled ? { appid: loginQq.appid, associated_domains: loginQq.associatedDomains } : undefined,
        apple: loginApple.enabled ? { team_id: loginApple.teamId, bundle_id: loginApple.bundleId } : undefined,
        univerify: loginUniverify.enabled ? { api_key: loginUniverify.apiKey, api_secret: loginUniverify.apiSecret } : undefined,
      } : undefined,
      payment: paymentEnabled.value ? {
        enabled: true,
        weixin: paymentWeixin.enabled ? { mch_id: paymentWeixin.mchId, api_key: paymentWeixin.apiKey } : undefined,
        alipay: paymentAlipay.enabled ? { app_id: paymentAlipay.appId, private_key: paymentAlipay.privateKey, public_key: paymentAlipay.publicKey } : undefined,
        iap_apple: paymentIapApple.enabled ? { shared_secret: paymentIapApple.sharedSecret } : undefined,
      } : undefined,
      speech: speechEnabled.value ? {
        enabled: true,
        engine: speechEngine.value,
        ifly_appid: iflyAppid.value || undefined,
        baidu_api_key: bSpeechApiKey.value || undefined,
        baidu_secret_key: bSpeechSecretKey.value || undefined,
        ali_nls_access_key_id: aliNlsAccessKeyId.value || undefined,
        ali_nls_access_key_secret: aliNlsAccessKeySecret.value || undefined,
      } : undefined,
      statistic: statisticEnabled.value ? {
        enabled: true,
        provider: statisticProvider.value,
        umeng_appkey: umengAppkey.value || undefined,
        umeng_channel: umengChannel.value || undefined,
        mta_appid: mtaAppid.value || undefined,
      } : undefined,
      face_recognition: faceRecognitionEnabled.value ? {
        enabled: true,
        provider: faceProvider.value,
        dcloud_license: dcloudLicense.value || undefined,
        bd_api_key: bdFaceApiKey.value || undefined,
        bd_secret_key: bdFaceSecretKey.value || undefined,
        ali_access_key_id: aliFaceAccessKeyId.value || undefined,
        ali_access_key_secret: aliFaceAccessKeySecret.value || undefined,
      } : undefined,
      uni_ad: uniAdEnabled.value ? {
        enabled: true,
        csj_app_id: csjAppId.value || undefined,
        gdt_appid: gdtAppid.value || undefined,
        gromore_app_id: gromoreAppId.value || undefined,
        admob_app_id: admobAppId.value || undefined,
      } : undefined,
      x5_tbs: x5Enabled.value ? { enabled: true } : undefined,
      livepusher: livepusherEnabled.value ? {
        enabled: true,
        license_url: livepusherLicenseUrl.value || undefined,
        license_key: livepusherLicenseKey.value || undefined,
      } : undefined,
      ui_webview: uiWebviewEnabled.value ? { enabled: true } : undefined,
    }

    await invoke('save_module_config', { projectPath, config })

    message.success('模块配置已保存')
  } catch (e) {
    console.error(e)
    message.error(String(e))
  }
}

onMounted(async () => {
  await loadProjectPath()
  loadModuleConfig()
})

watch(() => props.projectId, async () => {
  await loadProjectPath()
  loadModuleConfig()
})
</script>

<template>
  <div class="module-config-panel">
    <n-spin :show="loading">
      <div v-if="!getProjectPath()" style="padding: 40px; text-align: center;">
        <n-empty description="请先设置 UniApp 本地项目路径">
          <template #extra>
            <n-space vertical :size="12" style="max-width: 720px;">
              <n-text depth="3">
                模块配置需要读取和写入项目目录中的 manifest.json 和 dcloud_properties.xml。
              </n-text>
              <n-space :size="8" style="width: 100%;">
                <n-input
                  v-model:value="projectPath"
                  placeholder="输入或选择 UniApp 本地项目目录"
                  style="flex: 1;"
                  @keyup.enter="saveProjectPath"
                />
                <n-button @click="chooseProjectPath">选择</n-button>
                <n-button type="primary" :loading="projectPathLoading" @click="saveProjectPath">保存路径</n-button>
              </n-space>
            </n-space>
          </template>
        </n-empty>
      </div>

      <n-collapse v-else>
        <!-- Push 推送 -->
        <n-collapse-item title="Push 推送 (uniPush)" name="push">
          <template #header>
            <n-space align="center" :size="8">
              <n-icon :size="18"><NotificationsOutline /></n-icon>
              <n-text strong>Push 推送 (uniPush)</n-text>
              <n-switch v-model:value="pushEnabled" size="small" />
              <n-tag size="small" :type="pushEnabled ? 'success' : 'default'" round>{{ pushEnabled ? '已启用' : '已禁用' }}</n-tag>
            </n-space>
          </template>

          <n-space vertical :size="16">
            <n-alert type="info" title="uniPush 基础配置">
              在 <a href="https://dev.dcloud.net.cn" target="_blank">DCloud 开发者中心</a> 申请 uniPush 应用，获取以下信息。
            </n-alert>

            <n-grid :cols="3" :x-gap="16">
              <n-gi>
                <n-form-item label="AppID">
                  <n-input v-model:value="unipushAppid" placeholder="从 uniPush 控制台获取" />
                </n-form-item>
              </n-gi>
              <n-gi>
                <n-form-item label="AppKey">
                  <n-input v-model:value="unipushAppkey" placeholder="从 uniPush 控制台获取" />
                </n-form-item>
              </n-gi>
              <n-gi>
                <n-form-item label="AppSecret">
                  <n-input v-model:value="unipushAppsecret" placeholder="从 uniPush 控制台获取" show-password-on="click" type="password" />
                </n-form-item>
              </n-gi>
            </n-grid>

            <n-divider>厂商推送通道</n-divider>
            <n-text depth="3" style="font-size: 12px;">启用厂商推送可提升 Android 端到达率。各厂商密钥在对应开放平台申请。</n-text>

            <n-space vertical :size="12">
              <div v-for="vendor in pushVendors" :key="vendor.name" style="display: flex; gap: 12px; align-items: flex-start;">
                <n-checkbox v-model:checked="vendor.enabled" style="padding-top: 6px; min-width: 60px;">
                  {{ vendor.label }}
                </n-checkbox>
                <n-space :size="8">
                  <n-input v-for="field in vendor.fields" :key="field.key"
                    :placeholder="field.label"
                    :value="field.value"
                    @update:value="(v: string) => field.value = v"
                    :disabled="!vendor.enabled"
                    style="width: 180px;" size="small" />
                </n-space>
              </div>
            </n-space>

            <n-button type="primary" @click="saveModuleConfig">
              <template #icon><n-icon><CheckmarkCircleOutline /></n-icon></template>
              保存 Push 配置
            </n-button>
          </n-space>
        </n-collapse-item>

        <!-- Location 定位 -->
        <n-collapse-item title="Location 定位" name="location">
          <template #header>
            <n-space align="center" :size="8">
              <n-icon :size="18"><LocationOutline /></n-icon>
              <n-text strong>Location 定位</n-text>
            </n-space>
          </template>
          <n-space vertical :size="12">
            <n-radio-group v-model:value="locationEngine">
              <n-radio-button value="system">系统定位</n-radio-button>
              <n-radio-button value="baidu">百度地图</n-radio-button>
              <n-radio-button value="amap">高德地图</n-radio-button>
            </n-radio-group>

            <n-input v-if="locationEngine === 'baidu'" v-model:value="baiduAk" placeholder="百度地图 AK (开放平台申请)" />
            <n-input v-if="locationEngine === 'amap'" v-model:value="amapKey" placeholder="高德地图 Key (控制台申请)" />

            <n-button type="primary" @click="saveModuleConfig" size="small">保存定位配置</n-button>
          </n-space>
        </n-collapse-item>

        <!-- Share 分享 -->
        <n-collapse-item title="Share 分享" name="share">
          <template #header>
            <n-space align="center" :size="8">
              <n-icon :size="18"><ShareSocialOutline /></n-icon>
              <n-text strong>Share 分享</n-text>
            </n-space>
          </template>
          <n-space vertical :size="16">
            <div style="display: flex; gap: 24px;">
              <n-card size="small" embedded style="flex: 1;">
                <template #header><n-space :size="4"><n-checkbox v-model:checked="shareWeixin.enabled" /><n-text strong>微信</n-text></n-space></template>
                <n-form-item label="AppID"><n-input v-model:value="shareWeixin.appid" placeholder="微信开放平台 AppID" :disabled="!shareWeixin.enabled" /></n-form-item>
                <n-form-item label="Secret"><n-input v-model:value="shareWeixin.secret" :disabled="!shareWeixin.enabled" /></n-form-item>
              </n-card>
              <n-card size="small" embedded style="flex: 1;">
                <template #header><n-space :size="4"><n-checkbox v-model:checked="shareQq.enabled" /><n-text strong>QQ</n-text></n-space></template>
                <n-form-item label="AppID"><n-input v-model:value="shareQq.appid" placeholder="QQ互联 AppID" :disabled="!shareQq.enabled" /></n-form-item>
              </n-card>
            </div>
            <n-card size="small" embedded>
              <template #header><n-space :size="4"><n-checkbox v-model:checked="shareSina.enabled" /><n-text strong>新浪微博</n-text></n-space></template>
              <n-grid :cols="3" :x-gap="12">
                <n-gi><n-form-item label="AppKey"><n-input v-model:value="shareSina.appkey" :disabled="!shareSina.enabled" /></n-form-item></n-gi>
                <n-gi><n-form-item label="Secret"><n-input v-model:value="shareSina.secret" :disabled="!shareSina.enabled" /></n-form-item></n-gi>
                <n-gi><n-form-item label="RedirectURI"><n-input v-model:value="shareSina.redirectUri" :disabled="!shareSina.enabled" /></n-form-item></n-gi>
              </n-grid>
            </n-card>
            <n-button type="primary" @click="saveModuleConfig" size="small">保存分享配置</n-button>
          </n-space>
        </n-collapse-item>

        <!-- Map 地图 -->
        <n-collapse-item title="Map 地图" name="map">
          <template #header>
            <n-space align="center" :size="8">
              <n-icon :size="18"><MapOutline /></n-icon>
              <n-text strong>Map 地图</n-text>
              <n-switch v-model:value="mapEnabled" size="small" />
              <n-tag size="small" :type="mapEnabled ? 'success' : 'default'" round>{{ mapEnabled ? '已启用' : '已禁用' }}</n-tag>
            </n-space>
          </template>
          <n-space vertical :size="12">
            <n-alert type="info" title="地图引擎选择">
              选择地图服务提供商后填写对应的 API Key 或密钥。
            </n-alert>

            <n-radio-group v-model:value="mapEngine">
              <n-radio-button value="amap">高德地图 (AMap)</n-radio-button>
              <n-radio-button value="tencent">腾讯地图</n-radio-button>
              <n-radio-button value="google">Google Maps</n-radio-button>
              <n-radio-button value="apple">Apple Maps</n-radio-button>
            </n-radio-group>

            <n-grid :cols="2" :x-gap="16">
              <n-gi v-if="mapEngine === 'amap'">
                <n-form-item label="高德 Web端 JSAPI Key">
                  <n-input v-model:value="mapAmapKey" placeholder="从高德开放平台控制台获取" />
                </n-form-item>
              </n-gi>
              <n-gi v-if="mapEngine === 'tencent'">
                <n-form-item label="腾讯地图 Key">
                  <n-input v-model:value="tencentMapKey" placeholder="从腾讯位置服务控制台获取" />
                </n-form-item>
              </n-gi>
              <n-gi v-if="mapEngine === 'google'">
                <n-form-item label="Google Maps API Key">
                  <n-input v-model:value="googleMapsApiKey" placeholder="从 Google Cloud Console 获取" />
                </n-form-item>
              </n-gi>
              <n-gi v-if="mapEngine === 'apple'">
                <n-form-item label="Apple Maps">
                  <n-text depth="3">iOS/macOS 使用系统内置 Apple Maps，无需额外配置 Key。</n-text>
                </n-form-item>
              </n-gi>
            </n-grid>

            <n-button type="primary" @click="saveModuleConfig" size="small">保存地图配置</n-button>
          </n-space>
        </n-collapse-item>

        <!-- Login 登录鉴权 -->
        <n-collapse-item title="Login 登录鉴权" name="login">
          <template #header>
            <n-space align="center" :size="8">
              <n-icon :size="18"><LogInOutline /></n-icon>
              <n-text strong>Login 登录鉴权</n-text>
              <n-switch v-model:value="loginEnabled" size="small" />
              <n-tag size="small" :type="loginEnabled ? 'success' : 'default'" round>{{ loginEnabled ? '已启用' : '已禁用' }}</n-tag>
            </n-space>
          </template>
          <n-space vertical :size="16">
            <n-alert type="info" title="第三方登录配置">
              启用需要的登录方式并填写对应平台的开发者凭证信息。配置将写入 manifest.json 的 AppModules 登录模块中。
            </n-alert>

            <div style="display: flex; gap: 16px; flex-wrap: wrap;">
              <n-card size="small" embedded style="flex: 1; min-width: 240px;">
                <template #header><n-space :size="4"><n-checkbox v-model:checked="loginWeixin.enabled" /><n-text strong>微信登录</n-text></n-space></template>
                <n-form-item label="AppID">
                  <n-input v-model:value="loginWeixin.appid" placeholder="微信开放平台 AppID" :disabled="!loginWeixin.enabled" />
                </n-form-item>
                <n-form-item label="Universal Links">
                  <n-input v-model:value="loginWeixin.universalLinks" placeholder="iOS Universal Links" :disabled="!loginWeixin.enabled" />
                </n-form-item>
              </n-card>

              <n-card size="small" embedded style="flex: 1; min-width: 240px;">
                <template #header><n-space :size="4"><n-checkbox v-model:checked="loginQq.enabled" /><n-text strong>QQ 登录</n-text></n-space></template>
                <n-form-item label="AppID">
                  <n-input v-model:value="loginQq.appid" placeholder="QQ互联 AppID" :disabled="!loginQq.enabled" />
                </n-form-item>
                <n-form-item label="Associated Domains">
                  <n-input v-model:value="loginQq.associatedDomains" placeholder="iOS Associated Domains" :disabled="!loginQq.enabled" />
                </n-form-item>
              </n-card>

              <n-card size="small" embedded style="flex: 1; min-width: 240px;">
                <template #header><n-space :size="4"><n-checkbox v-model:checked="loginApple.enabled" /><n-text strong>Apple 登录</n-text></n-space></template>
                <n-form-item label="Team ID">
                  <n-input v-model:value="loginApple.teamId" placeholder="Apple Developer Team ID" :disabled="!loginApple.enabled" />
                </n-form-item>
                <n-form-item label="Bundle ID">
                  <n-input v-model:value="loginApple.bundleId" placeholder="应用 Bundle Identifier" :disabled="!loginApple.enabled" />
                </n-form-item>
              </n-card>

              <n-card size="small" embedded style="flex: 1; min-width: 240px;">
                <template #header><n-space :size="4"><n-checkbox v-model:checked="loginUniverify.enabled" /><n-text strong>一键登录</n-text></n-space></template>
                <n-form-item label="API Key">
                  <n-input v-model:value="loginUniverify.apiKey" placeholder="运营商一键登录 API Key" :disabled="!loginUniverify.enabled" />
                </n-form-item>
                <n-form-item label="API Secret">
                  <n-input v-model:value="loginUniverify.apiSecret" placeholder="运营商一键登录 API Secret" :disabled="!loginUniverify.enabled" show-password-on="click" type="password" />
                </n-form-item>
              </n-card>
            </div>

            <n-button type="primary" @click="saveModuleConfig" size="small">保存登录配置</n-button>
          </n-space>
        </n-collapse-item>

        <!-- Payment 支付 -->
        <n-collapse-item title="Payment 支付" name="payment">
          <template #header>
            <n-space align="center" :size="8">
              <n-icon :size="18"><CardOutline /></n-icon>
              <n-text strong>Payment 支付</n-text>
              <n-switch v-model:value="paymentEnabled" size="small" />
              <n-tag size="small" :type="paymentEnabled ? 'success' : 'default'" round>{{ paymentEnabled ? '已启用' : '已禁用' }}</n-tag>
            </n-space>
          </template>
          <n-space vertical :size="16">
            <n-alert type="info" title="支付渠道配置">
              启用需要的支付渠道并填写商户/应用凭证。配置将写入 dcloud_properties.xml 和 AndroidManifest.xml。
            </n-alert>

            <div style="display: flex; gap: 16px; flex-wrap: wrap;">
              <n-card size="small" embedded style="flex: 1; min-width: 280px;">
                <template #header><n-space :size="4"><n-checkbox v-model:checked="paymentWeixin.enabled" /><n-text strong>微信支付</n-text></n-space></template>
                <n-form-item label="商户号 (mch_id)">
                  <n-input v-model:value="paymentWeixin.mchId" placeholder="微信支付商户号" :disabled="!paymentWeixin.enabled" />
                </n-form-item>
                <n-form-item label="API 密钥">
                  <n-input v-model:value="paymentWeixin.apiKey" placeholder="微信支付 APIv3 密钥" :disabled="!paymentWeixin.enabled" show-password-on="click" type="password" />
                </n-form-item>
              </n-card>

              <n-card size="small" embedded style="flex: 1; min-width: 280px;">
                <template #header><n-space :size="4"><n-checkbox v-model:checked="paymentAlipay.enabled" /><n-text strong>支付宝</n-text></n-space></template>
                <n-form-item label="AppID">
                  <n-input v-model:value="paymentAlipay.appId" placeholder="支付宝开放平台 AppID" :disabled="!paymentAlipay.enabled" />
                </n-form-item>
                <n-form-item label="应用私钥">
                  <n-input v-model:value="paymentAlipay.privateKey" type="textarea" :rows="3" placeholder="RSA2 应用私钥" :disabled="!paymentAlipay.enabled" />
                </n-form-item>
                <n-form-item label="支付宝公钥">
                  <n-input v-model:value="paymentAlipay.publicKey" type="textarea" :rows="3" placeholder="支付宝公钥" :disabled="!paymentAlipay.enabled" />
                </n-form-item>
              </n-card>

              <n-card size="small" embedded style="flex: 1; min-width: 240px;">
                <template #header><n-space :size="4"><n-checkbox v-model:checked="paymentIapApple.enabled" /><n-text strong>Apple IAP</n-text></n-space></template>
                <n-form-item label="Shared Secret">
                  <n-input v-model:value="paymentIapApple.sharedSecret" placeholder="App Store Connect Shared Secret" :disabled="!paymentIapApple.enabled" show-password-on="click" type="password" />
                </n-form-item>
                <n-text depth="3" style="font-size: 12px;">用于服务端验证 App 内购收据。</n-text>
              </n-card>
            </div>

            <n-button type="primary" @click="saveModuleConfig" size="small">保存支付配置</n-button>
          </n-space>
        </n-collapse-item>

        <!-- Speech 语音输入 -->
        <n-collapse-item title="Speech 语音输入" name="speech">
          <template #header>
            <n-space align="center" :size="8">
              <n-icon :size="18"><MicOutline /></n-icon>
              <n-text strong>Speech 语音输入</n-text>
              <n-switch v-model:value="speechEnabled" size="small" />
              <n-tag size="small" :type="speechEnabled ? 'success' : 'default'" round>{{ speechEnabled ? '已启用' : '已禁用' }}</n-tag>
            </n-space>
          </template>
          <n-space vertical :size="12">
            <n-alert type="info" title="语音识别引擎">
              选择语音识别服务商并填写对应 API 凭证。
            </n-alert>

            <n-radio-group v-model:value="speechEngine">
              <n-radio-button value="xunfei">讯飞语音</n-radio-button>
              <n-radio-button value="baidu">百度语音</n-radio-button>
              <n-radio-button value="ali">阿里云智能语音</n-radio-button>
              <n-radio-button value="system">系统内置</n-radio-button>
            </n-radio-group>

            <n-grid :cols="2" :x-gap="16">
              <n-gi v-if="speechEngine === 'xunfei'">
                <n-form-item label="讯飞 AppID">
                  <n-input v-model:value="iflyAppid" placeholder="讯飞开放平台 AppID" />
                </n-form-item>
              </n-gi>
              <n-gi v-if="speechEngine === 'baidu'">
                <n-form-item label="百度 API Key">
                  <n-input v-model:value="bSpeechApiKey" placeholder="百度 AI 开放平台 API Key" />
                </n-form-item>
                <n-form-item label="百度 Secret Key">
                  <n-input v-model:value="bSpeechSecretKey" placeholder="百度 AI 开放平台 Secret Key" show-password-on="click" type="password" />
                </n-form-item>
              </n-gi>
              <n-gi v-if="speechEngine === 'ali'">
                <n-form-item label="AccessKey ID">
                  <n-input v-model:value="aliNlsAccessKeyId" placeholder="阿里云 NLS AccessKey ID" />
                </n-form-item>
                <n-form-item label="AccessKey Secret">
                  <n-input v-model:value="aliNlsAccessKeySecret" placeholder="阿里云 NLS AccessKey Secret" show-password-on="click" type="password" />
                </n-form-item>
              </n-gi>
              <n-gi v-if="speechEngine === 'system'">
                <n-text depth="3">使用操作系统内置的语音识别能力，无需额外配置。</n-text>
              </n-gi>
            </n-grid>

            <n-button type="primary" @click="saveModuleConfig" size="small">保存语音配置</n-button>
          </n-space>
        </n-collapse-item>

        <!-- Statistic 统计分析 -->
        <n-collapse-item title="Statistic 统计分析" name="statistic">
          <template #header>
            <n-space align="center" :size="8">
              <n-icon :size="18"><BarChartOutline /></n-icon>
              <n-text strong>Statistic 统计分析</n-text>
              <n-switch v-model:value="statisticEnabled" size="small" />
              <n-tag size="small" :type="statisticEnabled ? 'success' : 'default'" round>{{ statisticEnabled ? '已启用' : '已禁用' }}</n-tag>
            </n-space>
          </template>
          <n-space vertical :size="12">
            <n-alert type="info" title="统计分析服务">
              选择数据统计平台并填写对应的应用标识。
            </n-alert>

            <n-radio-group v-model:value="statisticProvider">
              <n-radio-button value="umeng">友盟+ (Umeng)</n-radio-button>
              <n-radio-button value="mta">腾讯 MTA</n-radio-button>
              <n-radio-button value="baidu">百度统计</n-radio-button>
              <n-radio-button value="dcloud">DCloud 统计</n-radio-button>
            </n-radio-group>

            <n-grid :cols="2" :x-gap="16">
              <n-gi v-if="statisticProvider === 'umeng'">
                <n-form-item label="友盟 AppKey">
                  <n-input v-model:value="umengAppkey" placeholder="友盟+ 控制台 AppKey" />
                </n-form-item>
                <n-form-item label="渠道标识">
                  <n-input v-model:value="umengChannel" placeholder="如: 应用商店分发渠道" />
                </n-form-item>
              </n-gi>
              <n-gi v-if="statisticProvider === 'mta'">
                <n-form-item label="MTA AppID">
                  <n-input v-model:value="mtaAppid" placeholder="腾讯 MTA AppID" />
                </n-form-item>
              </n-gi>
              <n-gi v-if="statisticProvider === 'dcloud' || statisticProvider === 'baidu'">
                <n-text depth="3">{{ statisticProvider === 'dcloud' ? '使用 DCloud 内置统计服务，自动采集基础数据。' : '使用百度统计 SDK，需在百度统计后台创建应用。' }}</n-text>
              </n-gi>
            </n-grid>

            <n-button type="primary" @click="saveModuleConfig" size="small">保存统计配置</n-button>
          </n-space>
        </n-collapse-item>

        <!-- FaceRecognition 实人认证 -->
        <n-collapse-item title="实人认证 (FaceRecognition)" name="face_recognition">
          <template #header>
            <n-space align="center" :size="8">
              <n-icon :size="18"><PersonOutline /></n-icon>
              <n-text strong>实人认证 (FaceRecognition)</n-text>
              <n-switch v-model:value="faceRecognitionEnabled" size="small" />
              <n-tag size="small" :type="faceRecognitionEnabled ? 'success' : 'default'" round>{{ faceRecognitionEnabled ? '已启用' : '已禁用' }}</n-tag>
            </n-space>
          </template>
          <n-space vertical :size="12">
            <n-alert type="warning">
              ⚠️ 此模块仅支持 Android 平台，iOS 端不支持实人认证功能。
            </n-alert>

            <n-radio-group v-model:value="faceProvider">
              <n-radio-button value="dcloud">DCloud 实人认证</n-radio-button>
              <n-radio-button value="baidu">百度人脸识别</n-radio-button>
              <n-radio-button value="aliyun">阿里云实人认证</n-radio-button>
            </n-radio-group>

            <n-grid :cols="2" :x-gap="16">
              <n-gi v-if="faceProvider === 'dcloud'">
                <n-form-item label="DCloud License">
                  <n-input v-model:value="dcloudLicense" placeholder="DCloud 实人认证 License" />
                </n-form-item>
              </n-gi>
              <n-gi v-if="faceProvider === 'baidu'">
                <n-form-item label="API Key">
                  <n-input v-model:value="bdFaceApiKey" placeholder="百度 AI 人脸识别 API Key" />
                </n-form-item>
                <n-form-item label="Secret Key">
                  <n-input v-model:value="bdFaceSecretKey" placeholder="百度 AI 人脸识别 Secret Key" show-password-on="click" type="password" />
                </n-form-item>
              </n-gi>
              <n-gi v-if="faceProvider === 'aliyun'">
                <n-form-item label="AccessKey ID">
                  <n-input v-model:value="aliFaceAccessKeyId" placeholder="阿里云实人认证 AccessKey ID" />
                </n-form-item>
                <n-form-item label="AccessKey Secret">
                  <n-input v-model:value="aliFaceAccessKeySecret" placeholder="阿里云实人认证 AccessKey Secret" show-password-on="click" type="password" />
                </n-form-item>
              </n-gi>
            </n-grid>

            <n-button type="primary" @click="saveModuleConfig" size="small">保存实人认证配置</n-button>
          </n-space>
        </n-collapse-item>

        <!-- UniAD 广告 -->
        <n-collapse-item title="uni-AD 广告" name="uni_ad">
          <template #header>
            <n-space align="center" :size="8">
              <n-icon :size="18"><MegaphoneOutline /></n-icon>
              <n-text strong>uni-AD 广告</n-text>
              <n-switch v-model:value="uniAdEnabled" size="small" />
              <n-tag size="small" :type="uniAdEnabled ? 'success' : 'default'" round>{{ uniAdEnabled ? '已启用' : '已禁用' }}</n-tag>
            </n-space>
          </template>
          <n-space vertical :size="16">
            <n-alert type="info" title="广告聚合平台">
              选择需要接入的广告平台并填写各平台的 AppID / 广告位 ID。
            </n-alert>

            <n-grid :cols="2" :x-gap="16" :y-gap="12">
              <n-gi>
                <n-card size="small" embedded>
                  <template #header><n-space :size="4"><n-checkbox v-model:checked="csjEnabled" /><n-text strong>穿山甲 (CSJ)</n-text></n-space></template>
                  <n-form-item label="AppID">
                    <n-input v-model:value="csjAppId" placeholder="字节跳动穿山甲 AppID" />
                  </n-form-item>
                </n-card>
              </n-gi>
              <n-gi>
                <n-card size="small" embedded>
                  <template #header><n-space :size="4"><n-checkbox v-model:checked="gdtEnabled" /><n-text strong>优量汇 (GDT)</n-text></n-space></template>
                  <n-form-item label="AppID">
                    <n-input v-model:value="gdtAppid" placeholder="腾讯广告优量汇 AppID" />
                  </n-form-item>
                </n-card>
              </n-gi>
              <n-gi>
                <n-card size="small" embedded>
                  <template #header><n-space :size="4"><n-checkbox v-model:checked="gromoreEnabled" /><n-text strong>Gromore (华为)</n-text></n-space></template>
                  <n-form-item label="AppID">
                    <n-input v-model:value="gromoreAppId" placeholder="华为 Gromore 广告 AppID" />
                  </n-form-item>
                </n-card>
              </n-gi>
              <n-gi>
                <n-card size="small" embedded>
                  <template #header><n-space :size="4"><n-checkbox v-model:checked="admobEnabled" /><n-text strong>AdMob (Google)</n-text></n-space></template>
                  <n-form-item label="App ID">
                    <n-input v-model:value="admobAppId" placeholder="Google AdMob App ID" />
                  </n-form-item>
                </n-card>
              </n-gi>
            </n-grid>

            <n-button type="primary" @click="saveModuleConfig" size="small">保存广告配置</n-button>
          </n-space>
        </n-collapse-item>

        <!-- X5 TBS WebView -->
        <n-collapse-item title="X5 TBS WebView" name="x5_tbs">
          <template #header>
            <n-space align="center" :size="8">
              <n-icon :size="18"><GlobeOutline /></n-icon>
              <n-text strong>X5 TBS WebView</n-text>
              <n-switch v-model:value="x5Enabled" size="small" />
              <n-tag size="small" :type="x5Enabled ? 'success' : 'default'" round>{{ x5Enabled ? '已启用' : '已禁用' }}</n-tag>
            </n-space>
          </template>
          <n-space vertical :size="12">
            <n-alert type="info">
              腾讯 X5 内核基于 Chromium 定制，提供比系统 WebView 更好的兼容性和性能表现。
              启用后将使用 X5 内核替代 Android 系统 WebView 渲染 web-view 组件和 rich-text 等内容。
            </n-alert>
            <n-space align="center" :size="8">
              <n-tag type="warning" size="small" bordered>仅 Android</n-tag>
              <n-text depth="3">首次加载时需联网下载 X5 内核包（约 30~40MB）。</n-text>
            </n-space>

            <n-button type="primary" @click="saveModuleConfig" size="small">保存 X5 配置</n-button>
          </n-space>
        </n-collapse-item>

        <!-- LivePusher 直播推流 -->
        <n-collapse-item title="LivePusher 直播推流" name="livepusher">
          <template #header>
            <n-space align="center" :size="8">
              <n-icon :size="18"><VideocamOutline /></n-icon>
              <n-text strong>LivePusher 直播推流</n-text>
              <n-switch v-model:value="livepusherEnabled" size="small" />
              <n-tag size="small" :type="livepusherEnabled ? 'success' : 'default'" round>{{ livepusherEnabled ? '已启用' : '已禁用' }}</n-tag>
            </n-space>
          </template>
          <n-space vertical :size="12">
            <n-alert type="info" title="直播推流组件">
              基于 LivePusher 组件实现音视频直播推流能力，支持 RTMP/HLS/HTTP-FLV 协议。
              需要在 <a href="https://dev.dcloud.net.cn" target="_blank">DCloud 开发者中心</a> 申请 License。
            </n-alert>

            <n-grid :cols="2" :x-gap="16">
              <n-gi>
                <n-form-item label="License URL">
                  <n-input v-model:value="livepusherLicenseUrl" placeholder="LivePusher License 文件下载地址" />
                </n-form-item>
              </n-gi>
              <n-gi>
                <n-form-item label="License Key">
                  <n-input v-model:value="livepusherLicenseKey" placeholder="LivePusher License Key" show-password-on="click" type="password" />
                </n-form-item>
              </n-gi>
            </n-grid>

            <n-button type="primary" @click="saveModuleConfig" size="small">保存直播推流配置</n-button>
          </n-space>
        </n-collapse-item>

        <!-- iOS UIWebview -->
        <n-collapse-item title="iOS UIWebview" name="ui_webview">
          <template #header>
            <n-space align="center" :size="8">
              <n-icon :size="18"><PhonePortraitOutline /></n-icon>
              <n-text strong>iOS UIWebview</n-text>
              <n-switch v-model:value="uiWebviewEnabled" size="small" />
              <n-tag size="small" :type="uiWebviewEnabled ? 'success' : 'default'" round>{{ uiWebviewEnabled ? '已启用' : '已禁用' }}</n-tag>
            </n-space>
          </template>
          <n-space vertical :size="12">
            <n-alert type="warning">
              ⚠️ UIWebView 已被 Apple 废弃（deprecated since iOS 12）。建议优先使用 WKWebView。
              仅在需要兼容旧版 iOS 或特定场景下才启用此选项。
            </n-alert>
            <n-space align="center" :size="8">
              <n-tag type="info" size="small" bordered>仅 iOS</n-tag>
              <n-text depth="3">启用后在 iOS 平台使用 UIWebView 替代 WKWebView。</n-text>
            </n-space>

            <n-button type="primary" @click="saveModuleConfig" size="small">保存 UIWebview 配置</n-button>
          </n-space>
        </n-collapse-item>

        <!-- UTS 内置模块 -->
        <n-collapse-item title="UTS 内置模块" name="uts_plugins">
          <template #header>
            <n-space align="center" :size="8">
              <n-icon :size="18"><CodeWorkingOutline /></n-icon>
              <n-text strong>UTS 内置模块</n-text>
            </n-space>
          </template>
          <n-space vertical :size="12">
            <n-alert type="info" title="UTS (Uni Type Script) 插件体系">
              UTS 是 uni-app 的原生插件开发语言，基于 TypeScript 语法，可编译为 Kotlin (Android) / Swift (iOS) 原生代码。
              通过 UTS 插件可以扩展 uni-app 不具备的原生能力。
            </n-alert>

            <n-text depth="3" style="line-height: 1.8;">
              UTS 内置模块包含以下常用原生能力封装：
            </n-text>

            <n-grid :cols="2" :x-gap="12" :y-gap="8">
              <n-gi>
                <n-text>• <n-text strong>蓝牙 (Bluetooth)</n-text> — 低功耗蓝牙通信</n-text>
              </n-gi>
              <n-gi>
                <n-text>• <n-text strong>NFC</n-text> — 近场通信读写</n-text>
              </n-gi>
              <n-gi>
                <n-text>• <n-text strong>指纹/面容 (Biometric)</n-text> — 本地生物识别认证</n-text>
              </n-gi>
              <n-gi>
                <n-text>• <n-text strong>SQLite</n-text> — 本地结构化存储</n-text>
              </n-gi>
              <n-gi>
                <n-text>• <n-text strong>Zip 解压缩</n-text> — 文件压缩与解压</n-text>
              </n-gi>
              <n-gi>
                <n-text>• <n-text strong>相机增强 (CameraPlus)</n-text> — 自定义相机取景</n-text>
              </n-gi>
              <n-gi>
                <n-text>• <n-text strong>录音 (Recorder)</n-text> — 音频录制与格式转换</n-text>
              </n-gi>
              <n-gi>
                <n-text>• <n-text strong>文件管理 (File)</n-text> — 高级文件操作</n-text>
              </n-gi>
            </n-grid>

            <n-divider />

            <n-text depth="3" style="line-height: 1.6;">
              更多 UTS 模块及自定义插件开发文档请访问：
              <a href="https://doc.dcloud.net.cn/uni-app-x/plugin/uts-plugin.html" target="_blank">UTS 插件官方文档</a>
              <br/>
              插件市场：<a href="https://ext.dcloud.net.cn" target="_blank">DCloud 插件市场</a>
            </n-text>
          </n-space>
        </n-collapse-item>
      </n-collapse>
    </n-spin>
  </div>
</template>

<style scoped>
.module-config-panel {
  padding: 0;
}
</style>

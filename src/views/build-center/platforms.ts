import { LogoAndroid, LogoApple, PhonePortraitOutline } from '@vicons/ionicons5'
import type { Component } from 'vue'
import type { Platform } from './types'

export interface PlatformOption {
  key: Platform
  label: string
  icon: Component
  description: string
  color: string
  bgColor: string
}

export const platforms: PlatformOption[] = [
  { key: 'android', label: 'Android', icon: LogoAndroid, description: 'APK 安装包', color: '#2f9e44', bgColor: '#e8f5e9' },
  { key: 'ios', label: 'iOS', icon: LogoApple, description: '离线 SDK / IPA', color: '#1c7ed6', bgColor: '#e7f5ff' },
  { key: 'harmony', label: '鸿蒙', icon: PhonePortraitOutline, description: 'HAP 安装包', color: '#d6336c', bgColor: '#fff0f6' }
]

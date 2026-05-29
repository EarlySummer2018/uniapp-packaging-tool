import { createRouter, createWebHashHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    component: () => import('../views/ProjectList.vue'),
    children: [
      {
        path: '',
        name: 'Home',
        component: () => import('../views/HomeContent.vue'),
        meta: { title: '项目列表' }
      },
      {
        path: 'project/:id',
        name: 'ProjectConfig',
        component: () => import('../views/ProjectConfig.vue'),
        meta: { title: '项目配置' }
      },
      {
        path: 'build/:id',
        name: 'BuildCenter',
        component: () => import('../views/BuildCenter.vue'),
        meta: { title: '构建中心' }
      },
      {
        path: 'sdk-manager',
        name: 'SdkManager',
        component: () => import('../views/SdkManager.vue'),
        meta: { title: 'SDK 管理' }
      },
      {
        path: 'history',
        name: 'BuildHistory',
        component: () => import('../views/BuildHistory.vue'),
        meta: { title: '打包历史' }
      },
      {
        path: 'settings',
        name: 'Settings',
        component: () => import('../views/Settings.vue'),
        meta: { title: '设置' }
      }
    ]
  },
  {
    path: '/:pathMatch(.*)*',
    redirect: '/'
  }
]

const router = createRouter({
  history: createWebHashHistory(),
  routes
})

router.beforeEach((to, _from, next) => {
  if (to.meta.title) {
    document.title = `${to.meta.title} - UniPack Tool`
  }
  next()
})

export default router

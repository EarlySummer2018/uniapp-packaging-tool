import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import App from './App.vue'
import './styles/main.css'
import { useBuildStore } from './stores/build'

const app = createApp(App)

const pinia = createPinia()

app.use(pinia)
app.use(router)

void useBuildStore(pinia).setupGlobalListener()

app.mount('#app')

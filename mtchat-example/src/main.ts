import { createApp } from 'vue'
import { createRouter, createWebHistory } from 'vue-router'
import PrimeVue from 'primevue/config'
import Aura from '@primeuix/themes/aura'
import ToastService from 'primevue/toastservice'
import ConfirmationService from 'primevue/confirmationservice'
import Tooltip from 'primevue/tooltip'

import 'primeicons/primeicons.css'

import App from './App.vue'
import AdminPage from './pages/AdminPage.vue'
import ChatPage from './pages/ChatPage.vue'
import InlinePage from './pages/InlinePage.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', redirect: '/admin' },
    { path: '/admin', component: AdminPage },
    { path: '/chat', component: ChatPage },
    { path: '/inline', component: InlinePage },
    { path: '/inline/:objectType/:objectId', component: InlinePage },
  ],
})

const app = createApp(App)

app.use(router)
app.use(PrimeVue, {
  theme: {
    preset: Aura,
    options: {
      darkModeSelector: '.dark-mode',
    },
  },
})
app.use(ToastService)
app.use(ConfirmationService)
app.directive('tooltip', Tooltip)

app.mount('#app')

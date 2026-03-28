import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import { setInset } from './composables/useInset'

const app = createApp(App)

app.use(router)

app.mount('#app')

setInset()

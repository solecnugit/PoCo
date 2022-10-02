import { createApp } from 'vue'
import './index.css'
import App from './App.vue'
import { createRouter, createWebHashHistory } from "vue-router";
import { routes } from "./route";
import "flowbite";

const router = createRouter({
    history: createWebHashHistory(),
    routes,
})

createApp(App)
    .use(router)
    .mount('#app')

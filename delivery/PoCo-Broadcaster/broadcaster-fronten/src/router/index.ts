// src/router/index.ts
import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import TasksView from '@/views/TasksView.vue'
import TaskDetailView from '@/views/TaskDetailView.vue'
import CreateTaskView from '@/views/CreateTasksView.vue'
// import WorkerQoSView from '@/views/WorkerQoSModel.vue'

const routes = [
  {
    path: '/',
    name: 'home',
    component: HomeView,
  },
  {
    path: '/tasks',
    name: 'tasks',
    component: TasksView, // 懒加载
  },
  {
    path: '/tasks/new',
    name: 'newTask',
    component: CreateTaskView, // 懒加载
  },
  {
    path: '/tasks/:id',
    name: 'taskDetail',
    component: TaskDetailView, // 懒加载
    props: true,
  },
]

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes,
})

export default router

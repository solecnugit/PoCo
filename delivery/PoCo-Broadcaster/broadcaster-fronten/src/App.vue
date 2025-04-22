<template>
  <header>
    <nav class="navbar navbar-expand-lg navbar-dark bg-primary">
      <div class="container">
        <router-link class="navbar-brand" to="/">PoCo 媒体转码系统</router-link>
        <button class="navbar-toggler" type="button" data-bs-toggle="collapse" data-bs-target="#navbarNav">
          <span class="navbar-toggler-icon"></span>
        </button>
        <div class="collapse navbar-collapse" id="navbarNav">
          <ul class="navbar-nav">
            <li class="nav-item">
              <router-link class="nav-link" to="/" active-class="active">首页</router-link>
            </li>
            <li class="nav-item">
              <router-link class="nav-link" to="/tasks" active-class="active">我的任务</router-link>
            </li>
            <li class="nav-item">
              <router-link class="nav-link" to="/tasks/new" active-class="active">创建任务</router-link>
            </li>
          </ul>
          <ul class="navbar-nav ms-auto">
            <li class="nav-item">
              <span class="nav-link">{{ accountId || '未连接钱包' }}</span>
            </li>
          </ul>
        </div>
      </div>
    </nav>
  </header>

  <main class="container mt-4">
    <router-view />
  </main>

  <footer class="footer mt-5 py-3 bg-light">
    <div class="container text-center">
      <span class="text-muted">PoCo 媒体转码系统 &copy; {{ new Date().getFullYear() }}</span>
    </div>
  </footer>

  <alert-component />
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useAppStore } from './stores/app';
import AlertComponent from './components/common/Alert.vue';

const appStore = useAppStore();
const accountId = computed(() => appStore.accountId);
</script>

<style>
#app {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

main {
  flex: 1;
}
</style>
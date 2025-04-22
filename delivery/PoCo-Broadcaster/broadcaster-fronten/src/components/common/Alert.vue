<template>
  <transition name="fade">
    <div 
      v-if="alert"
      class="alert-container"
    >
      <div 
        class="alert" 
        :class="`alert-${alert.type}`" 
        role="alert"
      >
        {{ alert.message }}
        <button 
          type="button" 
          class="btn-close" 
          @click="dismiss"
          aria-label="Close"
        ></button>
      </div>
    </div>
  </transition>
</template>

<script lang="ts">
import { defineComponent, computed } from 'vue';
import { useAppStore } from '@/stores/app';

export default defineComponent({
  name: 'AlertComponent',
  
  setup() {
    const appStore = useAppStore();
    
    const alert = computed(() => appStore.getAlert);
    
    const dismiss = () => {
      appStore.clearAlert();
    };
    
    return {
      alert,
      dismiss
    };
  }
});
</script>

<style scoped>
.alert-container {
  position: fixed;
  top: 20px;
  right: 20px;
  z-index: 1050;
  max-width: 350px;
}

.alert {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-right: 2.5rem;
  box-shadow: 0 0.5rem 1rem rgba(0, 0, 0, 0.15);
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s, transform 0.3s;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateX(30px);
}
</style>
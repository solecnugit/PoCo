<template>
  <div class="file-uploader">
    <div
      class="upload-zone"
      :class="{ 'is-dragging': isDragging }"
      @dragenter.prevent="onDragEnter"
      @dragleave.prevent="onDragLeave"
      @dragover.prevent
      @drop.prevent="onDrop"
    >
      <input
        type="file"
        ref="fileInput"
        class="file-input"
        @change="onFileSelected"
        :accept="accept"
      />
      
      <div v-if="!isUploading" class="upload-content">
        <i class="bi bi-cloud-upload fs-3 mb-2"></i>
        <p class="mb-1">拖放文件到此处或</p>
        <button 
          type="button" 
          class="btn btn-primary" 
          @click="triggerFileInput"
        >
          选择文件
        </button>
        <small v-if="selectedFile" class="d-block mt-2">
          已选择: {{ selectedFile.name }} ({{ formatFileSize(selectedFile.size) }})
        </small>
      </div>
      
      <div v-else class="progress-container">
        <div class="progress mb-2">
          <div 
            class="progress-bar" 
            role="progressbar" 
            :style="{ width: `${uploadProgress}%` }"
            :aria-valuenow="uploadProgress" 
            aria-valuemin="0" 
            aria-valuemax="100"
          >
            {{ uploadProgress }}%
          </div>
        </div>
        <p>{{ uploadStatus }}</p>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, computed } from 'vue';
import { useAppStore } from '@/stores/app';
import IpfsService from '@/services/ipfs-service';

export default defineComponent({
  name: 'FileUploader',
  
  props: {
    accept: {
      type: String,
      default: 'video/mp4,video/webm,video/ogg,video/x-matroska'
    }
  },
  
  emits: ['upload-start', 'upload-progress', 'upload-success', 'upload-error'],
  
  setup(props, { emit }) {
    const appStore = useAppStore();
    const fileInput = ref<HTMLInputElement | null>(null);
    const selectedFile = ref<File | null>(null);
    const isUploading = ref(false);
    const uploadProgress = ref(0);
    const uploadStatus = ref('');
    const isDragging = ref(false);
    
    const config = computed(() => appStore.getConfig);
    
    // 触发文件输入点击
    const triggerFileInput = () => {
      fileInput.value?.click();
    };
    
    // 文件选择处理
    const onFileSelected = (event: Event) => {
      const input = event.target as HTMLInputElement;
      if (input.files && input.files.length > 0) {
        selectedFile.value = input.files[0];
      }
    };
    
    // 拖拽事件处理
    const onDragEnter = () => {
      isDragging.value = true;
    };
    
    const onDragLeave = () => {
      isDragging.value = false;
    };
    
    const onDrop = (event: DragEvent) => {
      isDragging.value = false;
      
      if (event.dataTransfer?.files && event.dataTransfer.files.length > 0) {
        selectedFile.value = event.dataTransfer.files[0];
      }
    };
    
    // 格式化文件大小
    const formatFileSize = (bytes: number): string => {
      if (bytes === 0) return '0 Bytes';
      
      const k = 1024;
      const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
      const i = Math.floor(Math.log(bytes) / Math.log(k));
      
      return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    };
    
    // 上传文件
    const uploadFile = async () => {
      if (!selectedFile.value) {
        emit('upload-error', new Error('未选择文件'));
        return;
      }
      
      try {
        isUploading.value = true;
        uploadProgress.value = 0;
        uploadStatus.value = '准备上传...';
        
        emit('upload-start');
        
        const ipfsService = new IpfsService(config.value);
        
        // 模拟进度更新，实际上传无法获取真实进度
        const progressInterval = setInterval(() => {
          if (uploadProgress.value < 90) {
            uploadProgress.value += Math.random() * 10;
            uploadStatus.value = `上传中... ${Math.round(uploadProgress.value)}%`;
            emit('upload-progress', Math.round(uploadProgress.value));
          }
        }, 500);
        
        // 上传文件到IPFS
        const cid = await ipfsService.uploadFile(selectedFile.value);
        
        // 上传完成
        clearInterval(progressInterval);
        uploadProgress.value = 100;
        uploadStatus.value = '上传完成';
        emit('upload-progress', 100);
        emit('upload-success', cid);
        
        // 清理状态
        setTimeout(() => {
          isUploading.value = false;
          uploadProgress.value = 0;
          uploadStatus.value = '';
        }, 1500);
        
        return cid;
      } catch (error: any) {
        uploadStatus.value = '上传失败';
        emit('upload-error', error);
        throw error;
      } finally {
        isUploading.value = false;
      }
    };
    
    return {
      fileInput,
      selectedFile,
      isUploading,
      uploadProgress,
      uploadStatus,
      isDragging,
      triggerFileInput,
      onFileSelected,
      onDragEnter,
      onDragLeave,
      onDrop,
      formatFileSize,
      uploadFile
    };
  }
});
</script>

<style scoped>
.file-uploader {
  width: 100%;
}

.upload-zone {
  border: 2px dashed #ccc;
  border-radius: 6px;
  padding: 2rem;
  text-align: center;
  position: relative;
  transition: all 0.3s ease;
}

.upload-zone.is-dragging {
  border-color: #007bff;
  background-color: rgba(0, 123, 255, 0.05);
}

.file-input {
  display: none;
}

.upload-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  color: #555;
}

.progress-container {
  width: 100%;
}

.progress {
  height: 0.75rem;
}
</style>
<template>
  <div class="create-task">
    <h2>创建新转码任务</h2>
    <div class="card border-0 shadow-sm mt-4">
      <div class="card-body">
        <form @submit.prevent="submitForm">
          <div class="mb-4">
            <label for="source-file" class="form-label">源文件</label>
            <file-uploader
              @upload-start="onUploadStart"
              @upload-progress="onUploadProgress"
              @upload-success="onUploadSuccess"
              @upload-error="onUploadError"
              ref="fileUploader"
            />
            <!-- 添加进度条 -->
            <div class="progress mt-2" v-if="isUploading">
              <div 
                class="progress-bar" 
                role="progressbar" 
                :style="{width: `${uploadProgress}%`}" 
                :aria-valuenow="uploadProgress" 
                aria-valuemin="0" 
                aria-valuemax="100"
              >
                {{uploadProgress}}%
              </div>
            </div>
            <div class="form-text">支持的格式: MP4, MOV, AVI, MKV等。</div>
          </div>
          
          <div class="row mb-4">
            <div class="col-md-6">
              <label for="target-codec" class="form-label">目标编解码器</label>
              <select 
                class="form-select" 
                id="target-codec" 
                v-model="form.targetCodec" 
                required
              >
                <option value="">请选择...</option>
                <option value="h264">H.264 (AVC)</option>
                <option value="h265">H.265 (HEVC)</option>
                <option value="vp9">VP9</option>
                <option value="av1">AV1</option>
              </select>
            </div>
            <div class="col-md-6">
              <label for="target-resolution" class="form-label">目标分辨率</label>
              <select 
                class="form-select" 
                id="target-resolution" 
                v-model="form.targetResolution" 
                required
              >
                <option value="original">保持原始分辨率</option>
                <option value="1920x1080">1080p (1920x1080)</option>
                <option value="1280x720">720p (1280x720)</option>
                <option value="854x480">480p (854x480)</option>
                <option value="640x360">360p (640x360)</option>
              </select>
            </div>
          </div>
          
          <div class="row mb-4">
            <div class="col-md-6">
              <label for="target-bitrate" class="form-label">目标比特率</label>
              <select 
                class="form-select" 
                id="target-bitrate" 
                v-model="form.targetBitrate" 
                required
              >
                <option value="">请选择...</option>
                <option value="500k">低 (500 kbps)</option>
                <option value="1500k">中 (1500 kbps)</option>
                <option value="3000k">高 (3000 kbps)</option>
                <option value="5000k">超高 (5000 kbps)</option>
              </select>
            </div>
            <div class="col-md-6">
              <label for="target-framerate" class="form-label">目标帧率</label>
              <select 
                class="form-select" 
                id="target-framerate" 
                v-model="form.targetFramerate"
              >
                <option value="original">保持原始帧率</option>
                <option value="30">30 fps</option>
                <option value="25">25 fps</option>
                <option value="24">24 fps</option>
              </select>
            </div>
          </div>
          
          <div class="mb-4">
            <label for="additional-params" class="form-label">额外参数</label>
            <input 
              type="text" 
              class="form-control" 
              id="additional-params" 
              v-model="form.additionalParams"
              placeholder="可选的FFmpeg参数"
            >
            <div class="form-text">高级选项：可添加额外的FFmpeg转码参数。</div>
          </div>
          
          <div class="d-flex">
            <button 
  type="submit" 
  class="btn btn-primary me-2" 
  :disabled="isSubmitting"
>
  <span v-if="isSubmitting">
    <span class="spinner-border spinner-border-sm me-2" role="status"></span>
    创建中...
  </span>
  <span v-else>创建任务</span>
</button>
            <button 
              type="button" 
              class="btn btn-outline-secondary" 
              @click="resetForm"
            >
              重置
            </button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue';
import { useRouter } from 'vue-router';
import { useTasksStore } from '@/stores/tasks';
import { useAppStore } from '../stores/app';
import FileUploader from '../components/ipfs/FileUploader.vue';
import type { TranscodingRequirement } from '../types';

const router = useRouter();
const tasksStore = useTasksStore();
const appStore = useAppStore();

const fileUploader = ref<InstanceType<typeof FileUploader> | null>(null);
const isSubmitting = ref(false);
// 添加上传状态变量
const isUploading = ref(false);
const uploadProgress = ref(0);

// 表单数据
const form = reactive({
  sourceIpfs: '',
  targetCodec: '',
  targetResolution: 'original',
  targetBitrate: '',
  targetFramerate: 'original',
  additionalParams: ''
});

// 上传回调
// const onUploadStart = () => {
//   console.log('文件上传开始');
// };
const onUploadStart = () => {
  console.log('文件上传开始');
  isUploading.value = true;
  uploadProgress.value = 0;
  
  // 显示上传开始提示
  appStore.setAlert({
    type: 'info',
    message: '文件上传开始，请稍候...',
    timeout: 3000
  });
};

const onUploadProgress = (progress: number) => {
  console.log(`上传进度: ${progress}%`);
  // 更新进度条
  uploadProgress.value = Math.round(progress);
};

// const onUploadSuccess = (cid: string) => {
//   console.log(`文件上传成功，CID: ${cid}`);
// };

const onUploadSuccess = (cid: string) => {
  console.log(`文件上传成功，CID: ${cid}`);
  // 设置表单的sourceIpfs值，这将使创建任务按钮可用
  form.sourceIpfs = cid;
  // 重置上传状态
  isUploading.value = false;
  uploadProgress.value = 100;
  
  // 显示成功消息
  appStore.setAlert({
    type: 'success',
    message: '文件上传成功',
    timeout: 3000
  });
};

const onUploadError = (error: Error) => {
  console.error('文件上传失败:', error);
  // 重置上传状态
  isUploading.value = false;
  
  // 显示错误消息
  appStore.setAlert({
    type: 'error',
    message: `文件上传失败: ${error.message}`,
    timeout: 5000
  });
};

// 表单提交
const submitForm = async () => {
  // 检查是否选择了文件
  if (!fileUploader.value || !fileUploader.value.selectedFile) {
    appStore.setAlert({
      type: 'error',
      message: '请先选择源文件',
      timeout: 3000
    });
    return;
  }

  try {
    isSubmitting.value = true;
    
    // 步骤1: 上传文件到IPFS
    console.log('开始上传文件到IPFS...');
    const cid = await fileUploader.value.uploadFile();
    console.log(`文件上传成功，获取到CID: ${cid}`);
    
    // 设置源文件IPFS地址
    form.sourceIpfs = cid;
    
    // 步骤2: 构建转码要求对象
    const requirements: TranscodingRequirement = {
      target_codec: form.targetCodec,
      target_resolution: form.targetResolution,
      target_bitrate: form.targetBitrate,
      target_framerate: form.targetFramerate,
      additional_params: form.additionalParams
    };
    
    console.log('准备创建任务，参数:', {
      sourceIpfs: form.sourceIpfs,
      requirements
    });
    
    // 步骤3: 创建任务
    const taskId = await tasksStore.createTask(form.sourceIpfs, requirements);
    
    console.log('任务创建成功，任务ID:', taskId);
    
    // 显示成功消息
    appStore.setAlert({
      type: 'success',
      message: `任务创建成功，任务ID: ${taskId}`,
      timeout: 3000
    });

     // 添加监听器来捕获任务状态变化事件
     const statusChangeListener = (event: CustomEvent) => {
      const { taskId: changedTaskId, message } = event.detail;
      if (changedTaskId === taskId) {
        // 为特定任务显示状态变化通知
        appStore.setAlert({
          type: 'info',
          message,
          timeout: 5000
        });
        // 监听器只使用一次
        window.removeEventListener('taskStatusChanged', statusChangeListener as EventListener);
      }
    };

    // 添加事件监听
    window.addEventListener('taskStatusChanged', statusChangeListener as EventListener);
    
    // 重置表单
    resetForm();
    
    // 跳转到任务列表
    router.push('/tasks');
    
  } catch (error: any) {
    console.error('创建任务失败:', error);
    appStore.setAlert({
      type: 'error',
      message: `创建任务失败: ${error.message}`,
      timeout: 5000
    });
  } finally {
    isSubmitting.value = false;
  }
};

// 重置表单
const resetForm = () => {
  form.sourceIpfs = '';
  form.targetCodec = '';
  form.targetResolution = 'original';
  form.targetBitrate = '';
  form.targetFramerate = 'original';
  form.additionalParams = '';
  
  // 重置上传状态
  isUploading.value = false;
  uploadProgress.value = 0;
  
  // 重置文件上传组件（如果组件提供该方法）
  if (fileUploader.value && typeof fileUploader.value.reset === 'function') {
    fileUploader.value.reset();
  }
};
</script>

<style scoped>
.create-task {
  max-width: 800px;
  margin: 0 auto;
}
</style>
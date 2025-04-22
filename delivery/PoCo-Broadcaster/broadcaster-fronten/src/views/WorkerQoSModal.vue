  <!-- WorkerQoSModal.vue -->
<template>
    <div class="modal fade" id="workerQoSModal" tabindex="-1" aria-hidden="true">
      <div class="modal-dialog modal-lg">
        <div class="modal-content">
          <div class="modal-header bg-info text-white">
            <h5 class="modal-title">
              <i class="bi bi-bar-chart-fill me-2"></i>工作节点QoS信息
            </h5>
            <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
          </div>
          <div class="modal-body">
            <div v-if="loading" class="text-center py-4">
              <div class="spinner-border text-primary" role="status">
                <span class="visually-hidden">加载中...</span>
              </div>
              <p class="mt-2">加载数据中...</p>
            </div>
            
            <div v-else>
              <!-- QoS评分卡片 -->
              <div class="card mb-4">
                <div class="card-header">
                  <h6 class="mb-0">QoS评分详情</h6>
                </div>
                <div class="card-body">
                  <div class="row">
                    <div class="col-md-6">
                      <div class="mb-3">
                        <h6 class="fw-bold">服务可靠性(SR):</h6>
                        <div class="progress">
                          <div 
                            class="progress-bar bg-primary" 
                            role="progressbar" 
                            :style="`width: ${(qosDetails?.service_reliability_score || 0) * 100}%`"
                            :aria-valuenow="(qosDetails?.service_reliability_score || 0)  * 100" 
                            aria-valuemin="0" 
                            aria-valuemax="100"
                          >
                            {{ Math.round((qosDetails?.service_reliability_score || 0)  * 100) }}%
                          </div>
                        </div>
                      </div>
                      <div class="mb-3">
                        <h6 class="fw-bold">时间稳定性(TS):</h6>
                        <div class="progress">
                          <div 
                            class="progress-bar bg-success" 
                            role="progressbar" 
                            :style="`width: ${(qosDetails?.time_stability_score || 0)  * 100}%`"
                            :aria-valuenow="(qosDetails?.time_stability_score || 0) * 100" 
                            aria-valuemin="0" 
                            aria-valuemax="100"
                          >
                            {{ Math.round((qosDetails?.time_stability_score || 0) * 100) }}%
                          </div>
                        </div>
                      </div>
                    </div>
                    
                    <div class="col-md-6">
                      <div class="mb-3">
                        <h6 class="fw-bold">性能表现(PP):</h6>
                        <div class="progress">
                          <div 
                            class="progress-bar bg-info" 
                            role="progressbar" 
                            :style="`width: ${(qosDetails?.performance_score || 0) * 100}%`"
                            :aria-valuenow="(qosDetails?.performance_score || 0) * 100" 
                            aria-valuemin="0" 
                            aria-valuemax="100"
                          >
                            {{ Math.round((qosDetails?.performance_score || 0) * 100) }}%
                          </div>
                        </div>
                      </div>
                      
                      <div class="mb-3">
                        <h6 class="fw-bold">综合QoS评分:</h6>
                        <div class="progress">
                          <div 
                            class="progress-bar bg-danger" 
                            role="progressbar" 
                            :style="`width: ${(qosDetails?.overall_qos_score || 0) * 100}%`"
                            :aria-valuenow="(qosDetails?.overall_qos_score || 0) * 100" 
                            aria-valuemin="0" 
                            aria-valuemax="100"
                          >
                            {{ Math.round((qosDetails?.overall_qos_score || 0) * 100) }}%
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                  
                  <p class="text-muted small mt-3">
                    最后更新时间: {{ formatDate(qosDetails?.last_update_time) }}
                  </p>
                </div>
              </div>
              
              <!-- 性能摘要卡片 -->
              <div class="card">
                <div class="card-header">
                  <h6 class="mb-0">性能摘要</h6>
                </div>
                <div class="card-body">
                  <div class="row">
                    <div class="col-md-6">
                      <p><strong>已完成任务数:</strong> {{ performanceSummary?.total_tasks_completed || 0 }}</p>
                      <p><strong>平均视频评分:</strong> {{ formatScore(performanceSummary?.avg_video_score) }}</p>
                      <p><strong>平均音频评分:</strong> {{ formatScore(performanceSummary?.avg_audio_score) }}</p>
                      <p><strong>平均同步性能:</strong> {{ formatScore(performanceSummary?.avg_sync_score) }} ms</p>
                    </div>
                    <div class="col-md-6">
                      <p><strong>平均转码时长:</strong> {{ formatDuration(performanceSummary?.avg_encoding_duration) }}</p>
                      <p><strong>任务完成率:</strong> {{ formatPercentage(performanceSummary?.completion_rate) }}</p>
                      <p><strong>规格达标率:</strong> {{ formatPercentage(performanceSummary?.compliance_rate) }}</p>
                      <p><strong>过去7天活跃天数:</strong> {{ performanceSummary?.service_days_last_week || 0 }}</p>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div class="modal-footer">
            <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">关闭</button>
          </div>
        </div>
      </div>
    </div>
  </template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { useQoSStore } from '@/stores/qos';
import { storeToRefs } from 'pinia';

// 获取store
const qosStore = useQoSStore();

// 从store中提取响应式状态
const { 
  qosDetails, 
  performanceSummary, 
  loading: storeLoading, 
  error,
  currentWorkerId 
} = storeToRefs(qosStore);

// 本地loading状态
const loading = ref(false);

// 格式化函数
const formatDate = (timestamp: number | undefined | null) => {
  if (!timestamp) return '未知';
  const date = new Date(timestamp / 1000000); // 假设时间戳是纳秒
  return date.toLocaleString();
};

const formatScore = (score: number | undefined | null) => {
  if (score === undefined || score === null) return 'N/A';
  return score.toFixed(2);
};

const formatPercentage = (value: number | undefined | null) => {
  if (value === undefined || value === null) return 'N/A';
  return `${(value * 100).toFixed(2)}%`;
};

const formatDuration = (duration: number | undefined | null) => {
  if (!duration) return 'N/A';
  return `${(duration / 1000).toFixed(2)}秒`;
};

// 只使用watch来监听currentWorkerId的变化并加载数据
watch(currentWorkerId, async (newWorkerId) => {
  if (newWorkerId) {
    try {
      loading.value = true;
      console.log(`正在加载工作节点 ${newWorkerId} 的QoS数据...`);
      await qosStore.fetchWorkerFullData(newWorkerId);
      console.log('QoS数据加载完成');
    } catch (error) {
      console.error('获取工作节点QoS数据失败:', error);
    } finally {
      loading.value = false;
    }
  }
}, { immediate: true }); // 添加immediate: true使得组件挂载后立即执行一次

// 定义组件名称
defineOptions({
  name: 'WorkerQoSModal'
});

</script>



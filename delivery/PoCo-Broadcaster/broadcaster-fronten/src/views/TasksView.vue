<template>
    <div class="tasks-view">
      <div class="d-flex justify-content-between align-items-center mb-4">
        <h2>我的任务列表</h2>
        <div>
          <button class="btn btn-primary me-2" @click="refreshTasks">
            <i class="bi bi-arrow-clockwise me-1"></i> 刷新任务
          </button>
          <router-link to="/tasks/new" class="btn btn-success">
            <i class="bi bi-plus-lg me-1"></i> 创建新任务
          </router-link>
        </div>
      </div>
  
      <div v-if="isLoading" class="text-center my-5">
        <div class="spinner-border" role="status">
          <span class="visually-hidden">加载中...</span>
        </div>
        <p class="mt-2">加载任务列表...</p>
      </div>
  
      <div v-else-if="error" class="alert alert-danger" role="alert">
        <i class="bi bi-exclamation-triangle-fill me-2"></i>
        {{ error }}
        <button class="btn btn-sm btn-outline-danger ms-3" @click="refreshTasks">重试</button>
      </div>
  
      <div v-else-if="tasks.length === 0" class="text-center my-5">
        <i class="bi bi-inbox display-1 text-muted"></i>
        <p class="mt-3 lead">您还没有创建任何任务</p>
        <router-link to="/tasks/new" class="btn btn-primary mt-2">
          创建第一个任务
        </router-link>
      </div>
  
      <div v-else class="table-responsive">
        <table class="table table-hover">
          <thead class="table-light">
            <tr>
              <th>任务ID</th>
              <th>状态</th>
              <th>源文件</th>
              <th>转码产出</th>
              <th>目标格式</th>
              <th>创建时间</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="task in tasks" :key="task.task_id">
              <td>{{ task.task_id.substring(30, 40) }}</td>
                <td>
                  <span class="badge" :class="getStatusClass(task.status)">
                    {{ getStatusText(task.status) }}
                  </span>
                </td>
                <td> <!-- 添加这个 td 标签 -->
                  <button
                    class="btn btn-sm btn-link text-truncate"
                    @click="openIpfsFile(task.source_ipfs)"
                    :title="task.source_ipfs"
                  >
                    <i class="bi bi-file-earmark"></i>
                    {{ task.source_ipfs.substring(0, 10) }}...
                  </button>
                </td>

            <td>
              <button
                v-if="['Completed', 'Verified'].includes(task.status) && task.result_ipfs"
                class="btn btn-sm btn-link text-truncate"
                @click="openIpfsFile(task.result_ipfs)"
                :title="task.result_ipfs"
              >
                <i class="bi bi-file-earmark-check"></i>
                {{ task.result_ipfs ? task.result_ipfs.substring(0, 10) + '...' : '无' }}
              </button>
              <span v-else class="text-muted">
                <i class="bi bi-hourglass-split"></i> 待完成
              </span>
            </td>
              <td>{{ task.requirements.target_codec }} / {{ task.requirements.target_resolution }}</td>
              <td>{{ formatDate(task.publish_time) }}</td>
              <td>
                <div class="btn-group btn-group-sm">
                  <router-link 
                    :to="`/tasks/${task.task_id}`" 
                    class="btn btn-outline-primary"
                  >
                    查看
                  </router-link>
                  <!-- <button 
                    v-if="task.status === 'Published'"
                    class="btn btn-outline-success"
                    @click="assignTask(task.task_id)"
                  >
                    分配
                  </button> -->
                  <!-- <button 
                    v-if="task.status === 'Completed'"
                    class="btn btn-outline-info"
                    @click="selectVerifiers(task.task_id)"
                  >
                    选择验证者
                  </button> -->
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </template>
  
  <script setup lang="ts">
  import { ref, computed, onMounted, onUnmounted } from 'vue';
  import { useTasksStore } from '../stores/tasks';
  import { useAppStore } from '../stores/app';
  import { TaskStatus } from '../types';

  // 在script setup部分添加
  onMounted(async () => {

    console.log("TaskView初始loading状态:", tasksStore.isLoading);
    await refreshTasks();

    console.log("TaskView刷新后loading状态:", tasksStore.isLoading);
    
    // 添加任务状态变化监听器
    window.addEventListener('taskStatusChanged', handleTaskStatusChanged as EventListener);
  });

    
  

  // 在组件卸载时移除监听器
  onUnmounted(() => {
    window.removeEventListener('taskStatusChanged', handleTaskStatusChanged as EventListener);
  });

  // 添加处理任务状态变化的方法
  const handleTaskStatusChanged = (event: CustomEvent) => {
    const { taskId, oldStatus, newStatus, message } = event.detail;
    
    // 显示通知
    appStore.setAlert({
      type: 'info',
      message,
      timeout: 5000
    });
    
    // 刷新任务列表以显示最新状态
    refreshTasks();
  };
  
  const tasksStore = useTasksStore();
  const appStore = useAppStore();
  
  const tasks = computed(() => tasksStore.allTasks);
  const isLoading = computed(() => tasksStore.isLoading);
  const error = computed(() => tasksStore.getError);
  
  // 刷新任务列表
  const refreshTasks = async () => {
    try {
      console.log("开始刷新任务");
      await tasksStore.fetchTasks();
      console.log("刷新任务完成");
    } catch (error: any) {
      console.error('加载任务失败:', error);
    }finally {
    console.log("refreshTasks finally块, loading状态:", tasksStore.isLoading);
  }
  };
  
  // 获取IPFS URL
  const getIpfsUrl = (cid: string) => {
    return `${appStore.getConfig.ipfsConfig.gateway}/ipfs/${cid}`;
  };

  // 打开IPFS文件
  const openIpfsFile = (ipfsHash: string) => {
    if (!ipfsHash) return;
    
    // 构建IPFS网关URL
    const ipfsUrl = getIpfsUrl(ipfsHash);
    
    // 打开新窗口访问文件
    window.open(ipfsUrl, '_blank');
  };

  
  // 分配任务
  const assignTask = async (taskId: string) => {
    try {
      const workerId = prompt('请输入工作节点 ID:');
      if (workerId) {
        await tasksStore.assignTask(taskId, workerId);
        appStore.setAlert({
          type: 'success',
          message: `任务成功分配给工作节点 ${workerId}`,
          timeout: 3000
        });
      }
    } catch (error: any) {
      appStore.setAlert({
        type: 'error',
        message: `分配任务失败: ${error.message}`,
        timeout: 3000
      });
    }
  };
  
  // 选择验证者
  const selectVerifiers = async (taskId: string) => {
    try {
      const verifiers = await tasksStore.selectVerifiers(taskId);
      appStore.setAlert({
        type: 'success',
        message: `已为任务选择验证者: ${verifiers.join(', ')}`,
        timeout: 3000
      });
    } catch (error: any) {
      appStore.setAlert({
        type: 'error',
        message: `选择验证者失败: ${error.message}`,
        timeout: 3000
      });
    }
  };
  
  // 格式化日期
    const formatDate = (timestamp: number | null) => {
    if (!timestamp) return '未指定';
    
    // 转换纳秒级时间戳为毫秒 (如果需要)
    // NEAR区块链时间戳通常是纳秒级
    let timeMs = timestamp;
    if (timestamp > 1000000000000000) {  // 纳秒级时间戳通常大于这个值
      timeMs = Math.floor(timestamp / 1000000);
    }
    
    return new Date(timeMs).toLocaleString();
  };

  // 获取状态颜色
  const getStatusClass = (status: TaskStatus) => {
    switch (status) {
      case TaskStatus.Published: return 'bg-primary';
      case TaskStatus.Assigned: return 'bg-warning';
      case TaskStatus.Completed: return 'bg-success';
      case TaskStatus.Verified: return 'bg-info';
      default: return 'bg-secondary';
    }
  };
  
  // 获取状态文本
// 获取状态文本
const getStatusText = (status: TaskStatus) => {
  switch (status) {
    // case TaskStatus.Published: return '任务拍卖中';
    case TaskStatus.Queued: return '任务滞留中';
    case TaskStatus.OfferCollecting: return '任务拍卖中';
    case TaskStatus.Assigned: return '已分配任务';
    case TaskStatus.Completed: return '任务已完成';
    case TaskStatus.Verified: return '已验证';
    default: return status;
  }
};
  
  // onMounted(async () => {
  //   await refreshTasks();
  // });
  </script>
  
  <style scoped>
  .tasks-view {
    min-height: 60vh;
  }
  
  .badge {
    font-size: 0.85em;
    padding: 0.5em 0.75em;
  }
  </style>
import { defineStore } from 'pinia'
import ApiService from '@/services/api-service'
import type { TaskData, TranscodingRequirement, QosProof } from '@/types'

// 在store中添加一个Set来跟踪正在监控的任务
const monitoringTasks = new Set<string>()

export const useTasksStore = defineStore('tasks', {
  state: () => ({
    tasks: [] as TaskData[],
    currentTask: null as TaskData | null,
    loading: false,
    error: null as string | null,
  }),

  getters: {
    // 按状态筛选任务
    tasksByStatus: (state) => {
      return (status: string) => state.tasks.filter((task) => task.status === status)
    },

    // 获取所有任务
    allTasks: (state) => state.tasks,

    // 获取当前任务
    getCurrentTask: (state) => state.currentTask,

    // 获取加载状态
    isLoading: (state) => state.loading,

    // 获取错误信息
    getError: (state) => state.error,
  },

  actions: {
    // 加载任务列表
    // 加载任务列表
    async fetchTasks() {
      this.loading = true
      try {
        // 改为获取当前broadcaster的所有任务
        const tasks = await ApiService.getBroadcasterTasks()
        this.tasks = tasks
        return tasks
      } catch (error: any) {
        this.error = error.message || '获取广播者任务列表失败'
        throw error
      } finally {
        this.loading = false
      }
    },

    // 加载单个任务
    async fetchTask(taskId: string) {
      this.loading = true
      try {
        const task = await ApiService.getTask(taskId)
        this.currentTask = task
        return task
      } catch (error: any) {
        this.error = error.message || `获取任务 ${taskId} 失败`
        throw error
      } finally {
        this.loading = false
      }
    },

    // 创建任务
    // 修改createTask方法以自动启动监控
    async createTask(
      sourceIpfs: string,
      requirements: TranscodingRequirement,
      hwAccelerationPreferred: boolean = false,
    ) {
      this.loading = true
      try {
        const taskId = await ApiService.createTask(
          sourceIpfs,
          requirements,
          hwAccelerationPreferred,
        )

        // 创建成功后自动开始监控任务状态
        this.monitorTaskStatus(taskId)

        return taskId
      } catch (error: any) {
        this.error = error.message || '创建任务失败'
        throw error
      } finally {
        this.loading = false
      }
    },

    // 添加到 store 中的 actions 部分

    // 改进的monitorTaskStatus方法
    async monitorTaskStatus(
      taskId: string,
      interval: number = 5000,
      maxAttempts: number = 12, // 默认最多监控1分钟(5秒*12)
    ) {
      // 如果已经在监控这个任务，直接返回
      if (monitoringTasks.has(taskId)) {
        console.log(`任务 ${taskId} 已经在监控中`)
        return
      }

      // 添加到监控集合
      monitoringTasks.add(taskId)
      console.log(`开始监控任务 ${taskId}`)

      let attempts = 0
      let lastStatus = ''
      let backoffFactor = 1 // 用于指数退避

      const checkStatus = async () => {
        if (attempts >= maxAttempts) {
          monitoringTasks.delete(taskId)
          console.log(`监控任务 ${taskId} 已达到最大尝试次数，停止监控`)
          return
        }

        try {
          const task = await ApiService.getTask(taskId)

          if (lastStatus && task.status !== lastStatus) {
            console.log(`任务 ${taskId} 状态从 ${lastStatus} 变为 ${task.status}`)

            // 当任务被分配时显示提示
            if (task.status === 'Assigned' && task.assigned_worker) {
              const workerAddress = task.assigned_worker
              const shortWorker =
                workerAddress.length > 10
                  ? `${workerAddress.substring(0, 6)}...${workerAddress.substring(workerAddress.length - 4)}`
                  : workerAddress

              // 创建自定义事件用于通知
              const event = new CustomEvent('taskStatusChanged', {
                detail: {
                  taskId,
                  oldStatus: lastStatus,
                  newStatus: task.status,
                  message: `任务已分配给工作节点 ${shortWorker}`,
                },
              })
              window.dispatchEvent(event)
            }
          }

          lastStatus = task.status

          // 如果任务已经达到终止状态，停止轮询
          if (
            task.status === 'Assigned' ||
            task.status === 'Completed' ||
            task.status === 'Verified'
          ) {
            monitoringTasks.delete(taskId)
            console.log(`任务 ${taskId} 已达到终止状态 ${task.status}，停止监控`)
            return
          }

          attempts++

          // 使用指数退避增加下次检查的时间间隔
          backoffFactor = Math.min(backoffFactor * 1.5, 5) // 最多增加到5倍
          const nextInterval = interval * backoffFactor

          console.log(
            `将在 ${nextInterval / 1000} 秒后再次检查任务 ${taskId} 状态, 当前状态: ${task.status}`,
          )
          setTimeout(checkStatus, nextInterval)
        } catch (error) {
          console.error(`监控任务状态失败: ${error}`)
          attempts++

          // 即使出错也使用指数退避
          backoffFactor = Math.min(backoffFactor * 1.5, 5)
          setTimeout(checkStatus, interval * backoffFactor)
        }
      }

      // 开始第一次检查
      await checkStatus()
    },
    // 分配任务
    // async assignTask(taskId: string, workerId: string) {
    //   this.loading = true
    //   try {
    //     const success = await ApiService.assignTask(taskId, workerId)
    //     if (success) {
    //       // 重新加载任务
    //       await this.fetchTask(taskId)
    //     }
    //     return success
    //   } catch (error: any) {
    //     this.error = error.message || `分配任务 ${taskId} 失败`
    //     throw error
    //   } finally {
    //     this.loading = false
    //   }
    // },

    // 选择验证者
    async selectVerifiers(taskId: string) {
      this.loading = true
      try {
        const verifiers = await ApiService.selectVerifiers(taskId)
        // 重新加载任务
        await this.fetchTask(taskId)
        return verifiers
      } catch (error: any) {
        this.error = error.message || `为任务 ${taskId} 选择验证者失败`
        throw error
      } finally {
        this.loading = false
      }
    },

    // 获取QoS证明
    async fetchQosProof(taskId: string) {
      this.loading = true
      try {
        const proof = await ApiService.getConsensusProof(taskId)
        return proof
      } catch (error: any) {
        this.error = error.message || `获取任务 ${taskId} 的QoS证明失败`
        throw error
      } finally {
        this.loading = false
      }
    },

    // 清除错误
    clearError() {
      this.error = null
    },
  },
})

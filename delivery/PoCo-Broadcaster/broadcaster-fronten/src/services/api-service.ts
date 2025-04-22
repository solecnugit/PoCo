//broadcaster-frontend/src/services/api-service
import axios from 'axios'
import type { TaskData, TranscodingRequirement, QosProof } from '@/types'
import { TaskStatus } from '@/types'

export class ApiService {
  // 获取账户信息
  static async getAccountInfo() {
    const response = await axios.get('/api/account')
    return response.data
  }

  // 获取IPFS状态
  static async getIpfsStatus() {
    const response = await axios.get('/api/ipfs/status')
    return response.data
  }

  // 获取任务列表
  static async getTasks(fromIndex = 0, limit = 50): Promise<TaskData[]> {
    console.log('获取任务列表')
    const response = await axios.get(`/api/tasks?from_index=${fromIndex}&limit=${limit}`)
    return response.data
  }

  // 获取单个任务
  static async getTask(taskId: string): Promise<TaskData> {
    const response = await axios.get(`/api/tasks/${taskId}`)
    return response.data
  }

  // 更新发布任务方法
  static async createTask(
    sourceIpfs: string,
    requirements: TranscodingRequirement,
    hwAccelerationPreferred: boolean = false,
  ): Promise<string> {
    const response = await axios.post('/api/tasks', {
      source_ipfs: sourceIpfs,
      requirements,
      hw_acceleration_preferred: hwAccelerationPreferred,
    })
    return response.data.taskId
  }

  // 添加任务状态监控方法
  static async monitorTaskStatus(
    taskId: string,
    onStatusChange: (oldStatus: string, newStatus: string, task: TaskData) => void,
    interval: number = 5000,
    maxAttempts: number = 12, // 默认最多监控1分钟(5秒*12)
  ): Promise<void> {
    let attempts = 0
    let lastStatus = ''

    const checkStatus = async () => {
      if (attempts >= maxAttempts) return

      try {
        const task = await this.getTask(taskId)

        if (lastStatus && task.status !== lastStatus) {
          onStatusChange(lastStatus, task.status, task)
        }

        lastStatus = task.status

        // 如果任务已经被分配或完成，停止轮询
        // 如果任务已经被分配或完成，停止轮询
        if (task.status !== TaskStatus.OfferCollecting && task.status !== TaskStatus.Queued) {
          return
        }

        attempts++
        setTimeout(checkStatus, interval)
      } catch (error) {
        console.error(`监控任务状态失败: ${error}`)
        attempts++
        setTimeout(checkStatus, interval)
      }
    }

    // 开始第一次检查
    await checkStatus()
  }

  // 在ApiService中添加
  static async getVerifierProof(taskId: string, verifierId: string) {
    try {
      const response = await fetch(`/api/tasks/${taskId}/verifier-proof/${verifierId}`)
      if (!response.ok) {
        throw new Error(`服务器错误: ${response.status}`)
      }
      return await response.json()
    } catch (error) {
      console.error('获取验证者质量证明失败:', error)
      throw error
    }
  }

  // 分配任务
  // static async assignTask(taskId: string, workerId: string): Promise<boolean> {
  //   const response = await axios.post(`/api/tasks/${taskId}/assign`, {
  //     worker_id: workerId,
  //   })
  //   return response.data.success
  // }

  // 获取当前broadcaster的所有任务
  static async getBroadcasterTasks(): Promise<TaskData[]> {
    console.log('获取broadcaster任务列表')
    const response = await axios.get('/api/broadcaster/tasks')
    return response.data
  }

  // 获取工作节点信息
  static async getWorkerInfo(workerId: string) {
    const response = await axios.get(`/api/workers/${workerId}`)
    return response.data
  }

  // 获取工作节点QoS评分详情
  static async getWorkerQosDetails(workerId: string) {
    const response = await axios.get(`/api/workers/${workerId}/qos`)
    return response.data
  }

  // 获取工作节点性能摘要
  static async getWorkerPerformanceSummary(workerId: string) {
    const response = await axios.get(`/api/workers/${workerId}/performance`)
    return response.data
  }

  // 选择验证者
  static async selectVerifiers(taskId: string): Promise<string[]> {
    const response = await axios.post(`/api/tasks/${taskId}/verifiers`)
    return response.data.verifiers
  }

  // 获取QoS共识证明
  static async getConsensusProof(taskId: string): Promise<QosProof> {
    const response = await axios.get(`/api/consensus/${taskId}`)
    return response.data
  }

  // 上传文件到IPFS
  static async uploadFileToIpfs(file: File): Promise<string> {
    const formData = new FormData()
    formData.append('file', file)

    const response = await axios.post('/api/ipfs/upload', formData, {
      headers: {
        'Content-Type': 'multipart/form-data',
      },
      onUploadProgress: (progressEvent) => {
        // 这里可以处理上传进度
        const percentCompleted = Math.round(
          (progressEvent.loaded * 100) / (progressEvent.total || 1),
        )
        console.log(`上传进度: ${percentCompleted}%`)
      },
    })

    return response.data.Hash
  }
}

export default ApiService

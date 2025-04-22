import { defineStore } from 'pinia'
import ApiService from '@/services/api-service'
import type {
  WorkerInfo,
  WorkerPerformanceSummary,
  QosScoreDetails,
  VerifierQosProof,
} from '@/types'

export const useQoSStore = defineStore('workers', {
  state: () => ({
    workerInfo: null as WorkerInfo | null,
    performanceSummary: null as WorkerPerformanceSummary | null,
    qosDetails: null as QosScoreDetails | null,
    verifierProof: null as VerifierQosProof | null, // 新增：验证者质量证明
    loading: false,
    error: null as string | null,
    currentWorkerId: null as string | null,
    isVerifierProofModalVisible: false, // 新增：控制验证者质量证明模态框显示
  }),

  getters: {
    isLoading: (state) => state.loading,
    getError: (state) => state.error,
    getWorkerInfo: (state) => state.workerInfo,
    getWorkerPerformanceSummary: (state) => state.performanceSummary,
    getQosDetails: (state) => state.qosDetails,
    getVerifierProof: (state) => state.verifierProof, // 新增
  },

  actions: {
    // 获取工作节点信息
    async fetchWorkerInfo(workerId: string) {
      console.log('inside fetchWorkerInfo')
      this.loading = true
      try {
        const info = await ApiService.getWorkerInfo(workerId)
        this.workerInfo = info
        return info
      } catch (error: any) {
        this.error = error.message || `获取工作节点 ${workerId} 信息失败`
        throw error
      } finally {
        this.loading = false
      }
    },

    setCurrentWorkerId(workerId: string) {
      this.currentWorkerId = workerId
    },

    // 获取工作节点QoS评分详情
    async fetchWorkerQosDetails(workerId: string) {
      this.loading = true
      try {
        const details = await ApiService.getWorkerQosDetails(workerId)
        this.qosDetails = details
        return details
      } catch (error: any) {
        this.error = error.message || `获取工作节点 ${workerId} QoS评分详情失败`
        throw error
      } finally {
        this.loading = false
      }
    },

    // 获取工作节点性能摘要
    async fetchWorkerPerformanceSummary(workerId: string) {
      this.loading = true
      try {
        const summary = await ApiService.getWorkerPerformanceSummary(workerId)
        this.performanceSummary = summary
        return summary
      } catch (error: any) {
        this.error = error.message || `获取工作节点 ${workerId} 性能摘要失败`
        throw error
      } finally {
        this.loading = false
      }
    },

    // 一次性获取工作节点所有信息
    async fetchWorkerFullData(workerId: string) {
      this.loading = true
      try {
        // 并行请求所有数据
        const [info, qosDetails, performanceSummary] = await Promise.all([
          this.fetchWorkerInfo(workerId),
          this.fetchWorkerQosDetails(workerId),
          this.fetchWorkerPerformanceSummary(workerId),
        ])

        return {
          info,
          qosDetails,
          performanceSummary,
        }
      } catch (error: any) {
        this.error = error.message || `获取工作节点 ${workerId} 数据失败`
        throw error
      } finally {
        this.loading = false
      }
    },

    // 新增：获取验证者质量证明
    async fetchVerifierProof(taskId: string, verifierId: string) {
      this.loading = true
      try {
        // 这里需要在ApiService中添加相应的方法
        const proof = await ApiService.getVerifierProof(taskId, verifierId)
        this.verifierProof = proof
        return proof
      } catch (error: any) {
        this.error = error.message || `获取验证者 ${verifierId} 的质量证明失败`
        throw error
      } finally {
        this.loading = false
      }
    },

    // 新增：设置验证者质量证明模态框可见性
    setVerifierProofModalVisible(visible: boolean = true) {
      this.isVerifierProofModalVisible = visible
    },

    // 新增：显示验证者质量证明模态框
    // showVerifierProofModal(show: boolean = true) {
    //   this.showVerifierProofModal = show
    // },

    // 新增：查看验证者质量证明
    async viewVerifierProof(taskId: string, verifierId: string) {
      try {
        this.loading = true

        // 调用API获取验证者质量证明
        const proof = await this.fetchVerifierProof(taskId, verifierId)

        // 如果没有找到证明
        if (!proof) {
          // 这里可以使用你的提醒机制，如appStore.setAlert
          console.warn('该验证节点尚未提交评估结果')
          return false
        }

        // 显示验证者质量证明模态框
        this.setVerifierProofModalVisible(true)
        return true
      } catch (error: any) {
        console.error('获取验证者质量证明失败:', error)
        this.error = `获取验证者质量证明失败: ${error.message}`
        return false
      } finally {
        this.loading = false
      }
    },

    // 清除错误
    clearError() {
      this.error = null
    },

    // 重置状态
    resetState() {
      this.workerInfo = null
      this.performanceSummary = null
      this.qosDetails = null
      this.verifierProof = null // 新增
      this.error = null
      this.isVerifierProofModalVisible = false // 新增
    },
  },
})

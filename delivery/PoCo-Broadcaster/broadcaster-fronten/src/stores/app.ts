import { defineStore } from 'pinia'
import ApiService from '@/services/api-service'
import type { BroadcasterConfig } from '@/types'

// 默认应用配置
const defaultConfig: BroadcasterConfig = {
  networkId: 'testnet',
  nodeUrl: 'https://rpc.testnet.near.org',
  walletUrl: 'https://wallet.testnet.near.org',
  helperUrl: 'https://helper.testnet.near.org',
  explorerUrl: 'https://explorer.testnet.near.org',
  contractId: 'pococontract11.testnet',
  broadcasterAccountId: 'pocobroadcaster1.testnet',
  ipfsConfig: {
    host: window.location.hostname,
    port: 5001,
    protocol: 'http',
    gateway: 'http://106.75.224.49:8080',
  },
}

export interface AlertMessage {
  type: 'success' | 'error' | 'info' | 'warning'
  message: string
  timeout?: number
}

export const useAppStore = defineStore('app', {
  state: () => ({
    config: { ...defaultConfig } as BroadcasterConfig,
    alert: null as AlertMessage | null,
    ipfsConnected: false,
  }),

  getters: {
    getConfig: (state) => state.config,
    accountId: (state) => state.config.broadcasterAccountId,
    getAlert: (state) => state.alert,
    isIpfsConnected: (state) => state.ipfsConnected,
    ipfsGateway: (state) => state.config.ipfsConfig.gateway,
  },

  actions: {
    // 初始化应用
    async initialize() {
      try {
        // 获取账户信息
        await this.fetchAccountInfo()

        // 检查IPFS状态
        await this.checkIpfsStatus()

        return true
      } catch (error: any) {
        this.setAlert({
          type: 'error',
          message: '初始化应用失败: ' + (error.message || '未知错误'),
        })
        return false
      }
    },

    // 获取账户信息
    async fetchAccountInfo() {
      try {
        const accountInfo = await ApiService.getAccountInfo()
        this.config.broadcasterAccountId = accountInfo.accountId
        return accountInfo
      } catch (error: any) {
        console.error('获取账户信息失败:', error)
        throw error
      }
    },

    // 检查IPFS状态
    async checkIpfsStatus() {
      try {
        const status = await ApiService.getIpfsStatus()
        this.ipfsConnected = status.status === 'connected'
        return status
      } catch (error) {
        this.ipfsConnected = false
        console.error('IPFS状态检查失败:', error)
        return { status: 'disconnected' }
      }
    },

    // 显示提示信息
    setAlert(alert: AlertMessage) {
      this.alert = alert

      if (alert.timeout !== undefined && alert.timeout > 0) {
        setTimeout(() => {
          this.clearAlert()
        }, alert.timeout)
      }
    },

    // 清除提示信息
    clearAlert() {
      this.alert = null
    },

    // 更新配置
    updateConfig(config: Partial<BroadcasterConfig>) {
      this.config = { ...this.config, ...config }
    },
  },
})

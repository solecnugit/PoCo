// src/services/near-service.ts
import { connect, keyStores, Contract, Account, WalletConnection } from 'near-api-js'
import type { BroadcasterConfig, TaskData, TranscodingRequirement } from '../types'

export class NearService {
  private config: BroadcasterConfig
  private nearConnection: any
  private accountId: string
  private contractId: string
  private contract: any
  private walletConnection: WalletConnection | null = null

  constructor(config: BroadcasterConfig) {
    this.config = config
    this.accountId = config.broadcasterAccountId || ''
    this.contractId = config.contractId
  }

  /**
   * 初始化NEAR连接
   */
  async init(): Promise<boolean> {
    try {
      // 配置密钥存储
      const keyStore = new keyStores.BrowserLocalStorageKeyStore()

      // 连接到NEAR
      this.nearConnection = await connect({
        networkId: this.config.networkId,
        nodeUrl: this.config.nodeUrl,
        walletUrl: this.config.walletUrl,
        helperUrl: this.config.helperUrl,
        keyStore,
      })

      // 初始化钱包连接
      this.walletConnection = new WalletConnection(this.nearConnection, 'poco-broadcaster')

      if (this.walletConnection.isSignedIn()) {
        this.accountId = this.walletConnection.getAccountId()
        this.config.broadcasterAccountId = this.accountId

        // 获取账户对象
        const account = await this.nearConnection.account(this.accountId)

        // 初始化合约接口
        this.contract = new Contract(account, this.contractId, {
          // 视图方法 - 不需要签名
          viewMethods: ['get_available_tasks', 'get_task', 'get_consensus_proof'],
          // 修改方法 - 需要签名
          changeMethods: [
            'publish_task',
            'assign_task',
            'select_verifiers',
            'request_supplemental_verifier',
          ],
          useLocalViewExecution: false,
        })
      }

      console.log(`已连接到NEAR网络，合约ID: ${this.contractId}`)
      return true
    } catch (error) {
      console.error('连接NEAR网络失败:', error)
      return false
    }
  }

  /**
   * 检查是否已登录
   */
  isSignedIn(): boolean {
    return this.walletConnection !== null && this.walletConnection.isSignedIn()
  }

  /**
   * 获取当前账户ID
   */
  getAccountId(): string {
    return this.accountId
  }

  /**
   * 登录NEAR钱包
   */
  // login(): void {
  //   if (this.walletConnection) {
  //     this.walletConnection.requestSignIn({
  //       contractId: this.contractId,
  //       methodNames: ['publish_task', 'assign_task', 'select_verifiers'],
  //     })
  //   }
  // }

  /**
   * 登出NEAR钱包
   */
  logout(): void {
    if (this.walletConnection) {
      this.walletConnection.signOut()
      this.accountId = ''
      this.config.broadcasterAccountId = ''
    }
  }

  /**
   * 发布任务
   */
  async publishTask(sourceIpfs: string, requirements: TranscodingRequirement): Promise<string> {
    try {
      if (!this.isSignedIn() || !this.contract) {
        throw new Error('请先登录NEAR钱包')
      }

      const result = await this.contract.publish_task({
        source_ipfs: sourceIpfs,
        requirements: requirements,
      })

      console.log(`任务已发布，任务ID: ${result}`)
      return result
    } catch (error) {
      console.error('发布任务失败:', error)
      throw error
    }
  }

  /**
   * 分配任务给工作节点
   */
  async assignTask(taskId: string, workerId: string): Promise<boolean> {
    try {
      if (!this.isSignedIn() || !this.contract) {
        throw new Error('请先登录NEAR钱包')
      }

      const result = await this.contract.assign_task({
        task_id: taskId,
        worker_id: workerId,
      })

      console.log(`任务 ${taskId} 已分配给工作节点 ${workerId}`)
      return result
    } catch (error) {
      console.error(`分配任务 ${taskId} 失败:`, error)
      throw error
    }
  }

  /**
   * 获取任务详情
   */
  async getTask(taskId: string): Promise<TaskData | null> {
    try {
      if (!this.contract) {
        await this.init()
      }

      const task = await this.contract.get_task({
        task_id: taskId,
      })

      return task
    } catch (error) {
      console.error(`获取任务 ${taskId} 详情失败:`, error)
      return null
    }
  }

  /**
   * 获取可用任务
   */
  async getAvailableTasks(fromIndex: number = 0, limit: number = 50): Promise<TaskData[]> {
    try {
      if (!this.contract) {
        await this.init()
      }

      const tasks = await this.contract.get_available_tasks({
        from_index: fromIndex,
        limit: limit,
      })

      return tasks
    } catch (error) {
      console.error('获取可用任务失败:', error)
      return []
    }
  }

  /**
   * 选择验证节点
   */
  async selectVerifiers(taskId: string): Promise<string[]> {
    try {
      if (!this.isSignedIn() || !this.contract) {
        throw new Error('请先登录NEAR钱包')
      }

      const verifiers = await this.contract.select_verifiers({
        task_id: taskId,
      })

      console.log(`为任务 ${taskId} 选择的验证节点:`, verifiers)
      return verifiers
    } catch (error) {
      console.error(`为任务 ${taskId} 选择验证节点失败:`, error)
      throw error
    }
  }
}

export default NearService

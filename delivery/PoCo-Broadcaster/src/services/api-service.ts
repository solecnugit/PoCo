// src/services/api-service.ts
import axios from "axios";
import { TaskData, TranscodingRequirement } from "../types";

export class ApiService {
  // 获取任务列表
  static async getTasks(fromIndex = 0, limit = 50): Promise<TaskData[]> {
    const response = await axios.get(
      `/api/tasks?from_index=${fromIndex}&limit=${limit}`
    );
    return response.data;
  }

  // 获取单个任务
  static async getTask(taskId: string): Promise<TaskData> {
    const response = await axios.get(`/api/tasks/${taskId}`);
    return response.data;
  }

  // 更新发布任务方法
  static async createTask(
    sourceIpfs: string,
    requirements: TranscodingRequirement,
    hwAccelerationPreferred: boolean = false
  ): Promise<string> {
    const response = await axios.post("/api/tasks", {
      source_ipfs: sourceIpfs,
      requirements,
      hw_acceleration_preferred: hwAccelerationPreferred,
    });
    return response.data.taskId;
  }

  // 分配任务
  // 更新验证者选择方法
  static async selectAndAssignVerifiers(taskId: string): Promise<string[]> {
    const response = await axios.post(`/api/tasks/${taskId}/verifiers`);
    return response.data.verifiers;
  }
  // 新增检查offer超时方法
  static async checkOfferTimeout(taskId: string): Promise<boolean> {
    const response = await axios.post(
      `/api/tasks/${taskId}/check_offer_timeout`
    );
    return response.data.success;
  }

  static async getTaskOffers(taskId: string): Promise<any[]> {
    const response = await axios.get(`/api/tasks/${taskId}/offers`);
    return response.data;
  }

  // 获取工作节点信息
  static async getWorkerInfo(workerId: string): Promise<any> {
    const response = await axios.get(`/api/workers/${workerId}`);
    return response.data;
  }

  // // 选择验证者
  // static async selectVerifiers(taskId: string): Promise<string[]> {
  //   const response = await axios.post(`/api/tasks/${taskId}/verifiers`);
  //   return response.data.verifiers;
  // }

  // 获取QoS共识证明
  static async getConsensusProof(taskId: string): Promise<any> {
    const response = await axios.get(`/api/consensus/${taskId}`);
    return response.data;
  }
}

export default ApiService;

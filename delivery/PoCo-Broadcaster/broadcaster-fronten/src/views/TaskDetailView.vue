<template>
    <div class="task-detail">
      <div class="d-flex justify-content-between align-items-center mb-4">
        <h2>任务详情</h2>
        <router-link to="/tasks" class="btn btn-outline-primary">
          <i class="bi bi-arrow-left me-1"></i> 返回任务列表
        </router-link>
      </div>
  
      <div v-if="isLoading" class="text-center my-5">
        <div class="spinner-border" role="status">
          <span class="visually-hidden">加载中...</span>
        </div>
        <p class="mt-2">加载任务详情...</p>
      </div>
  
      <div v-else-if="error" class="alert alert-danger" role="alert">
        <i class="bi bi-exclamation-triangle-fill me-2"></i>
        {{ error }}
        <button class="btn btn-sm btn-outline-danger ms-3" @click="loadTask">重试</button>
      </div>
  
      <template v-else-if="task">
        <div class="card border-0 shadow-sm mb-4">
          <div class="card-header bg-primary text-white d-flex justify-content-between align-items-center">
            <h5 class="mb-0">
              <i class="bi bi-info-circle me-2"></i>任务详细信息
            </h5>
            <span class="badge bg-light text-dark">
              <i class="bi bi-clock"></i> {{ formatTimeAgo(task.publish_time) }}
            </span>
          </div>
          <div class="card-body">
            <!-- 基本信息部分 -->
            <div class="row mb-4">
              <div class="col-md-6">
                <div class="d-flex mb-3">
                  <div class="col-md-4 fw-bold text-muted"><i class="bi bi-hash me-1"></i>任务ID:</div>
                  <div class="col-md-8 text-break">{{ task.task_id }}</div>
                </div>
                <div class="d-flex mb-3">
                  <div class="col-md-4 fw-bold text-muted"><i class="bi bi-person me-1"></i>发布者:</div>
                  <div class="col-md-8">{{ task.broadcaster_id }}</div>
                </div>
                <div class="d-flex mb-3">
                  <div class="col-md-4 fw-bold text-muted"><i class="bi bi-reception-4 me-1"></i>状态:</div>
                  <div class="col-md-8">
                    <span 
                      class="badge" 
                      :class="getStatusClass(task.status)"
                      style="font-size: 0.9rem; padding: 0.4em 0.8em;"
                    >
                      {{ getStatusText(task.status) }}
                    </span>
                  </div>
                </div>
              </div>
              <div class="col-md-6">
                <div class="d-flex mb-3">
                  <div class="col-md-4 fw-bold text-muted"><i class="bi bi-calendar me-1"></i>发布时间:</div>
                  <div class="col-md-8">{{ task.publish_time ? formatDate(task.publish_time) : '未指定' }}</div>
                </div>
                <div class="d-flex mb-3">
                  <div class="col-md-4 fw-bold text-muted"><i class="bi bi-calendar-check me-1"></i>分配时间:</div>
                  <div class="col-md-8">{{ task.assignment_time ? formatDate(task.assignment_time) : '未分配' }}</div>
                </div>
                <div class="d-flex mb-3">
                  <div class="col-md-4 fw-bold text-muted"><i class="bi bi-calendar-check-fill me-1"></i>完成时间:</div>
                  <div class="col-md-8">{{ task.completion_time ? formatDate(task.completion_time) : '未完成' }}</div>
                </div>
              </div>
            </div>

            <!-- 文件信息部分 -->
            <div class="card mb-4 border-light">
              <div class="card-header bg-light">
                <h6 class="mb-0"><i class="bi bi-file-earmark me-2"></i>媒体文件信息</h6>
              </div>
              <div class="card-body">
                <div class="row mb-3 align-items-center">
                  <div class="col-md-2 fw-bold text-muted">源文件:</div>
                  <div class="col-md-10 d-flex align-items-center">
                    <button 
                      class="btn btn-primary btn-sm me-2"
                      @click="openIpfsFile(task.source_ipfs)"
                    >
                      <i class="bi bi-play-fill me-1"></i>查看源文件
                    </button>
                    <div class="input-group" style="max-width: 350px;">
                      <span class="input-group-text">CID</span>
                      <input type="text" class="form-control form-control-sm" :value="task.source_ipfs" readonly>
                      <button class="btn btn-outline-secondary btn-sm" @click="copyToClipboard(task.source_ipfs)">
                        <i class="bi bi-clipboard"></i>
                      </button>
                    </div>
                  </div>
                </div>
                <div class="row mb-3 align-items-center">
                  <div class="col-md-2 fw-bold text-muted">转码结果:</div>
                  <div class="col-md-10 d-flex align-items-center">
                    <template v-if="task.result_ipfs">
                      <button 
                        class="btn btn-success btn-sm me-2"
                        @click="openIpfsFile(task.result_ipfs)"
                      >
                        <i class="bi bi-play-fill me-1"></i>查看结果
                      </button>
                      <div class="input-group" style="max-width: 350px;">
                        <span class="input-group-text">CID</span>
                        <input type="text" class="form-control form-control-sm" :value="task.result_ipfs" readonly>
                        <button class="btn btn-outline-secondary btn-sm" @click="copyToClipboard(task.result_ipfs)">
                          <i class="bi bi-clipboard"></i>
                        </button>
                      </div>
                    </template>
                    <span v-else class="badge bg-warning text-dark">
                      <i class="bi bi-hourglass-split me-1"></i>未完成
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

            <div class="card mb-4 border-light" v-if="task.status === 'Completed' || task.status === 'Verified'">
              <div class="card-header bg-light">
                <h6 class="mb-0"><i class="bi bi-film me-2"></i>媒体质量评估采样点</h6>
              </div>
              <div class="card-body">
                <!-- GOP时间轴可视化 -->
                <div v-if="task.selected_gops && task.selected_gops.length > 0" class="mb-4">
                  <div class="d-flex justify-content-between align-items-center mb-2">
                    <div class="fw-bold">GOP采样点位置:</div>
                    <div class="badge bg-info">{{ task.selected_gops.length }} 个采样点</div>
                  </div>
                  <div class="gop-timeline mb-2">
                    <div class="timeline-container">
                      <div class="timeline-bar"></div>
                      <div 
                        v-for="(gopTimestamp, index) in task.selected_gops" 
                        :key="index"
                        class="timeline-marker"
                        :style="{ left: calculatePosition(gopTimestamp, getEstimatedDuration()) + '%' }"
                        :title="`GOP ${index+1}: ${formatTimecode(gopTimestamp)}`"
                      >
                        <span class="marker-label">{{ index + 1 }}</span>
                      </div>
                    </div>
                    <div class="timeline-labels d-flex justify-content-between">
                      <span>开始</span>
                      <span>结束</span>
                    </div>
                  </div>
                  
                  <!-- GOP采样点表格 -->
                  <div class="mt-3">
                    <div class="fw-bold mb-2">采样点详情:</div>
                    <div class="table-responsive">
                      <table class="table table-sm">
                        <thead class="table-light">
                          <tr>
                            <th scope="col">#</th>
                            <th scope="col">时间点</th>
                            <th scope="col">说明</th>
                          </tr>
                        </thead>
                        <tbody>
                          <tr v-for="(gopTimestamp, index) in task.selected_gops" :key="index">
                            <th scope="row">{{ index + 1 }}</th>
                            <td>{{ formatTimecode(gopTimestamp) }}</td>
                            <td class="text-muted">媒体质量评估采样点 {{ index + 1 }}</td>
                          </tr>
                        </tbody>
                      </table>
                    </div>
                  </div>
                </div>
                
                <!-- 改进的关键帧分布显示 -->
<div v-if="task.keyframe_timestamps && task.keyframe_timestamps.length > 0" class="mt-3">
  <div class="d-flex justify-content-between align-items-center mb-2">
    <div class="fw-bold">视频关键帧分布:</div>
    <div class="badge bg-secondary">{{ task.keyframe_timestamps.length }} 个关键帧</div>
  </div>
  
  <!-- 关键帧时间轴 -->
  <div class="keyframe-distribution mb-3 mt-2">
    <div class="distribution-container" style="height: 60px; position: relative;">
      <div class="distribution-bar" style="position: absolute; left: 0; right: 0; top: 30px; height: 2px; background-color: #e9ecef;"></div>
      <div 
        v-for="(kfTimestamp, index) in task.keyframe_timestamps" 
        :key="index"
        class="keyframe-dot"
        :class="{ 'sampled-gop': isSampledGop(kfTimestamp) }"
        :style="{ 
          left: calculatePosition(kfTimestamp, getVideoDuration()) + '%',
          top: getStaggeredPosition(index) + 'px'
        }"
        :title="`关键帧: ${formatTimecode(kfTimestamp)}${isSampledGop(kfTimestamp) ? ' (已采样)' : ''}`"
      ></div>
    </div>
    <div class="distribution-labels d-flex justify-content-between">
      <small class="text-muted">视频开始</small>
      <small class="text-muted">视频结束 ({{ formatSimpleTime(getVideoDuration()) }})</small>
    </div>
  </div>
  
  <!-- 关键帧聚集区域展开视图 -->
  <div v-if="getClusteredFrames().length > 0" class="mt-3">
    <div class="fw-bold mb-2">
      <i class="bi bi-zoom-in me-1"></i>密集关键帧区域:
    </div>
    <div 
      v-for="(cluster, clusterIndex) in getClusteredFrames()" 
      :key="'cluster-'+clusterIndex" 
      class="cluster-view mb-3 p-2 border rounded"
    >
      <div class="d-flex justify-content-between align-items-center mb-2">
        <span class="badge bg-info">
          {{ formatSimpleTime(cluster.startTime) }} - {{ formatSimpleTime(cluster.endTime) }}
        </span>
        <small>{{ cluster.frames.length }} 个关键帧</small>
      </div>
      <div class="cluster-frames">
        <span 
          v-for="(frame, frameIndex) in cluster.frames" 
          :key="'frame-'+frameIndex" 
          class="badge me-2 mb-1"
          :class="isSampledGop(frame) ? 'bg-success' : 'bg-light text-dark'"
        >
          {{ formatTimecode(frame) }}
        </span>
      </div>
    </div>
  </div>
  
  <div class="alert alert-light border">
    <small>
      <i class="bi bi-info-circle me-1"></i>
      绿色点表示被选为质量评估样本的GOP，灰色点表示未被选中的关键帧。时间轴上高度不同是为了避免时间相近的关键帧重叠显示。
    </small>
  </div>
</div>

            <!-- 处理节点信息 -->
            <div class="card mb-4 border-light">
              <div class="card-header bg-light">
                <h6 class="mb-0"><i class="bi bi-cpu me-2"></i>处理节点信息</h6>
              </div>
              <div class="card-body">
                <div class="row mb-3">
                  <div class="col-md-2 fw-bold text-muted">工作节点:</div>
                  <div class="col-md-10 d-flex align-items-center">
                    <span v-if="task.assigned_worker" class="badge bg-info text-dark me-2">
                      <i class="bi bi-pc-display me-1"></i>{{ task.assigned_worker }}
                    </span>
                    <span v-else class="badge bg-secondary me-2">未分配</span>
                    
                    <!-- 添加查看QoS信息的按钮 -->
                    <button 
                      v-if="task.assigned_worker" 
                      class="btn btn-outline-info btn-sm"
                      @click="viewWorkerQoS(task.assigned_worker)"
                    >
                      <i class="bi bi-bar-chart me-1"></i>查看工作节点QoS
                    </button>
                  </div>
                </div>
                <div class="row mb-3">
                <div class="col-md-2 fw-bold text-muted">验证节点:</div>
                <div class="col-md-10">
                  <template v-if="task.assigned_verifiers && task.assigned_verifiers.length > 0">
                    <div v-for="(verifier, index) in task.assigned_verifiers" :key="index" class="mb-2 d-flex align-items-center">
                      <span class="badge bg-secondary me-2">
                        <i class="bi bi-shield-check me-1"></i>{{ verifier }}
                      </span>
                      <button 
                        class="btn btn-outline-info btn-sm" 
                        @click="viewVerifierProof(verifier)"
                        title="查看此验证节点的评估结果"
                      >
                        <i class="bi bi-bar-chart-line me-1"></i>评估详情
                      </button>
                    </div>
                  </template>
                  <span v-else class="badge bg-secondary">未分配</span>
                </div>
              </div>
                <div class="row">
                  <div class="col-md-2 fw-bold text-muted">QoS评估:</div>
                  <div class="col-md-10">
                    <button 
                      v-if="task.qos_proof_id" 
                      class="btn btn-info btn-sm"
                      @click="viewQosDetails"
                    >
                      <i class="bi bi-graph-up me-1"></i>查看QoS报告
                    </button>
                    <span v-else class="badge bg-secondary">
                      <i class="bi bi-hourglass-split me-1"></i>未生成
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      <!-- </template> -->
        
        <div class="card border-0 shadow-sm">
          <div class="card-header bg-light">
            <h5 class="mb-0">转码参数</h5>
          </div>
          <div class="card-body">
            <div class="row mb-3">
              <div class="col-md-3 fw-bold">编解码器:</div>
              <div class="col-md-9">{{ task.requirements.target_codec }}</div>
            </div>
            <div class="row mb-3">
              <div class="col-md-3 fw-bold">目标分辨率:</div>
              <div class="col-md-9">{{ task.requirements.target_resolution }}</div>
            </div>
            <div class="row mb-3">
              <div class="col-md-3 fw-bold">目标比特率:</div>
              <div class="col-md-9">{{ task.requirements.target_bitrate }}</div>
            </div>
            <div class="row mb-3">
              <div class="col-md-3 fw-bold">目标帧率:</div>
              <div class="col-md-9">{{ task.requirements.target_framerate }}</div>
            </div>
            <div class="row mb-3">
              <div class="col-md-3 fw-bold">额外参数:</div>
              <div class="col-md-9">{{ task.requirements.additional_params || '无' }}</div>
            </div>
          </div>
        </div>
        
        <div class="mt-4">
          <button 
            v-if="task.status === 'Published'" 
            class="btn btn-success me-2"
            @click="assignTask"
          >
            分配任务
          </button>
          <button 
            v-if="task.status === 'Completed'" 
            class="btn btn-info me-2"
            @click="selectVerifiers"
          >
            选择验证者
          </button>
        </div>
      </template>
  
      <div v-else class="alert alert-warning" role="alert">
        <i class="bi bi-exclamation-circle-fill me-2"></i>
        任务未找到，请返回任务列表查看有效任务。
      </div>
      

      <!-- 验证者质量证明模态框 -->
      <div 
        class="modal fade" 
        id="verifierProofModal" 
        tabindex="-1" 
        aria-labelledby="verifierProofModalLabel" 
        aria-hidden="true"
        ref="verifierProofModalRef"
      >
        <div class="modal-dialog modal-lg">
          <div class="modal-content">
            <div class="modal-header">
              <h5 class="modal-title" id="verifierProofModalLabel">
                验证节点质量评估结果
                <span v-if="currentVerifierProof" class="badge bg-info ms-2">
                  {{ currentVerifierProof.verifier_id }}
                </span>
              </h5>
              <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
            </div>
            <div class="modal-body">
              <div v-if="!currentVerifierProof" class="alert alert-warning">
                未找到验证节点质量评估结果
              </div>
              
              <template v-else>
                <!-- 基本信息 -->
                <div class="card mb-3">
                  <div class="card-header bg-light">
                    <h6 class="mb-0">基本信息</h6>
                  </div>
                  <div class="card-body">
                    <div class="row mb-2">
                      <div class="col-md-4 fw-bold">任务ID:</div>
                      <div class="col-md-8">{{ currentVerifierProof.task_id }}</div>
                    </div>
                    <div class="row mb-2">
                      <div class="col-md-4 fw-bold">验证节点ID:</div>
                      <div class="col-md-8">{{ currentVerifierProof.verifier_id }}</div>
                    </div>
                    <div class="row mb-2">
                      <div class="col-md-4 fw-bold">评估时间:</div>
                      <div class="col-md-8">{{ formatDate(currentVerifierProof.timestamp) }}</div>
                    </div>
                  </div>
                </div>
                
                <!-- 质量评分 -->
                <div class="card mb-3">
                  <div class="card-header bg-light">
                    <h6 class="mb-0">质量评分</h6>
                  </div>
                  <div class="card-body">
                    <div class="row mb-3">
                      <div class="col-md-4 fw-bold">视频质量分数 (VMAF):</div>
                      <div class="col-md-8">
                        <div class="d-flex align-items-center">
                          <span class="me-2">{{ currentVerifierProof.video_score.toFixed(2) }}</span>
                          <div class="progress" style="height: 10px; width: 200px;">
                            <div 
                              class="progress-bar" 
                              :class="getVmafScoreClass(currentVerifierProof.video_score)"
                              :style="{ width: (currentVerifierProof.video_score) + '%' }">
                            </div>
                          </div>
                          <span class="badge ms-2" :class="getVmafScoreClass(currentVerifierProof.video_score)">
                            {{ getVmafQualityLevel(currentVerifierProof.video_score) }}
                          </span>
                        </div>
                      </div>
                    </div>
                    
                    <div class="row mb-3" v-if="currentVerifierProof.audio_score">
                      <div class="col-md-4 fw-bold">音频质量分数 (PESQ):</div>
                      <div class="col-md-8">
                        <div class="d-flex align-items-center">
                          <span class="me-2">{{ currentVerifierProof.audio_score.toFixed(2) }}</span>
                          <div class="progress" style="height: 10px; width: 200px;">
                            <div 
                              class="progress-bar" 
                              :class="getPesqScoreClass(currentVerifierProof.audio_score)"
                              :style="{ width: (currentVerifierProof.audio_score / 5 * 100) + '%' }">
                            </div>
                          </div>
                        </div>
                      </div>
                    </div>
                    
                    <div class="row mb-2" v-if="currentVerifierProof.sync_score">
                      <div class="col-md-4 fw-bold">音视频同步分数:</div>
                      <div class="col-md-8">{{ currentVerifierProof.sync_score }}</div>
                    </div>
                  </div>
                </div>
                
                <!-- GOP评分 -->
                <div class="card mb-3" v-if="currentVerifierProof.gop_scores && currentVerifierProof.gop_scores.length > 0">
                  <div class="card-header bg-light">
                    <h6 class="mb-0">
                      GOP采样评分
                      <span class="badge bg-secondary ms-2">{{ currentVerifierProof.gop_scores.length }} 个样本</span>
                    </h6>
                  </div>
                  <div class="card-body">
                    <div class="table-responsive">
                      <table class="table table-sm table-hover">
                        <thead class="table-light">
                          <tr>
                            <th scope="col">#</th>
                            <th scope="col">时间点</th>
                            <th scope="col">VMAF评分</th>
                            <th scope="col">哈希</th>
                          </tr>
                        </thead>
                        <tbody>
                          <tr v-for="(gop, index) in currentVerifierProof.gop_scores" :key="index">
                            <th scope="row">{{ index + 1 }}</th>
                            <td>{{ formatTimecode(gop.timestamp) }}</td>
                            <td>
                              <div class="d-flex align-items-center">
                                {{ gop.vmaf_score.toFixed(2) }}
                                <div class="progress ms-2" style="width: 60px; height: 6px;">
                                  <div 
                                    class="progress-bar" 
                                    :class="getVmafScoreClass(gop.vmaf_score)"
                                    :style="{ width: (gop.vmaf_score) + '%' }">
                                  </div>
                                </div>
                              </div>
                            </td>
                            <td>
                              <span class="text-muted" style="font-size: 0.8rem;">
                                {{ truncateHash(gop.hash) }}
                              </span>
                            </td>
                          </tr>
                        </tbody>
                      </table>
                    </div>
                  </div>
                </div>
                
                <!-- 视频规格 -->
                <div class="card mb-3" v-if="currentVerifierProof.video_specs">
                  <div class="card-header bg-light">
                    <h6 class="mb-0">视频规格</h6>
                  </div>
                  <div class="card-body">
                    <div class="row">
                      <div class="col-md-3 fw-bold">编解码器:</div>
                      <div class="col-md-3">{{ currentVerifierProof.video_specs.codec }}</div>
                      <div class="col-md-3 fw-bold">分辨率:</div>
                      <div class="col-md-3">{{ currentVerifierProof.video_specs.resolution }}</div>
                    </div>
                    <div class="row mt-2">
                      <div class="col-md-3 fw-bold">比特率:</div>
                      <div class="col-md-3">{{ currentVerifierProof.video_specs.bitrate }} bps</div>
                      <div class="col-md-3 fw-bold">帧率:</div>
                      <div class="col-md-3">{{ currentVerifierProof.video_specs.framerate }} fps</div>
                    </div>
                  </div>
                </div>
              </template>
            </div>
            <div class="modal-footer">
              <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">关闭</button>
            </div>
          </div>
        </div>
      </div>
  
      <!-- QoS详情模态框 -->
      <div 
        class="modal fade" 
        id="qosModal" 
        tabindex="-1" 
        aria-labelledby="qosModalLabel" 
        aria-hidden="true"
        ref="qosModalRef"
      >
        <div class="modal-dialog modal-lg">
          <div class="modal-content">
            <div class="modal-header">
              <h5 class="modal-title" id="qosModalLabel">QoS 质量报告</h5>
              <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
            </div>
            <div class="modal-body">
              <div v-if="qosLoading" class="text-center my-3">
                <div class="spinner-border" role="status">
                  <span class="visually-hidden">加载中...</span>
                </div>
                <p class="mt-2">加载QoS报告...</p>
              </div>
              
              <div v-else-if="qosError" class="alert alert-danger">
                {{ qosError }}
              </div>
              
              <template v-else-if="qosProof">
                <div class="card mb-3">
                  <div class="card-header bg-light">
                    <h6 class="mb-0">基本信息</h6>
                  </div>
                  <div class="card-body">
                    <div class="row mb-2">
                      <div class="col-md-4 fw-bold">任务ID:</div>
                      <div class="col-md-8">{{ qosProof.task_id }}</div>
                    </div>
                    <div class="row mb-2">
                      <div class="col-md-4 fw-bold">工作节点:</div>
                      <div class="col-md-8">{{ qosProof.worker_id }}</div>
                    </div>
                    <div class="row mb-2">
                      <div class="col-md-4 fw-bold">时间戳:</div>
                      <div class="col-md-8">{{ formatDate(qosProof.timestamp) }}</div>
                    </div>
                    <div class="row mb-2">
                      <div class="col-md-4 fw-bold">状态:</div>
                      <div class="col-md-8">{{ qosProof.status }}</div>
                    </div>
                  </div>
                </div>
                
                <div class="card mb-3">
                  <div class="card-header bg-light">
                    <h6 class="mb-0">质量分数</h6>
                  </div>
                  <div class="card-body">
                    <div class="row mb-2">
                      <div class="col-md-4 fw-bold">视频质量分数 (VMAF):</div>
                      <div class="col-md-8">{{ qosProof.video_score.toFixed(2) }}</div>
                    </div>
                    <div class="row mb-2">
                      <div class="col-md-4 fw-bold">音频质量分数 (PESQ):</div>
                      <div class="col-md-8">
                        {{ qosProof.audio_score > 0 ? qosProof.audio_score.toFixed(2) : 'N/A' }}
                      </div>
                    </div>
                    <div class="row mb-2">
                      <div class="col-md-4 fw-bold">音视频同步分数:</div>
                      <div class="col-md-8">
                        {{ qosProof.sync_score > 0 ? qosProof.sync_score.toFixed(2) : 'N/A' }}
                      </div>
                    </div>
                  </div>
                </div>
  
                <!-- 更多QoS信息 ... -->
              </template>
            </div>
            <div class="modal-footer">
              <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">关闭</button>
            </div>
          </div>
        </div>
      </div>

      <WorkerQoSModal />
    </div>
  </template>

  
  <script setup lang="ts">
import { ref, computed, onMounted, nextTick, onUnmounted, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useTasksStore } from '../stores/tasks';
import { useAppStore } from '../stores/app';
import { useQoSStore } from '@/stores/qos';
import { TaskStatus } from '../types';
import { storeToRefs } from 'pinia';
import WorkerQoSModal from '@/views/WorkerQoSModal.vue';
// import { nextTick } from 'vue';
import bootstrap from 'bootstrap/dist/js/bootstrap.bundle.js';

const loading = ref(false)
const showWorkerQoSModal = ref(false)
const verifierProofModalRef = ref(null);
const currentVerifierProof = computed(() => qosStore.verifierProof);

// 在script setup部分添加
onMounted(async () => {
  await loadTask();
  
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
  
  // 如果是当前查看的任务发生状态变化
  if (task.value && task.value.task_id === taskId) {
    // 显示通知
    appStore.setAlert({
      type: 'info',
      message,
      timeout: 5000
    });
    
    // 重新加载任务以显示最新状态
    loadTask();
  }
};

const route = useRoute();
const router = useRouter();
const tasksStore = useTasksStore();
const appStore = useAppStore();
const qosStore = useQoSStore();

// 在声明qosStore之后，提取store中的响应式状态
const { 
  workerInfo, 
  performanceSummary,  // 这里提取performanceSummary 
  qosDetails,
  verifierProof, isVerifierProofModalVisible 
} = storeToRefs(qosStore);

const isLoading = computed(() => tasksStore.isLoading);
const error = computed(() => tasksStore.getError);
const task = computed(() => tasksStore.getCurrentTask);

// QoS相关
const qosProof = ref(null);
const qosLoading = ref(false);
const qosError = ref('');
const qosModalRef = ref(null);
let qosModal = null;

// 加载任务
const loadTask = async () => {
  try {
    loading.value = true; // 确保在开始加载前设置loading状态
    const taskId = route.params.id as string;
    await tasksStore.fetchTask(taskId);
  } catch (error: any) {
    console.error('加载任务详情失败:', error);
  } finally {
    loading.value = false; // 确保在加载完成后重置loading状态，无论成功还是失败
  }
};

// 获取IPFS URL
const getIpfsUrl = (cid: string) => {
  return `${appStore.getConfig.ipfsConfig.gateway}/ipfs/${cid}`;
};

const formatTimeAgo = (timestamp:number | null) => {
    if (!timestamp) return '未知时间';
    
    const now = Date.now();
    const date = new Date(timestamp / 1000000); // 假设时间戳是纳秒
    const diffInSeconds = Math.floor((now - date) / 1000);
    
    if (diffInSeconds < 60) return '刚刚';
    if (diffInSeconds < 3600) return `${Math.floor(diffInSeconds / 60)}分钟前`;
    if (diffInSeconds < 86400) return `${Math.floor(diffInSeconds / 3600)}小时前`;
    if (diffInSeconds < 604800) return `${Math.floor(diffInSeconds / 86400)}天前`;
    
    return formatDate(timestamp);
  };


// 分配任务
const assignTask = async () => {
  if (!task.value) return;
  
  try {
    const workerId = prompt('请输入工作节点 ID:');
    if (workerId) {
      await tasksStore.assignTask(task.value.task_id, workerId);
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

// 查看工作节点QoS的方法
const viewWorkerQoS = (workerId: string) => {
  // 设置当前工作节点ID
  qosStore.setCurrentWorkerId(workerId);
  
  // 查找模态框元素
  const modalElement = document.getElementById('workerQoSModal');
  if (!modalElement) {
    console.error('找不到模态框元素 workerQoSModal');
    return;
  }
  
  // 初始化并显示模态框
  const modal = new bootstrap.Modal(modalElement);
  modal.show();
  
  // 加载数据
  qosStore.fetchWorkerFullData(workerId);
};

// 查看验证者质量证明
const viewVerifierProof = async (verifierId) => {

  console.log("進入viewVerifierProof")
  if (!task.value) return;
  
  try {
    // 调用store中的方法获取并显示验证者质量证明
    const success = await qosStore.viewVerifierProof(task.value.task_id, verifierId);
    
    if (!success) {
      // 如果获取失败，显示提示信息
      appStore.setAlert({
        type: 'warning',
        message: '该验证节点尚未提交评估结果',
        timeout: 3000
      });
    }
    
    // 如果获取成功，模态框会通过store中的状态自动显示
  } catch (error) {
    console.error('查看验证者质量证明失败:', error);
    appStore.setAlert({
      type: 'error',
      message: `查看验证者质量证明失败: ${error.message}`,
      timeout: 3000
    });
  }
};

  // 打开IPFS文件
  const openIpfsFile = (ipfsHash: string) => {
    if (!ipfsHash) return;
    const ipfsUrl = getIpfsUrl(ipfsHash);
    window.open(ipfsUrl, '_blank');
  };
  
  // 复制到剪贴板
  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text)
      .then(() => {
        // 可以添加一个提示，比如使用 toast 通知
        alert('已复制到剪贴板');
      })
      .catch(err => {
        console.error('复制失败:', err);
      });
  };

// 选择验证者
const selectVerifiers = async () => {
  if (!task.value) return;
  
  try {
    const verifiers = await tasksStore.selectVerifiers(task.value.task_id);
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

// 查看QoS详情
const viewQosDetails = async () => {
  if (!task.value || !task.value.qos_proof_id) return;
  
  qosLoading.value = true;
  qosError.value = '';
  
  try {
    // 初始化模态框
    if (!qosModal && qosModalRef.value) {
      qosModal = new bootstrap.Modal(qosModalRef.value);
    }
    
    // 加载QoS报告
    const proof = await tasksStore.fetchQosProof(task.value.qos_proof_id);
    qosProof.value = proof;
    
    // 显示模态框
    qosModal.show();
  } catch (error: any) {
    qosError.value = `加载QoS报告失败: ${error.message}`;
    console.error('加载QoS报告失败:', error);
  } finally {
    qosLoading.value = false;
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

const formatScore = (score: number) => {
      if (score === undefined || score === null) return 'N/A'
      return score.toFixed(2)
    };

    const formatPercentage = (value: number) => {
      if (value === undefined || value === null) return 'N/A'
      return `${(value * 100).toFixed(2)}%`
    };

    const formatDuration = (duration: number) =>  {
      if (!duration) return 'N/A'
      return `${(duration / 1000).toFixed(2)}秒`
    }

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

// 截断哈希值以便显示
const truncateHash = (hash) => {
  if (!hash) return '';
  if (hash.length <= 16) return hash;
  return `${hash.substring(0, 8)}...${hash.substring(hash.length - 8)}`;
};

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

// 获取预估的视频时长（秒）
const getVideoDuration = () => {
  if (task.value && task.value.video_duration) {
    // 假设video_duration字段以秒为单位
    return task.value.video_duration;
  }
  
  // 如果没有提供视频时长，尝试从关键帧估计
  if (task.value && task.value.keyframe_timestamps && task.value.keyframe_timestamps.length > 0) {
    const lastTimestamp = task.value.keyframe_timestamps[task.value.keyframe_timestamps.length - 1];
    const duration = parseFloat(lastTimestamp);
    return isNaN(duration) ? 10 : (duration + 2); // 加2秒作为缓冲
  }
  
  return 10; // 默认10秒
};

// 计算时间戳在时间轴上的位置百分比
const calculatePosition = (timestamp, totalDuration) => {
  const seconds = parseFloat(timestamp);
  if (isNaN(seconds) || totalDuration <= 0) return 0;
  return (seconds / totalDuration) * 100;
};

// 更精确的时间码格式化，包括毫秒
const formatTimecode = (timestamp) => {
  const totalSeconds = parseFloat(timestamp);
  if (isNaN(totalSeconds)) return "00:00:000";
  
  const mins = Math.floor(totalSeconds / 60);
  const secs = Math.floor(totalSeconds % 60);
  const ms = Math.floor((totalSeconds - Math.floor(totalSeconds)) * 1000);
  
  return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}:${ms.toString().padStart(3, '0')}`;
};

// 检查是否为已采样的GOP
// const isSampledGop = (timestamp) => {
//   if (!task.value || !task.value.selected_gops) return false;
//   return task.value.selected_gops.includes(timestamp);
// };

// 简化的时间格式（只有分:秒）
const formatSimpleTime = (seconds) => {
  if (isNaN(seconds)) return "00:00";
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
};

// 错开显示时间相近的关键帧（交错排列）
const getStaggeredPosition = (index) => {
  // 在基准线（30px）上下错开排列
  return 30 + (index % 3 - 1) * 10;
};

// 识别密集关键帧区域（1秒内有多个关键帧的区域）
const getClusteredFrames = () => {
  if (!task.value || !task.value.keyframe_timestamps || task.value.keyframe_timestamps.length < 2) {
    return [];
  }
  
  const clusters = [];
  let currentCluster = null;
  
  // 按时间排序关键帧
  const sortedFrames = [...task.value.keyframe_timestamps].sort((a, b) => parseFloat(a) - parseFloat(b));
  
  for (let i = 0; i < sortedFrames.length; i++) {
    const currentTime = parseFloat(sortedFrames[i]);
    const nextTime = i < sortedFrames.length - 1 ? parseFloat(sortedFrames[i + 1]) : null;
    
    // 如果这是第一帧或与前一帧间隔超过1秒，创建新聚类
    if (currentCluster === null || (nextTime !== null && nextTime - currentTime < 1.0)) {
      if (currentCluster === null || currentTime - parseFloat(currentCluster.frames[currentCluster.frames.length - 1]) > 1.0) {
        if (currentCluster) {
          currentCluster.endTime = parseFloat(currentCluster.frames[currentCluster.frames.length - 1]);
          clusters.push(currentCluster);
        }
        currentCluster = {
          startTime: currentTime,
          endTime: currentTime,
          frames: [sortedFrames[i]]
        };
      } else {
        currentCluster.frames.push(sortedFrames[i]);
      }
    } else if (currentCluster) {
      currentCluster.frames.push(sortedFrames[i]);
    }
  }
  
  // 添加最后一个聚类
  if (currentCluster && currentCluster.frames.length > 1) {
    currentCluster.endTime = parseFloat(currentCluster.frames[currentCluster.frames.length - 1]);
    clusters.push(currentCluster);
  }
  
  return clusters.filter(cluster => cluster.frames.length > 1);
};

// 获取VMAF评分对应的CSS类
const getVmafScoreClass = (score) => {
  if (score >= 90) return 'bg-success';
  if (score >= 70) return 'bg-info';
  if (score >= 50) return 'bg-warning';
  return 'bg-danger';
};

// 获取VMAF评分对应的质量等级
const getVmafQualityLevel = (score) => {
  if (score >= 95) return '优秀';
  if (score >= 90) return '很好';
  if (score >= 80) return '良好';
  if (score >= 70) return '尚可';
  if (score >= 60) return '一般';
  return '较差';
};

// 获取PESQ评分对应的CSS类
const getPesqScoreClass = (score) => {
  if (score >= 4.0) return 'bg-success';
  if (score >= 3.0) return 'bg-info';
  if (score >= 2.0) return 'bg-warning';
  return 'bg-danger';
};

// 获取预估的视频时长（秒）
const getEstimatedDuration = () => {
  if (!task.value || !task.value.keyframe_timestamps || task.value.keyframe_timestamps.length === 0) {
    return 60; // 默认60秒
  }
  
  // 获取最后一个关键帧时间戳作为预估时长
  const lastTimestamp = task.value.keyframe_timestamps[task.value.keyframe_timestamps.length - 1];
  const duration = parseFloat(lastTimestamp);
  return isNaN(duration) ? 60 : (duration + 5); // 加5秒作为缓冲
};

// 计算时间戳在时间轴上的位置百分比
// const calculatePosition = (timestamp, totalDuration) => {
//   const seconds = parseFloat(timestamp);
//   if (isNaN(seconds) || totalDuration <= 0) return 0;
//   return (seconds / totalDuration) * 100;
// };

// 格式化时间码，从秒转为 MM:SS 格式
// const formatTimecode = (timestamp) => {
//   const seconds = parseFloat(timestamp);
//   if (isNaN(seconds)) return "00:00";
  
//   const mins = Math.floor(seconds / 60);
//   const secs = Math.floor(seconds % 60);
//   return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
// };

// 检查是否为已采样的GOP
const isSampledGop = (timestamp) => {
  if (!task.value || !task.value.selected_gops) return false;
  return task.value.selected_gops.includes(timestamp);
};

watch(() => qosStore.isVerifierProofModalVisible, (isVisible) => {
  if (isVisible && verifierProofModalRef.value) {
    const modal = new bootstrap.Modal(verifierProofModalRef.value);
    modal.show();
    
    // 模态框关闭时重置状态
    verifierProofModalRef.value.addEventListener('hidden.bs.modal', () => {
      qosStore.setVerifierProofModalVisible(false);
    });
  }
});

onMounted(async () => {
  await loadTask();
});


</script>

<style scoped>
.task-detail {
  max-width: 900px;
  margin: 0 auto;
}

.badge {
  font-size: 0.85em;
  padding: 0.5em 0.75em;
}

.gop-timeline {
  padding: 10px 0;
}

.timeline-container {
  position: relative;
  height: 40px;
  width: 100%;
}

.timeline-bar {
  position: absolute;
  top: 20px;
  left: 0;
  right: 0;
  height: 4px;
  background-color: #e9ecef;
  border-radius: 2px;
}

.timeline-marker {
  position: absolute;
  top: 12px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background-color: #0d6efd;
  transform: translateX(-50%);
  cursor: pointer;
  z-index: 2;
}

.marker-label {
  position: absolute;
  top: -20px;
  left: 50%;
  transform: translateX(-50%);
  font-size: 12px;
  font-weight: bold;
  color: #495057;
}

.timeline-labels {
  font-size: 12px;
  color: #6c757d;
}

.keyframe-dot {
  position: absolute;
  top: 15px;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background-color: #adb5bd;
  transform: translate(-50%, -50%);
}

.keyframe-dot {
  position: absolute;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background-color: #adb5bd; /* 灰色 - 默认未选中 */
  transform: translate(-50%, -50%);
  transition: all 0.2s ease;
}

.keyframe-dot.sampled-gop {
  background-color: #198754; /* 绿色 - 被选中的GOP */
  width: 8px;
  height: 8px;
  border: 1px solid #fff;
  box-shadow: 0 0 3px rgba(0,0,0,0.2);
}
</style>
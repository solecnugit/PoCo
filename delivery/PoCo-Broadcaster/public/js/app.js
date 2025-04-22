// public/js/app.js

// 配置
const config = {
    networkId: 'testnet',
    nodeUrl: 'https://rpc.testnet.near.org',
    walletUrl: 'https://wallet.testnet.near.org',
    helperUrl: 'https://helper.testnet.near.org',
    explorerUrl: 'https://explorer.testnet.near.org',
    contractId: 'pococontract1.testnet',
    broadcasterAccountId: '', // 将在登录后设置
    ipfsConfig: {
      host: 'window.location.hostname',
      port: 5001,
      protocol: 'http',
      gateway: 'http://106.75.224.49:8080'
    }
  };
  
  // 服务实例
  let nearService = {
    isSignedIn: () => true, // 我们总是使用本地凭证，所以始终返回true
    getAccountId: () => config.broadcasterAccountId,
    
    // API方法
    async getAvailableTasks(fromIndex = 0, limit = 50) {
      const response = await fetch(`/api/tasks?from_index=${fromIndex}&limit=${limit}`);
      if (!response.ok) throw new Error('获取任务失败');
      return await response.json();
    },
    
    async getTask(taskId) {
      const response = await fetch(`/api/tasks/${taskId}`);
      if (!response.ok) throw new Error('获取任务失败');
      return await response.json();
    },
    
    async publishTask(sourceIpfs, requirements) {
      const response = await fetch('/api/tasks', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ source_ipfs: sourceIpfs, requirements })
      });
      if (!response.ok) throw new Error('发布任务失败');
      const result = await response.json();
      return result.taskId;
    },
    
    async assignTask(taskId, workerId) {
      const response = await fetch(`/api/tasks/${taskId}/assign`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ worker_id: workerId })
      });
      if (!response.ok) throw new Error('分配任务失败');
      return (await response.json()).success;
    },
    
    async selectVerifiers(taskId) {
      const response = await fetch(`/api/tasks/${taskId}/verifiers`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' }
      });
      if (!response.ok) throw new Error('选择验证者失败');
      const result = await response.json();
      return result.verifiers;
    }
  };
// IPFS服务
let ipfsService = {
    async uploadFile(file) {
      try {
        console.log(`开始上传文件到IPFS: ${file.name}, 大小: ${file.size} 字节`);
        
        const formData = new FormData();
        formData.append('file', file);
        
        // 显示进度条
        elements.uploadProgress.classList.remove('d-none');
        const progressBar = elements.uploadProgress.querySelector('.progress-bar');
        progressBar.style.width = '0%';
        progressBar.textContent = '上传中...';
        
        // 通过后端API上传
        const response = await fetch('/api/ipfs/upload', {
          method: 'POST',
          body: formData
        });
        
        // 隐藏进度条
        elements.uploadProgress.classList.add('d-none');
        
        if (!response.ok) {
          throw new Error(`IPFS上传失败: ${response.statusText}`);
        }
        
        const data = await response.json();
        console.log(`文件上传成功, CID: ${data.Hash}`);
        return data.Hash;
      } catch (error) {
        console.error('上传文件到IPFS失败:', error);
        throw error;
      }
    },
    
    getFileUrl(cid) {
      // 使用配置的IPFS网关
      return `${config.ipfsConfig.gateway}/ipfs/${cid}`;
    }
  };
  
  let walletConnection = null;
  
  // DOM元素
  const sections = {
    home: document.getElementById('section-home'),
    tasks: document.getElementById('section-tasks'),
    newTask: document.getElementById('section-new-task'),
    taskDetail: document.getElementById('section-task-detail')
  };
  
  const elements = {
    navHome: document.getElementById('nav-home'),
    navTasks: document.getElementById('nav-tasks'),
    navNewTask: document.getElementById('nav-new-task'),
    walletStatus: document.getElementById('wallet-status'),
    walletConnect: document.getElementById('wallet-connect'),
    tasksTableBody: document.getElementById('tasks-table-body'),
    newTaskForm: document.getElementById('new-task-form'),
    refreshTasksBtn: document.getElementById('refresh-tasks-btn'),
    createTaskBtn: document.getElementById('create-task-btn'),
    getStartedBtn: document.getElementById('get-started-btn'),
    backToTasksBtn: document.getElementById('back-to-tasks-btn'),
    taskDetailContent: document.getElementById('task-detail-content'),
    uploadProgress: document.getElementById('upload-progress'),
    alertModal: new bootstrap.Modal(document.getElementById('alert-modal')),
    alertModalTitle: document.getElementById('alert-modal-title'),
    alertModalBody: document.getElementById('alert-modal-body')
  };
  
 // 初始化应用
async function initApp() {
    console.log('初始化应用...');
    
    try {
      // 获取账户信息
      const response = await fetch('/api/account');
      if (!response.ok) {
        throw new Error('无法获取账户信息');
      }
      
      const accountInfo = await response.json();
      config.broadcasterAccountId = accountInfo.accountId;
      console.log(`已连接账户: ${config.broadcasterAccountId}`);
      
      // 测试IPFS连接
      try {
        const ipfsResponse = await fetch('/api/ipfs/status');
        if (ipfsResponse.ok) {
          const ipfsStatus = await ipfsResponse.json();
          console.log(`IPFS连接状态: ${ipfsStatus.status}, 版本: ${ipfsStatus.version || '未知'}`);
        } else {
          console.warn('无法获取IPFS状态');
        }
      } catch (ipfsError) {
        console.warn('IPFS连接测试失败:', ipfsError);
      }
      
      // 更新钱包状态
      updateWalletStatus();
      
      // 绑定事件
      bindEvents();
      
      // 默认显示首页
      showSection('home');
      
    } catch (error) {
      console.error('初始化应用失败:', error);
      showAlert('错误', '初始化应用失败: ' + error.message);
    }
  }
  

  // 更新钱包状态显示 - 简化版
  function updateWalletStatus() {
    elements.walletStatus.textContent = `已连接: ${config.broadcasterAccountId}`;
    elements.walletConnect.style.display = 'none'; // 隐藏连接按钮，因为我们使用本地凭证
  }
    // 更新钱包状态显示
//   function updateWalletStatus() {
//     const isSignedIn = nearService && nearService.isSignedIn();
    
//     if (isSignedIn) {
//       config.broadcasterAccountId = nearService.getAccountId();
//       elements.walletStatus.textContent = `已连接: ${config.broadcasterAccountId}`;
//       elements.walletConnect.textContent = '断开连接';
//     } else {
//       elements.walletStatus.textContent = '未连接钱包';
//       elements.walletConnect.textContent = '连接钱包';
//     }
//   }
  
  // 绑定事件处理函数
  function bindEvents() {
    // 导航事件
    elements.navHome.addEventListener('click', (e) => {
      e.preventDefault();
      showSection('home');
    });
    
    elements.navTasks.addEventListener('click', (e) => {
      e.preventDefault();
      showSection('tasks');
    });
    
    elements.navNewTask.addEventListener('click', (e) => {
      e.preventDefault();
      showSection('newTask');
    });
    
    elements.getStartedBtn.addEventListener('click', (e) => {
      e.preventDefault();
      showSection('newTask');
    });
    
    elements.backToTasksBtn.addEventListener('click', (e) => {
      e.preventDefault();
      showSection('tasks');
    });
    
    // 钱包连接
    elements.walletConnect.addEventListener('click', handleWalletConnection);
    
    // 任务列表
    elements.refreshTasksBtn.addEventListener('click', loadTasks);
    elements.createTaskBtn.addEventListener('click', () => showSection('newTask'));
    
    // 任务表单
    elements.newTaskForm.addEventListener('submit', handleTaskSubmission);
  }
  
  // 处理钱包连接/断开
  async function handleWalletConnection(e) {
    e.preventDefault();
    
    if (nearService.isSignedIn()) {
      // 已登录，执行登出
      nearService.logout();
      updateWalletStatus();
      showSection('home');
    } else {
      // 未登录，执行登录
      nearService.login();
      // 登录是重定向操作，此处代码不会立即执行
    }
  }
  
  // 显示指定部分，隐藏其他部分
  function showSection(sectionName) {
    // 隐藏所有部分
    Object.values(sections).forEach(section => {
      section.classList.add('d-none');
    });
    
    // 显示指定部分
    sections[sectionName].classList.remove('d-none');
    
    // 更新导航激活状态
    document.querySelectorAll('.nav-link').forEach(link => {
      link.classList.remove('active');
    });
    
    // 设置相应导航为活跃
    switch(sectionName) {
      case 'home':
        elements.navHome.classList.add('active');
        break;
      case 'tasks':
        elements.navTasks.classList.add('active');
        loadTasks(); // 加载任务列表
        break;
      case 'newTask':
        elements.navNewTask.classList.add('active');
        break;
      case 'taskDetail':
        elements.navTasks.classList.add('active');
        break;
    }
  }
  
  // 加载任务列表
  async function loadTasks() {
    // 删除登录检查，因为我们使用本地凭证
    // if (!nearService.isSignedIn()) {
    //   showAlert('请先登录', '请先连接NEAR钱包以查看您的任务。');
    //   showSection('home');
    //   return;
    // }
    
    // 显示加载提示
    elements.tasksTableBody.innerHTML = '<tr><td colspan="6" class="text-center">加载中...</td></tr>';
    
    try {
      // 获取任务 - 使用我们定义的API方法
      const tasks = await nearService.getAvailableTasks(0, 50);
      
      // 过滤出属于当前用户的任务
      const myTasks = tasks.filter(task => task.broadcaster_id === config.broadcasterAccountId);
      
      if (myTasks.length === 0) {
        elements.tasksTableBody.innerHTML = '<tr><td colspan="6" class="text-center">没有找到任何任务。</td></tr>';
        return;
      }
      
      // 生成任务列表HTML
      const tasksHtml = myTasks.map(task => `
        <tr>
          <td>${task.task_id.substring(0, 10)}...</td>
          <td><span class="badge status-${task.status.toLowerCase()}">${task.status}</span></td>
          <td><a href="${ipfsService.getFileUrl(task.source_ipfs)}" target="_blank">${task.source_ipfs.substring(0, 10)}...</a></td>
          <td>${task.requirements.target_codec} / ${task.requirements.target_resolution}</td>
          <td>${new Date(task.assignment_time || 0).toLocaleString()}</td>
          <td>
            <button class="btn btn-sm btn-primary view-task" data-task-id="${task.task_id}">查看</button>
            ${task.status === 'Published' ? 
              `<button class="btn btn-sm btn-success assign-task" data-task-id="${task.task_id}">分配</button>` : 
              ''}
            ${task.status === 'Completed' ? 
              `<button class="btn btn-sm btn-info select-verifiers" data-task-id="${task.task_id}">选择验证者</button>` : 
              ''}
          </td>
        </tr>
      `).join('');
      
      elements.tasksTableBody.innerHTML = tasksHtml;
      
      // 绑定任务操作事件
      document.querySelectorAll('.view-task').forEach(button => {
        button.addEventListener('click', (e) => {
          const taskId = e.target.getAttribute('data-task-id');
          viewTaskDetail(taskId);
        });
      });
      
      document.querySelectorAll('.assign-task').forEach(button => {
        button.addEventListener('click', (e) => {
          const taskId = e.target.getAttribute('data-task-id');
          showAssignTaskDialog(taskId);
        });
      });
      
      document.querySelectorAll('.select-verifiers').forEach(button => {
        button.addEventListener('click', (e) => {
          const taskId = e.target.getAttribute('data-task-id');
          selectVerifiers(taskId);
        });
      });
      
    } catch (error) {
      console.error('加载任务失败:', error);
      elements.tasksTableBody.innerHTML = '<tr><td colspan="6" class="text-center text-danger">加载任务失败，请稍后重试。</td></tr>';
    }
  }
  
  // 查看任务详情
  async function viewTaskDetail(taskId) {
    // 显示加载中
    elements.taskDetailContent.innerHTML = `
      <div class="text-center">
        <div class="spinner-border" role="status">
          <span class="visually-hidden">加载中...</span>
        </div>
      </div>
    `;
    
    showSection('taskDetail');
    
    try {
      // 获取任务详情
      const task = await nearService.getTask(taskId);
      
      // 生成任务详情HTML
      let statusBadge = `<span class="badge status-${task.status.toLowerCase()}">${task.status}</span>`;
      
      let resultContent = '未完成';
      if (task.result_ipfs) {
        resultContent = `
          <a href="${ipfsService.getFileUrl(task.result_ipfs)}" target="_blank" class="btn btn-sm btn-primary">
            查看结果
          </a>
          <span class="ms-2">CID: ${task.result_ipfs}</span>
        `;
      }
      
      let verifiersContent = '未分配';
      if (task.assigned_verifiers && task.assigned_verifiers.length > 0) {
        verifiersContent = task.assigned_verifiers.join('<br>');
      }
      
      const qosContent = task.qos_proof_id ? 
        `<a href="#" class="view-qos" data-qos-id="${task.qos_proof_id}">查看QoS报告</a>` : 
        '未生成';
      
      const detailHtml = `
        <div class="card mb-4">
          <div class="card-header">
            <h5 class="mb-0">任务信息</h5>
          </div>
          <div class="card-body">
            <div class="row mb-3">
              <div class="col-md-3 fw-bold">任务ID:</div>
              <div class="col-md-9">${task.task_id}</div>
            </div>
            <div class="row mb-3">
              <div class="col-md-3 fw-bold">发布者:</div>
              <div class="col-md-9">${task.broadcaster_id}</div>
            </div>
            <div class="row mb-3">
              <div class="col-md-3 fw-bold">状态:</div>
              <div class="col-md-9">${statusBadge}</div>
            </div>
            <div class="row mb-3">
              <div class="col-md-3 fw-bold">源文件:</div>
              <div class="col-md-9">
                <a href="${ipfsService.getFileUrl(task.source_ipfs)}" target="_blank" class="btn btn-sm btn-primary">
                  查看源文件
                </a>
                <span class="ms-2">CID: ${task.source_ipfs}</span>
              </div>
            </div>
            <div class="row mb-3">
              <div class="col-md-3 fw-bold">转码结果:</div>
              <div class="col-md-9">${resultContent}</div>
            </div>
            <div class="row mb-3">
              <div class="col-md-3 fw-bold">工作节点:</div>
              <div class="col-md-9">${task.assigned_worker || '未分配'}</div>
            </div>
            <div class="row mb-3">
              <div class="col-md-3 fw-bold">验证节点:</div>
              <div class="col-md-9">${verifiersContent}</div>
            </div>
            <div class="row mb-3">
              <div class="col-md-3 fw-bold">QoS评估:</div>
              <div class="col-md-9">${qosContent}</div>
            </div>
            <div class="row mb-3">
              <div class="col-md-3 fw-bold">分配时间:</div>
              <div class="col-md-9">${task.assignment_time ? new Date(task.assignment_time).toLocaleString() : '未分配'}</div>
            </div>
            <div class="row mb-3">
              <div class="col-md-3 fw-bold">完成时间:</div>
              <div class="col-md-9">${task.completion_time ? new Date(task.completion_time).toLocaleString() : '未完成'}</div>
            </div>
          </div>
        </div>
        
        <div class="card">
          <div class="card-header">
            <h5 class="mb-0">转码参数</h5>
          </div>
          <div class="card-body">
            <div class="row mb-3">
              <div class="col-md-3 fw-bold">编解码器:</div>
              <div class="col-md-9">${task.requirements.target_codec}</div>
            </div>
            <div class="row mb-3">
              <div class="col-md-3 fw-bold">目标分辨率:</div>
              <div class="col-md-9">${task.requirements.target_resolution}</div>
            </div>
            <div class="row mb-3">
              <div class="col-md-3 fw-bold">目标比特率:</div>
              <div class="col-md-9">${task.requirements.target_bitrate}</div>
            </div>
            <div class="row mb-3">
              <div class="col-md-3 fw-bold">目标帧率:</div>
              <div class="col-md-9">${task.requirements.target_framerate}</div>
            </div>
            <div class="row mb-3">
              <div class="col-md-3 fw-bold">额外参数:</div>
              <div class="col-md-9">${task.requirements.additional_params || '无'}</div>
            </div>
          </div>
        </div>
        
        <div class="mt-4">
          ${task.status === 'Published' ? 
            `<button class="btn btn-success" id="detail-assign-btn" data-task-id="${task.task_id}">分配任务</button>` : 
            ''}
          ${task.status === 'Completed' ? 
            `<button class="btn btn-info" id="detail-select-verifiers-btn" data-task-id="${task.task_id}">选择验证者</button>` : 
            ''}
        </div>
      `;
      
      elements.taskDetailContent.innerHTML = detailHtml;
      
      // 绑定事件
      const assignBtn = document.getElementById('detail-assign-btn');
      if (assignBtn) {
        assignBtn.addEventListener('click', (e) => {
          const taskId = e.target.getAttribute('data-task-id');
          showAssignTaskDialog(taskId);
        });
      }
      
      const selectVerifiersBtn = document.getElementById('detail-select-verifiers-btn');
      if (selectVerifiersBtn) {
        selectVerifiersBtn.addEventListener('click', (e) => {
          const taskId = e.target.getAttribute('data-task-id');
          selectVerifiers(taskId);
        });
      }
      
      // 绑定QoS查看事件
      const qosLink = document.querySelector('.view-qos');
      if (qosLink) {
        qosLink.addEventListener('click', (e) => {
          e.preventDefault();
          const qosId = e.target.getAttribute('data-qos-id');
          viewQosDetail(qosId);
        });
      }
      
    } catch (error) {
      console.error('加载任务详情失败:', error);
      elements.taskDetailContent.innerHTML = `
        <div class="alert alert-danger" role="alert">
          加载任务详情失败，请稍后重试。
        </div>
      `;
    }
  }
  
  // 显示分配任务对话框
  function showAssignTaskDialog(taskId) {
    // 创建一个模态对话框
    const modalId = 'assign-task-modal';
    
    let modalHtml = `
      <div class="modal fade" id="${modalId}" tabindex="-1" aria-hidden="true">
        <div class="modal-dialog">
          <div class="modal-content">
            <div class="modal-header">
              <h5 class="modal-title">分配任务</h5>
              <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
            </div>
            <div class="modal-body">
              <form id="assign-task-form">
                <div class="mb-3">
                  <label for="worker-id" class="form-label">工作节点 ID</label>
                  <input type="text" class="form-control" id="worker-id" required 
                         placeholder="例如: worker.testnet">
                  <div class="form-text">输入要分配任务的工作节点账户ID。</div>
                </div>
              </form>
            </div>
            <div class="modal-footer">
              <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">取消</button>
              <button type="button" class="btn btn-primary" id="confirm-assign-btn">确认分配</button>
            </div>
          </div>
        </div>
      </div>
    `;
    
    // 添加模态框到文档
    const modalContainer = document.createElement('div');
    modalContainer.innerHTML = modalHtml;
    document.body.appendChild(modalContainer);
    
    // 显示模态框
    const modal = new bootstrap.Modal(document.getElementById(modalId));
    modal.show();
    
    // 绑定确认按钮
    document.getElementById('confirm-assign-btn').addEventListener('click', async () => {
      const workerId = document.getElementById('worker-id').value.trim();
      
      if (!workerId) {
        showAlert('错误', '请输入有效的工作节点ID');
        return;
      }
      
      try {
        // 隐藏模态框
        modal.hide();
        
        // 显示加载提示
        showAlert('处理中', '正在分配任务...');
        
        // 调用合约分配任务
        await nearService.assignTask(
          taskId,
          workerId
        );
        
        // 隐藏提示，显示成功消息
        setTimeout(() => {
          document.querySelector('.modal-backdrop')?.remove();
          showAlert('成功', `任务成功分配给工作节点 ${workerId}`);
          
          // 刷新任务列表
          if (sections.tasks.classList.contains('d-none')) {
            // 如果在详情页面，刷新详情
            viewTaskDetail(taskId);
          } else {
            // 如果在列表页面，刷新列表
            loadTasks();
          }
        }, 500);
        
      } catch (error) {
        console.error('分配任务失败:', error);
        showAlert('错误', '分配任务失败：' + (error.message || '未知错误'));
      }
    });
    
    // 模态框关闭时移除
    document.getElementById(modalId).addEventListener('hidden.bs.modal', function () {
      document.body.removeChild(modalContainer);
    });
  }
  
  // 选择验证者
  async function selectVerifiers(taskId) {
    try {
      // 显示加载提示
      showAlert('处理中', '正在选择验证者...');
      
      // 调用合约选择验证者
      const verifiers = await nearService.contract.select_verifiers({
        task_id: taskId
      });
      
      showAlert('成功', `已为任务选择验证者: ${verifiers.join(', ')}`);
      
      // 刷新任务信息
      if (sections.tasks.classList.contains('d-none')) {
        // 如果在详情页面，刷新详情
        viewTaskDetail(taskId);
      } else {
        // 如果在列表页面，刷新列表
        loadTasks();
      }
      
    } catch (error) {
      console.error('选择验证者失败:', error);
      showAlert('错误', '选择验证者失败：' + (error.message || '未知错误'));
    }
  }
  
  // 查看QoS报告
  async function viewQosDetail(qosId) {
    try {
      // 获取QoS详情
      const qosProof = await nearService.contract.get_consensus_proof({
        task_id: qosId
      });
      
      // 创建一个模态对话框
      const modalId = 'qos-detail-modal';
      
      let modalHtml = `
        <div class="modal fade" id="${modalId}" tabindex="-1" aria-hidden="true">
          <div class="modal-dialog modal-lg">
            <div class="modal-content">
              <div class="modal-header">
                <h5 class="modal-title">QoS 质量报告</h5>
                <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
              </div>
              <div class="modal-body">
                <div class="card mb-3">
                  <div class="card-header">
                    <h6 class="mb-0">基本信息</h6>
                  </div>
                  <div class="card-body">
                    <div class="row mb-2">
                      <div class="col-md-4 fw-bold">任务ID:</div>
                      <div class="col-md-8">${qosProof.task_id}</div>
                    </div>
                    <div class="row mb-2">
                      <div class="col-md-4 fw-bold">工作节点:</div>
                      <div class="col-md-8">${qosProof.worker_id}</div>
                    </div>
                    <div class="row mb-2">
                      <div class="col-md-4 fw-bold">时间戳:</div>
                      <div class="col-md-8">${new Date(qosProof.timestamp).toLocaleString()}</div>
                    </div>
                    <div class="row mb-2">
                      <div class="col-md-4 fw-bold">状态:</div>
                      <div class="col-md-8">${qosProof.status}</div>
                    </div>
                  </div>
                </div>
                
                <div class="card mb-3">
                  <div class="card-header">
                    <h6 class="mb-0">质量分数</h6>
                  </div>
                  <div class="card-body">
                    <div class="row mb-2">
                      <div class="col-md-4 fw-bold">视频质量分数 (VMAF):</div>
                      <div class="col-md-8">${qosProof.video_score.toFixed(2)}</div>
                    </div>
                    <div class="row mb-2">
                      <div class="col-md-4 fw-bold">音频质量分数 (PESQ):</div>
                      <div class="col-md-8">${qosProof.audio_score > 0 ? qosProof.audio_score.toFixed(2) : 'N/A'}</div>
                    </div>
                    <div class="row mb-2">
                      <div class="col-md-4 fw-bold">音视频同步分数:</div>
                      <div class="col-md-8">${qosProof.sync_score > 0 ? qosProof.sync_score.toFixed(2) : 'N/A'}</div>
                    </div>
                  </div>
                </div>
                
                <div class="card mb-3">
                  <div class="card-header">
                    <h6 class="mb-0">编码信息</h6>
                  </div>
                  <div class="card-body">
                    <div class="row mb-2">
                      <div class="col-md-4 fw-bold">编码开始时间:</div>
                      <div class="col-md-8">${new Date(qosProof.encoding_start_time).toLocaleString()}</div>
                    </div>
                    <div class="row mb-2">
                      <div class="col-md-4 fw-bold">编码结束时间:</div>
                      <div class="col-md-8">${new Date(qosProof.encoding_end_time).toLocaleString()}</div>
                    </div>
                    <div class="row mb-2">
                      <div class="col-md-4 fw-bold">编码耗时:</div>
                      <div class="col-md-8">${((qosProof.encoding_end_time - qosProof.encoding_start_time)/1000).toFixed(2)} 秒</div</div>
                  <div class="row mb-2">
                    <div class="col-md-4 fw-bold">编码器:</div>
                    <div class="col-md-8">${qosProof.video_specs.codec}</div>
                  </div>
                  <div class="row mb-2">
                    <div class="col-md-4 fw-bold">分辨率:</div>
                    <div class="col-md-8">${qosProof.video_specs.resolution}</div>
                  </div>
                  <div class="row mb-2">
                    <div class="col-md-4 fw-bold">比特率:</div>
                    <div class="col-md-8">${qosProof.video_specs.bitrate} bps</div>
                  </div>
                  <div class="row mb-2">
                    <div class="col-md-4 fw-bold">帧率:</div>
                    <div class="col-md-8">${qosProof.video_specs.framerate} fps</div>
                  </div>
                </div>
              </div>
              
              <div class="card mb-3">
                <div class="card-header">
                  <h6 class="mb-0">委员会信息</h6>
                </div>
                <div class="card-body">
                  <div class="row mb-2">
                    <div class="col-md-4 fw-bold">委员会成员:</div>
                    <div class="col-md-8">${qosProof.committee_members.join(', ')}</div>
                  </div>
                  <div class="row mb-2">
                    <div class="col-md-4 fw-bold">委员会领导者:</div>
                    <div class="col-md-8">${qosProof.committee_leader}</div>
                  </div>
                </div>
              </div>
              
              <div class="card">
                <div class="card-header">
                  <h6 class="mb-0">GOP验证</h6>
                </div>
                <div class="card-body">
                  <div class="row mb-2">
                    <div class="col-md-4 fw-bold">验证结果:</div>
                    <div class="col-md-8">${qosProof.gop_verification}</div>
                  </div>
                  <div class="row mb-2">
                    <div class="col-md-4 fw-bold">指定GOP数量:</div>
                    <div class="col-md-8">${qosProof.specified_gop_scores.length}</div>
                  </div>
                  
                  ${qosProof.specified_gop_scores.length > 0 ? `
                  <div class="table-responsive mt-3">
                    <table class="table table-sm table-bordered">
                      <thead>
                        <tr>
                          <th>GOP ID</th>
                          <th>VMAF分数</th>
                          <th>哈希 (前10位)</th>
                        </tr>
                      </thead>
                      <tbody>
                        ${qosProof.specified_gop_scores.map(gop => `
                          <tr>
                            <td>${gop.gop_id}</td>
                            <td>${gop.vmaf_score.toFixed(2)}</td>
                            <td>${gop.hash.substring(0, 10)}...</td>
                          </tr>
                        `).join('')}
                      </tbody>
                    </table>
                  </div>
                  ` : ''}
                </div>
              </div>
            </div>
            <div class="modal-footer">
              <button type="button" class="btn btn-primary" data-bs-dismiss="modal">关闭</button>
            </div>
          </div>
        </div>
      </div>
    `;
    
    // 添加模态框到文档
    const modalContainer = document.createElement('div');
    modalContainer.innerHTML = modalHtml;
    document.body.appendChild(modalContainer);
    
    // 显示模态框
    const modal = new bootstrap.Modal(document.getElementById(modalId));
    modal.show();
    
    // 模态框关闭时移除
    document.getElementById(modalId).addEventListener('hidden.bs.modal', function () {
      document.body.removeChild(modalContainer);
    });
    
  } catch (error) {
    console.error('获取QoS详情失败:', error);
    showAlert('错误', '获取QoS详情失败：' + (error.message || '未知错误'));
  }
}

// 处理任务提交
async function handleTaskSubmission(e) {
  e.preventDefault();
  
  if (!nearService.isSignedIn()) {
    showAlert('请先登录', '请先连接NEAR钱包以发布任务。');
    return;
  }
  
  // 获取表单数据
  const sourceFile = document.getElementById('source-file').files[0];
  const targetCodec = document.getElementById('target-codec').value;
  const targetResolution = document.getElementById('target-resolution').value;
  const targetBitrate = document.getElementById('target-bitrate').value;
  const targetFramerate = document.getElementById('target-framerate').value;
  const additionalParams = document.getElementById('additional-params').value;
  
  if (!sourceFile) {
    showAlert('错误', '请选择要转码的源文件');
    return;
  }
  
  try {
    // 显示上传进度
    elements.uploadProgress.classList.remove('d-none');
    const progressBar = elements.uploadProgress.querySelector('.progress-bar');
    progressBar.style.width = '0%';
    progressBar.textContent = '准备上传...';
    
    // 禁用提交按钮
    const submitButton = document.getElementById('submit-task-btn');
    submitButton.disabled = true;
    submitButton.textContent = '上传中...';
    
    // 上传文件到IPFS
    progressBar.textContent = '上传到IPFS...';
    const cid = await ipfsService.uploadFile(sourceFile);
    
    // 更新进度
    progressBar.style.width = '50%';
    progressBar.textContent = '创建任务...';
    
    // 创建转码要求对象
    const requirements = {
      target_codec: targetCodec,
      target_resolution: targetResolution,
      target_bitrate: targetBitrate,
      target_framerate: targetFramerate,
      additional_params: additionalParams
    };

    console.log("before publish task!!!")
    
    // 发布任务到合约
    const taskId = await nearService.publishTask(cid, requirements);
    
    // 更新进度
    progressBar.style.width = '100%';
    progressBar.textContent = '完成';
    
    // 重置表单
    document.getElementById('new-task-form').reset();
    
    // 隐藏进度条
    setTimeout(() => {
      elements.uploadProgress.classList.add('d-none');
      submitButton.disabled = false;
      submitButton.textContent = '创建任务';
    }, 1000);
    
    // 显示成功消息
    showAlert('成功', `任务已创建，任务ID: ${taskId}`);
    
    // 显示任务列表
    setTimeout(() => showSection('tasks'), 2000);
    
  } catch (error) {
    console.error('创建任务失败:', error);
    
    // 重置UI状态
    elements.uploadProgress.classList.add('d-none');
    document.getElementById('submit-task-btn').disabled = false;
    document.getElementById('submit-task-btn').textContent = '创建任务';
    
    // 显示错误
    showAlert('错误', '创建任务失败：' + (error.message || '未知错误'));
  }
}

// 显示提示对话框
function showAlert(title, message) {
  elements.alertModalTitle.textContent = title;
  elements.alertModalBody.textContent = message;
  elements.alertModal.show();
}

// 初始化页面
document.addEventListener('DOMContentLoaded', () => {
  // 加载NEAR API脚本
  const nearApiScript = document.createElement('script');
  nearApiScript.src = 'https://cdn.jsdelivr.net/npm/near-api-js@1.1.0/dist/near-api-js.min.js';
  nearApiScript.onload = () => {
    // NEAR API加载完成后初始化应用
    initApp();
  };
  document.head.appendChild(nearApiScript);
});
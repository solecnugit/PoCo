<template>
  <div class="home">
    <div class="jumbotron bg-light p-5 rounded-3 shadow-sm">
      <h1 class="display-4">欢迎使用PoCo媒体转码平台</h1>
      <p class="lead">PoCo是一个基于区块链的分布式媒体转码系统，提供高质量、低成本的媒体转码服务。</p>
      <hr class="my-4">
      <p>通过区块链技术，我们将闲置的计算资源整合起来，提供可靠的媒体转码服务，同时保证服务质量。</p>
      <div class="mt-4">
        <router-link to="/tasks/new" class="btn btn-primary btn-lg">
          开始使用
        </router-link>
        <router-link to="/tasks" class="btn btn-outline-secondary btn-lg ms-3">
          查看任务
        </router-link>
      </div>
    </div>

    <div class="row mt-5">
      <div class="col-md-4 mb-4">
        <div class="card h-100 border-0 shadow-sm">
          <div class="card-body text-center">
            <div class="feature-icon bg-primary bg-gradient text-white rounded-circle mb-3">
              <i class="bi bi-currency-bitcoin"></i>
            </div>
            <h4 class="card-title">基于区块链</h4>
            <p class="card-text">使用区块链技术确保交易透明、数据不可篡改，为服务质量提供可信保障。</p>
          </div>
        </div>
      </div>
      <div class="col-md-4 mb-4">
        <div class="card h-100 border-0 shadow-sm">
          <div class="card-body text-center">
            <div class="feature-icon bg-success bg-gradient text-white rounded-circle mb-3">
              <i class="bi bi-speedometer2"></i>
            </div>
            <h4 class="card-title">高质量转码</h4>
            <p class="card-text">通过分布式质量评估机制，确保转码结果达到预期质量标准。</p>
          </div>
        </div>
      </div>
      <div class="col-md-4 mb-4">
        <div class="card h-100 border-0 shadow-sm">
          <div class="card-body text-center">
            <div class="feature-icon bg-info bg-gradient text-white rounded-circle mb-3">
              <i class="bi bi-coin"></i>
            </div>
            <h4 class="card-title">低成本服务</h4>
            <p class="card-text">利用闲置计算资源，大幅降低转码成本，为内容创作者提供经济高效的解决方案。</p>
          </div>
        </div>
      </div>
    </div>

    <div class="row mt-4">
      <div class="col-lg-6 mb-4">
        <div class="card border-0 shadow-sm">
          <div class="card-body">
            <h4 class="card-title">如何使用</h4>
            <ol class="list-group list-group-numbered mt-3">
              <li class="list-group-item border-0">上传需要转码的源媒体文件</li>
              <li class="list-group-item border-0">选择目标编解码器、分辨率等参数</li>
              <li class="list-group-item border-0">提交任务，系统将自动选择最佳工作节点</li>
              <li class="list-group-item border-0">等待转码完成并下载结果文件</li>
            </ol>
          </div>
        </div>
      </div>
      <div class="col-lg-6 mb-4">
        <div class="card border-0 shadow-sm">
          <div class="card-body">
            <h4 class="card-title">转码统计</h4>
            <div class="table-responsive">
              <table class="table table-borderless">
                <tbody>
                  <tr>
                    <td>已完成任务</td>
                    <td><span class="badge bg-success">582</span></td>
                  </tr>
                  <tr>
                    <td>活跃工作节点</td>
                    <td><span class="badge bg-primary">128</span></td>
                  </tr>
                  <tr>
                    <td>平均转码时间</td>
                    <td><span class="badge bg-info">5分32秒</span></td>
                  </tr>
                  <tr>
                    <td>平均服务质量评分</td>
                    <td><span class="badge bg-warning">93.7</span></td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
export default {
  name: 'HomeView',
  setup() {
    // 这里可以添加首页的响应式数据和方法
    return {};
  }
}
</script>

<style scoped>
.jumbotron {
  margin-bottom: 2rem;
  background-color: #f8f9fa;
  padding: 2rem;
  border-radius: 0.5rem;
}

.feature-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 4rem;
  height: 4rem;
  font-size: 2rem;
}

.card {
  transition: transform 0.3s ease;
}

.card:hover {
  transform: translateY(-5px);
}

.list-group-item {
  padding: 0.75rem 0;
}
</style>
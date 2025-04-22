#!/bin/bash

# 设置环境变量
export NODE_ENV=production

# 尝试从.env文件加载配置
if [ -f ".env" ]; then
  echo "找到.env文件，将使用其中的配置"
else
  echo "警告：未找到.env文件，将使用默认配置或脚本中设置的配置"
fi

# 确保代码已编译
echo "检查是否需要编译..."
if [ ! -d "./dist" ] || [ ! -f "./dist/index.js" ]; then
  echo "正在编译TypeScript代码..."
  npm run build
fi

# 根据当前节点角色获取API端口
NODE_PORT=$(grep "PORT=" .env | cut -d= -f2)
API_PORT=$((NODE_PORT + 1000))

# 启动节点
NODE_ROLE=$(grep "IS_LEADER=" .env | cut -d= -f2)
if [ "$NODE_ROLE" = "true" ]; then
  ROLE_NAME="Leader"
else
  ROLE_NAME="Follower"
  NODE_ID=$(grep "NODE_ID=" .env | cut -d= -f2)
  ROLE_NAME="$ROLE_NAME:$NODE_ID"
fi

echo "正在启动${ROLE_NAME}节点..."
node --max-old-space-size=4096 ./dist/index.js > ${ROLE_NAME}.log 2>&1 &

# 保存进程ID以便后续管理
echo $! > ${ROLE_NAME}.pid
echo "${ROLE_NAME}节点已启动，进程ID: $(cat ${ROLE_NAME}.pid)"
echo "日志输出重定向到${ROLE_NAME}.log"

# 等待几秒确保节点启动
sleep 5

# 检查节点是否正常启动
if curl -s http://localhost:${API_PORT}/health | grep -q "ok"; then
  echo "${ROLE_NAME}节点API服务健康检查通过"
else
  echo "警告: ${ROLE_NAME}节点API服务可能未正常启动"
fi
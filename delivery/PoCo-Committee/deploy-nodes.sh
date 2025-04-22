#!/bin/bash
# deploy.sh
# 配置
REPO_URL="your-git-repo-url"
BRANCH="main" 
NODES=(
  "leader:10.24.136.124:8000:true"
  "follower1:10.24.216.33:8000:false" 
  "follower2:10.24.198.225:8000:false"
  "follower3:10.24.161.186:8000:false"
)
TOTAL_NODES=${#NODES[@]}
# 构建所有节点的peers列表
function build_peers() {
  local node_id=$1
  local peers=""

  for node in "${NODES[@]}"; do
    IFS=':' read -r id ip port is_leader <<< "$node"
    if [ "$id" != "$node_id" ]; then
      if [ -n "$peers" ]; then
        peers="${peers},"
      fi
      peers="${peers}${id}:${ip}:${port}"
    fi
  done

  echo "$peers"
}
# 为每个节点创建并部署配置
for node in "${NODES[@]}"; do
  IFS=':' read -r id ip port is_leader <<< "$node"
  echo "配置节点 $id ($ip)..."

  # 构建环境变量文件
  cat > ".env.${id}" << EOF
NODE_ID=${id}
IS_LEADER=${is_leader}
PORT=${port}
PEERS=$(build_peers "$id")
TOTAL_NODES=${TOTAL_NODES}
EOF
  # 可选：部署到远程服务器
  # scp ".env.${id}" "user@${ip}:/path/to/app/.env"
  # ssh "user@${ip}" "cd /path/to/app && npm install && npm run build && pm2 restart app || pm2 start npm --name 'app' -- start"

  echo "节点 $id 配置完成"
done
echo "所有节点配置完成"
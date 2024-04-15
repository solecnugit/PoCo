## 创建虚拟环境：

`conda activate py3810`

## build protocol

`python -m grpc_tools.protoc --proto_path=proto/ --python_out=. --grpc_python_out=. transcoding.proto `

## configuration before launch server & client

`unset http_proxy`

`unset https_proxy`

## launch server & client

`python -m transcodeserver`

`python -m transcodeclient`

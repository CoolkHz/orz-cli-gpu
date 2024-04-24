# 先更新bc库:apt-get install bc
# 手续费
gasFee=50000
# 目标轨迹地址（你钱包的orz token_account）
targetAddress="F8Bg6LVvt4PdKKACx5u3qbhANFz1LxUvnD8XkzhrnToW"
# RPC
httpsRPC="你的 rpc"

for file in $(ls -1rt /root/workspace/ore/*.json); do
    current_node="${rpc_nodes[$node_index]}"
    filename=$(basename "$file" .json)

    value=$(/root/orz-cli/target/release/orz --rpc $httpsRPC --keypair /$i.json rewards | grep -o '[0-9.]\+' | awk '{print $1}')
    if [ $(echo "$value > 0" | bc) -eq 1 ]; then
        /root/orz-cli/target/release/orz --rpc $httpsRPC --priority-fee $gasFee --keypair $keyFileName claim $value $targetAddress
    else
        echo "账户已提取"
    fi
done
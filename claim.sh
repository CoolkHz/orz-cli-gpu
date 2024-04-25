#!/bin/bash

priority_fee=50000

rpc_url="你的 rpc"

// 你的钱包的 orz 的 token account，注意这不是钱包地址。可在 solscan.io 你钱包地址的 portfolio 查看。
claim_address="F8Bg6LVvt4PdKKACx5u3qbhANFz1LxUvnD8XkzhrnToW"

# 定义包含密钥对文件的数组
keypairs=(/root/workspace/ore/*.json)

# 遍历每个密钥对文件，并执行ore命令来查询ore
for keypair in "${keypairs[@]}"
do
    # 从路径中提取钱包名
    filename=$(basename "$keypair")

    # 执行ore命令并提取结果
    result=$(/root/orz-cli-master/target/release/orz --rpc "$rpc_url" --keypair "$keypair" rewards)

    # 从结果中提取ore数量
    number=$(echo "$result" | awk '{print $1}')  # 

    # 如果ore数量小于 0.1，则跳过当前文件并继续处理下一个文件
    if (( $(echo "$number < 0.01" | bc -l) )); then
        echo "ORE小于 0.01，跳过Chaim钱包: $filename"
        continue
    fi

    # 将文件名和数字打印出来
    echo "钱包: $filename, ORE: $number"

    # 组装命令并执行
    /root/orz-cli-master/target/release/orz --rpc "$rpc_url" --keypair /root/key1/"$filename" --priority-fee "$priority_fee" claim $number $claim_address
done
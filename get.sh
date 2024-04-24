#!/bin/bash

RPC_URL='https://api.mainnet-beta.solana.com'

totalRewards=0
totalBalance=0
for keyFileName in $(ls -1rt /root/workspace/ore/*.json); do
        walletName=$(echo "$keyFileName" | awk -F '/' '{print $NF}')
        addr=$(solana-keygen pubkey $keyFileName)
        balance=$(solana balance -u $RPC_URL $addr)
        rewards=$(/root/.cargo/bin/ore --rpc $RPC_URL --keypair $keyFileName rewards)
        if [ $? -ne 1 ]; then
            rs_array=($rewards)
            totalRewards=$(bc <<<"$totalRewards + ${rs_array[0]}")
        fi
        bs_array=($balance)
        totalBalance=$(bc <<<"$totalBalance + ${bs_array[0]}")
        rewards_value=$(echo $rewards | awk '{print $1}')
        balance_value=$(echo $balance | awk '{print $1}')
        rewards_unit=$(echo $rewards | awk '{print $2}')
        balance_unit=$(echo $balance | awk '{print $2}')
        printf "钱包：%s 公钥：%s 产量：%.9f %s GAS余额：%.9f %s\n" "$walletName" "$addr" "$rewards_value" "$rewards_unit" "$balance_value" "$balance_unit"
done

current_date_time=$(date -u "+%Y-%m-%d %H:%M:%S" -d "+8 hour")
echo "当前时间：$current_date_time"
printf "当前总产量: %.9f ORZ GAS余额: %.9f SOL\n" $totalRewards $totalBalance
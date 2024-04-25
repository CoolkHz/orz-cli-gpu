# Orz CLI GPU
A command line interface for the Orz program.

# ORZ捡钱项目配置流程

**使用之前麻烦右上角点个 Star🌟，感谢您的支持！**

*建议v我10000000000顿kfc*

联系方式：[@coolkhz](https://t.me/coolkhz)

**归集暂时不好使，等后续更新归集脚本。目前只能手动归集。**

## 配置环境

前提是你的是**GPU**服务器并有N卡显卡驱动和CUDA开发环境!!!

### 切换用户

```
sudo -i
```
切换至 `root` 用户

### 安装Rust

```
// 执行下方两条命令配置临时源(国内服务器💻)
export RUSTUP_DIST_SERVER="https://rsproxy.cn"
export RUSTUP_UPDATE_ROOT="https://rsproxy.cn/rustup"

// 安装 rust 环境
curl --proto '=https' --tlsv1.2 -sSf https://rsproxy.cn/rustup-init.sh | sh

//执行下方命令配置环境变量
. "$HOME/.cargo/env"

//配置源
vim ~/.cargo/config

//将下方配置粘贴进去并保存

[source.crates-io]
replace-with = 'rsproxy-sparse'
[source.rsproxy]
registry = "https://rsproxy.cn/crates.io-index"
[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"
[registries.rsproxy]
index = "https://rsproxy.cn/crates.io-index"
[net]
git-fetch-with-cli = true


//国外服务器环境配置💻
curl https://sh.rustup.rs -sSf | sh

. "$HOME/.cargo/env"

```

### 安装Solana

```
// 下载程序 (国内服务器)
wget https://mirror.ghproxy.com/https://github.com/solana-labs/solana/releases/download/v1.17.28/solana-release-x86_64-unknown-linux-gnu.tar.bz2

// 解压压缩包
tar jxf solana-release-x86_64-unknown-linux-gnu.tar.bz2

// 设置环境变量(注意自己的目录)
export PATH="/root/solana-release/bin:$PATH"

// 查看是否安装成功
solana --version


//国外服务器
sh -c "$(curl -sSfL https://release.solana.com/v1.18.4/install)"

// 设置环境变量
export PATH="/root/.local/share/solana/install/active_release/bin:$PATH"

// 查看是否安装成功
solana --version
```

### 下载并编译 ORZ至尊无敌托马斯螺旋升天翻跟头GPU版本

```
// 下载程序
git clone https://github.com/CoolkHz/orz-cli-gpu.git

// 打开程序目录
cd orz-cli-gpu

// 更改权限
chmod +x ore_miner

// 创建编译目录
mkdir target && mkdir target/release

// 编译 orz
cargo build --release

// 更改权限
chmod 777 build-cuda-worker.sh

// 编译(可能就有报错，忽略即可。)
./build-cuda-worker.sh

// 赋予权限并移动到工作目录
chmod +x target/release/ore && mv target/release/ore /root/.cargo/bin/

chmod +x target/release/nonce-worker-gpu && mv target/release/nonce-worker-gpu /root/.cargo/bin/
```

### 配置 RPC

```
// 创建钱包和 rpc 目录
mkdir /root/workspace && mkdir /root/workspace/ore
```

创建 rpc.txt 文件并存入 `/root/workspace/ore/` 路径。

```
// rpc文件为 https链接+空格+ws链接，每行一个 rpc 配置。
https://rpc.com/rpc wss://rpc.com/rpc
```

RPC 尽量**多搞**。

### 配置钱包

```

// 将主钱包私钥命名为000.json 存入/root/workspace/ore/

// 创建钱包 num后面的数字为钱包数量
/root/orz-cli-gpu/ore_miner wallet new --num 17 -d /root/workspace/ore/

// 分发 sol(-s 后面的数字是给每个钱包分发的数量  小于这个数量就会进行分发转账)
/root/orz-cli-gpu/ore_miner wallet transfer -s 0.01

```

### 安装node/npm/pm2 (可选)

```
curl -o- https://mirror.ghproxy.com/https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.3/install.sh | bash

source /root/.bashrc

nvm install node

//国外服务器可以不操作这条命令
npm config set registry https://mirrors.huaweicloud.com/repository/npm/

npm install pm2@latest -g
```

## 配置sh等脚本监控

启动命令 `/root/orz-cli-gpu/ore_miner start -p 1000`

help 参数 `/root/orz-cli-gpu/ore_miner --help`

建议使用 pm2 监控启动

### 配置ore-sh脚本

创建`start.sh`启动脚本脚本文件

```
vim start.sh

// 将下面代码粘贴进去(注意路径),然后保存退出。
/root/orz-cli-gpu/ore_miner start -p 1000 -c 50000 -o 1 -t 1

```

配置好后，执行`pm2 start start.sh --name orz`

执行`pm2 logs orz` 查看日志是否启动即可

更多 pm2 的命令自己查找pm2 文档吧

### 多路显卡启动脚本（自行研究）

```
//每条命令一个 sh  用pm2 分别启动
export CUDA_VISIBLE_DEVICES=0
/root/ore-cli-master/ore_miner start -p 1001 -c 5000000 -o 0.1 -d /root/workspace/ore/1 -t 1 -u /root/workspace/ore/1/rpc.txt

export CUDA_VISIBLE_DEVICES=1
/root/ore-cli-master/ore_miner start -p 1001 -c 5000000 -o 0.1 -d /root/workspace/ore/2 -t 1 -u /root/workspace/ore/2/rpc.txt

export CUDA_VISIBLE_DEVICES=2
/root/ore-cli-master/ore_miner start -p 1001 -c 5000000 -o 0.1 -d /root/workspace/ore/3 -t 1 -u /root/workspace/ore/3/rpc.txt

export CUDA_VISIBLE_DEVICES=3
/root/ore-cli-master/ore_miner start -p 1001 -c 5000000 -o 0.1 -d /root/workspace/ore/4 -t 1 -u /root/workspace/ore/4/rpc.txt

```
### 查询产量

给脚本执行权限 `chmod +x get.sh`

然后执行此命令查询产量`/root/orz-cli-gpu/get.sh`

### 归集到主号

给脚本执行权限 `chmod +x claim.sh`

修改`claim.sh`中的**claim_address**和**rpc_url**
注意里面的路径 要改成官方的版本变异的`orz`

然后执行此命令归集orz`/root/orz-cli-gpu/claim.sh`
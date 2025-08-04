# PNL

## 代码

语言：rust

框架：axum

`docker`部署：

```shell
docker-compose build
docker-compose up -d
```

本地部署：

```shell
cargo run
```

## 思路

1. 通过`user address` 和 `token mint` 来获取关联代币账户（`get_token_accounts_by_owner`），默认使用返回的第一个账户作为统计其Pnl的对象

2. 通过`get_transaction_signatures`来获取前25个（由于速率限制）已经完全确定的交易hash。

3. 通过分组多线程获取交易信息（`get_transaction_with_config`）

4. 对交易信息进行分析，获取存在于内部交易里面的`swapEvent`数据，对其进行解码。同时从`post_balance`来获取代币的基本信息

5. 将这些数据转为`SwapItem`进行表示，方便后续进行统计，这里的转换过程涉及到不同代币在不同时间的价格获取，支持三种基础代币：USDC/USDT/SOl，通过`pyth network`进行获取

6. 对所有`SwapItem`按照时间进行排序，并且进行最终的处理，获得该25个交易的全局pnl数据，在这处理逻辑里面，包含了对`token_mint`最新价格的获取（通过jupiter api）



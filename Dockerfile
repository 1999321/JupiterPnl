# 使用官方 Rust 最新镜像作为构建环境
FROM rust:latest as builder

# 创建工作目录
WORKDIR /app

# 复制项目文件并构建（假设是Rust项目，需提前生成Cargo.toml和src/）
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# 构建发布版本（优化二进制大小）
RUN cargo build --release

# 使用轻量级运行时镜像
FROM debian:bookworm-slim

# 安装必要的运行时依赖
RUN apt-get update && apt-get install -y \
    curl \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 从构建阶段复制二进制文件
WORKDIR /app
COPY --from=builder /app/target/release/jupiter .

# 暴露 80 端口
EXPOSE 80

# 启动服务（假设二进制名为 `jupiter`）
CMD ["./jupiter"]
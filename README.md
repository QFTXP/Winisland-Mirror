# WinIsland 2.0.1 非官方修复版

> **这不是官方仓库，也不是原项目的镜像。**
> 这里存放的是基于开源镜像源码自行编译的 WinIsland 2.0.1：修好了灵言（LingoSoul）中文输入，并恢复了 2.0.0 的液态玻璃白色高光边缘。原项目已停止维护，本仓库与原开发者无关。

## 下载

前往 [Releases](../../releases/latest) 下载。两个文件二选一：

| 文件 | 适合谁 | 说明 |
| --- | --- | --- |
| `WinIsland-2.0.1-imefix-Setup.exe` | 第一次安装 | 安装包，双击即可；自签名证书 |
| `WinIsland-2.0.1-imefix.zip` | 已经装过官方 2.0.1 | 免安装，覆盖 `WinIsland.exe` 即可 |

每个文件都在 Release 页附了 SHA256，下载后可以校验。

## 安装

**用安装包**

1. 先退出正在运行的 WinIsland（托盘退出，任务管理器确认没有 `WinIsland.exe`）；
2. 双击 `WinIsland-2.0.1-imefix-Setup.exe`；
3. 安装过程会弹 `certutil` 的 UAC（导入自签名发布者证书），点“是”。

安装包与官方版使用同一个 AppId，会直接替换已安装的版本，**不需要先手动卸载**。

**用 zip 覆盖**

1. 先用官方 2.0.1 安装包正常安装；
2. 退出 WinIsland，备份安装目录里的 `WinIsland.exe`；
3. 把 zip 里的 `WinIsland.exe` 和 4 个 `.dll` 复制进安装目录（`resources` 可选覆盖）；
4. 重新启动。

**可能遇到的问题**

- 提示“应用身份注册失败”：不影响本体使用。用管理员 PowerShell 执行安装目录下 `identity\Install-Identity.ps1` 修复即可。
- 提示“未知发布者”：本构建未使用官方证书签名，属正常现象，以 Release 页的 SHA256 为准。

系统要求：Windows 10 2004 (19041) 及以上 / Windows 11，x64。灵言语音功能首次使用需下载约 190 MB 模型；只打字对话可以不下载。

## 改了什么

- **灵言中文输入**：输入法（IME）与候选框改绑到真正接收键盘焦点的 input 窗口；插件刷新不再抢走输入框焦点，拼音不会打到一半被中断。
- **液态玻璃边缘**：`border_opacity = 0`，去掉 2.0.1 新增的黑色描边；高光强度、宽度、光源方向恢复为 2.0.0 的数值。
- 其他功能、网络请求地址、更新器均未改动。

改动明细、构建方式与日期见 `SOURCE-源码与改动.txt`（Release 附件内也有）。

## 注意事项

- **非官方**：这不是 Eatgrapes 发布的版本，与原开发者无关；请不要当作官方包传播。
- **签名**：安装包为自签名，主程序未签名，Windows 可能提示“未知发布者”。
- **联网行为**：语音识别在本机完成；对话内容会发送到你自行配置的 AI 接口，API 密钥保存在 `%USERPROFILE%\.winisland`。项目本身不收集遥测（详见上游的 PRIVACY.md）。
- **维护状态**：上游项目已停止维护，本仓库只做必要的兼容修复，不保证后续更新。

## 来源与致谢

- 原项目：Eatgrapes / [WinIsland](https://github.com/WinIslandProject/WinIsland)（GNU GPLv3）
- 开源镜像（本仓库的上游）：[Neamyoo-dev/Winisland-Mirror](https://github.com/Neamyoo-dev/Winisland-Mirror)
- 本仓库：fork 自上述镜像，附带中文输入与边缘修复的提交

## 构建

本仓库使用 GitHub Actions 从源码构建，不需要本地工具链：

- `Build patched WinIsland`（`.github/workflows/build-patched.yml`）—— 产出 `WinIsland.exe` 与运行库；
- `Build patched installer`（`.github/workflows/build-installer.yml`）—— 产出自签名安装包（证书由 runner 现场生成）。

两条流程均为手动触发：Actions → 选择工作流 → Run workflow。

## 许可证

WinIsland 以 **GNU GPLv3** 发布，本仓库的修改同样遵循 GPLv3。修改内容与日期见 `SOURCE-源码与改动.txt`，许可证全文见 `LICENSE`。

## 免责声明

非官方构建，按“现状”提供；使用风险自负，作者不对任何后果负责。

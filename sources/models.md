# 模型时间线与成绩来源

核验日期：**2026-10-24**。此文件对应 `src/models.rs` 的 20 个精选条目，并非所有模型的完整清单。发布记录最新至 **2026-10-17**。共收录 15 个成绩：12 个历史 MMLU 成绩，来源日期最新至 **2025-01-22**；另有 3 个 **2026-10-24 查阅的 AdvancedIF 快照成绩**。动画中的领先者只表示**当前时间、当前赛道、已收录模型的公布成绩领先者**，不表示全部实际能力或当前全球综合排名。

## 三条赛道

| 指标 | 代码中的协议字符串 | 收录范围 |
| --- | --- | --- |
| MMLU | `5-shot / no-CoT` | 8 个公布结果；提供 5 个示例，不要求生成思维链 |
| MMLU | `0-shot CoT / simple-evals` | 4 个公布结果；采用 OpenAI simple-evals 的零示例推理提示框架 |
| AdvancedIF | `Mercor / single-shot / highest effort` | 3 个 Mercor 独立评估快照；单次作答代理，各模型最高推理档 |

排序必须同时匹配 **benchmark 和 protocol**，不能只按指标名称匹配。MMLU 是多学科选择题测试，AdvancedIF 测量复杂指令遵循；它们都不是智商、人格或全部实际能力的度量。分数按百分数保存，越高表示在该测试及评分方法下通过的比例越高。两种测试的百分数不能直接比较。

历史 MMLU 数值来自研究团队、开发商的论文及模型卡，**没有在本项目中统一独立重测**。第一条赛道的示例选择、答案提取、平均方式和实现仍可能不同；早期 MMLU 论文与后来模型卡也不保证采用相同聚合方法。共同协议标签是比较的必要条件，不能消除这些差异。

第二条赛道的 GPT-4o、Qwen2.5、DeepSeek-V3 来自同一份 DeepSeek-V3 报告表 6，评估者为 DeepSeek；它们不都是各自开发商的自报结果。R1 报告延续 simple-evals 提示框架，但推理计算预算与 V3 评估不同，因此也不是相同算力条件下的对照实验。V3 报告说明输出上限为 8K，R1 报告说明为 32,768 tokens。参见 [V3 评估方法与表 6](https://arxiv.org/html/2412.19437v1#S5) 和 [R1 评估方法](https://arxiv.org/html/2501.12948v1#S3)。

不收录与上述协议不符的 MMLU-Pro、多次采样选优、不同提示或不同模型变体成绩。典型区别：Llama 3.1 405B Instruct 的 **87.3 是 5-shot**，**88.6 是 0-shot CoT**；Gemini Ultra 的 90.04 使用 CoT@32，不能放入普通 5-shot 赛道。[Meta 模型卡](https://huggingface.co/meta-llama/Llama-3.1-405B-Instruct)、[Gemini 报告表 2](https://arxiv.org/html/2312.11805v1#S5.T2)

## AdvancedIF：截至查阅日的独立快照

Mercor 对 Meta 的 AdvancedIF 进行独立测试。其公开方法为 1,645 个文本任务、每题 4 次运行、单次作答代理（single-shot agent），用 Gemini 3.5 Flash 按专家编写的评分标准判卷，汇总为 Pass@1，而不是从 4 次中挑选最好的一次。这里的“独立”指评估者 Mercor 与这三家模型开发商不同，并不表示本项目复测，也不保证评估没有偏差。[Mercor 官方方法](https://www.mercor.com/apex/methodology/)

| 模型 | 原始发布日期 | 查阅快照日期 | AdvancedIF Pass@1 | 来源中显示的推理档 |
| --- | --- | --- | ---: | --- |
| Gemini 3.1 Pro Preview | 2026-02-19 | 2026-10-24 | 86.7% | High |
| Claude Opus 5.5 | 2026-09-22 | 2026-10-24 | 82.8% | Max |
| GPT-6.1 Sol | 2026-09-29 | 2026-10-24 | 82.4% | Max |

Gemini 的数值可见于 [AdvancedIF 官方榜](https://www.mercor.com/apex/oss-benchmarks/oss-advanced-if-leaderboard/)；全部三项都列于官方[完整模型选择表](https://www.mercor.com/apex/oss-benchmarks/oss-advanced-if-leaderboard/sample-task/)顶部的榜单区域，下面的示例任务内容不是这些数值的来源。

Mercor 说明测试采用各模型的最高推理档；因此 Gemini 的 High 与另外两款的 Max 在本赛道中属于“各自最高档”，**不等于相同 token 预算、时间或计算量**。同一评估流程也不能让单项指令遵循成绩代表总体能力；相邻分数不保证有统计显著差异。[Mercor 官方方法](https://www.mercor.com/apex/methodology/)

该榜没有给出每项结果的发表日期。本项目只知道 **2026-10-24 查阅时已经存在这些成绩**：三个条目的 `score_observed = true`，`reported = "2026-10-24"`。展示文字必须是“查阅快照 / observed snapshot”，不能称为“2026-10-24 首次公布”，也不能把数值提前套用到模型刚发布时。历史播放只有到达该快照日期后才能显示第三赛道。

该来源中的 DeepSeek 条目叫 `DeepSeek-V4.1-Flash`，未注明 `1017` 更新；不能把该成绩借给本项目的 `DeepSeek-V4.1-Flash-1017`，后者继续不排名。

## 时间含义

- `released` 表示所列具体版本第一次公开发布或开始提供有限访问的日期。它不等于论文提交日，也不保证当日所有用户均已获得访问。
- `score_observed = false` 时，`reported` 是选用的历史成绩来源版本日期；`score_observed = true` 时，`reported` 是本项目查阅榜单的日期，不是已知发表日期。模型可以先出现，但只有播放日期达到 `reported` 且 `score` 非空后，才进入对应赛道。
- 对没有成绩的条目，`reported = released` 只是时间字段占位；`score = None`、`benchmark = "none"`、`protocol = "none"`、`score_observed = false` 才是决定条件。不得把这些模型当作零分，也不得把占位日期当作成绩发布日期。
- `year` 按公历日期计算年内比例，包含闰年；排序和日期显示仍应优先采用完整 ISO 日期。
- 最新模型若没有已核实的上述协议结果，保留登场和介绍，不推测它的分数或能力排名。

## 历史 MMLU 成绩

以下日期均为 `YYYY-MM-DD`；“成绩日期”指本项目选用的来源日期，不声称是该成绩在互联网上最早出现的日期。

| 模型与版本 | 发布日期 | 成绩日期 | MMLU | 协议 | 成绩来源 |
| --- | --- | --- | ---: | --- | --- |
| GPT-3 175B | 2020-06-11 | 2020-09-07 | 43.9 | 5-shot / no-CoT | [MMLU 论文 v1，表 1](https://arxiv.org/pdf/2009.03300v1) |
| GPT-4，2023 年 3 月 | 2023-03-14 | 2023-03-15 | 86.4 | 5-shot / no-CoT | [GPT-4 技术报告 v1，表 2](https://arxiv.org/pdf/2303.08774v1) |
| Llama 2 70B base | 2023-07-18 | 2023-07-18 | 68.9 | 5-shot / no-CoT | [Meta 模型卡](https://huggingface.co/meta-llama/Llama-2-70b-hf)；[论文表 4](https://arxiv.org/html/2307.09288v2#S3.T4) |
| Gemini 1.0 Pro | 2023-12-06 | 2023-12-19 | 71.8 | 5-shot / no-CoT | [Gemini 报告 v1，表 2](https://arxiv.org/html/2312.11805v1#S5.T2) |
| Claude 3 Opus | 2024-03-04 | 2024-06-21 | 86.8 | 5-shot / no-CoT | [Claude 3 模型卡增补，表 1](https://www-cdn.anthropic.com/fed9cc193a14b84131812372d8d5857f8f304c52/Model_Card_Claude_3_Addendum.pdf) |
| Llama 3 70B Instruct | 2024-04-18 | 2024-04-18 | 82.0 | 5-shot / no-CoT | [Meta 模型卡，指令版表格](https://huggingface.co/meta-llama/Meta-Llama-3-70B-Instruct) |
| GPT-4o 2024-05-13 | 2024-05-13 | 2024-12-27 | 87.2 | 0-shot CoT / simple-evals | [DeepSeek-V3 报告 v1，表 6](https://arxiv.org/html/2412.19437v1#S5.T6) |
| Claude 3.5 Sonnet，2024 年 6 月 | 2024-06-20 | 2024-06-21 | 88.7 | 5-shot / no-CoT | [Claude 3 模型卡增补，表 1](https://www-cdn.anthropic.com/fed9cc193a14b84131812372d8d5857f8f304c52/Model_Card_Claude_3_Addendum.pdf) |
| Llama 3.1 405B Instruct | 2024-07-23 | 2024-07-23 | 87.3 | 5-shot / no-CoT | [Meta 模型卡，指令版表格](https://huggingface.co/meta-llama/Llama-3.1-405B-Instruct) |
| Qwen2.5 72B Instruct | 2024-09-19 | 2024-12-27 | 85.3 | 0-shot CoT / simple-evals | [DeepSeek-V3 报告 v1，表 6](https://arxiv.org/html/2412.19437v1#S5.T6) |
| DeepSeek-V3，1226 | 2024-12-26 | 2024-12-27 | 88.5 | 0-shot CoT / simple-evals | [DeepSeek-V3 报告 v1，表 6](https://arxiv.org/html/2412.19437v1#S5.T6) |
| DeepSeek-R1，0120 | 2025-01-20 | 2025-01-22 | 90.8 | 0-shot CoT / simple-evals | [DeepSeek-R1 报告 v1，表 4](https://arxiv.org/html/2501.12948v1#S3.T4) |

Claude 3.5 Sonnet 的官方 API 更新记录明确给出 **2024-06-20**，模型标识也是 `claude-3-5-sonnet-20240620`。目前官网介绍页显示 **2024-06-21**，因此发布采用 API 记录，所选增补模型卡中的两项成绩保守地在 6 月 21 日显示；并未把后来核验的 Opus 86.8 倒填回 3 月。参见 [API 发布记录](https://platform.claude.com/docs/en/release-notes/overview#june-20th-2024) 和 [Sonnet 公告](https://www.anthropic.com/news/claude-3-5-sonnet)。

## 发布记录与未排名条目

| 条目 | 日期 | 第一手来源与版本边界 |
| --- | --- | --- |
| BERT | 2018-11-02 | [Google 开源公告](https://research.google/blog/open-sourcing-bert-state-of-the-art-pre-training-for-natural-language-processing/)；采用权重公开日期，不采用 10 月论文日期。无收录成绩。 |
| GPT-2 初次发布 | 2019-02-14 | [OpenAI 公告](https://openai.com/index/better-language-models/)；当次仅公开较小版本，不能把它写成完整 1.5B 权重发布。无收录成绩。 |
| GPT-3 175B | 2020-06-11 | [OpenAI API 公告](https://openai.com/index/openai-api/)；采用有限 API 访问日期，不采用 5 月论文日期。 |
| ChatGPT / GPT-3.5 首发 | 2022-11-30 | [OpenAI 公告](https://openai.com/index/chatgpt/)；没有把后来泛称 GPT-3.5 的测试数值归给这个首发快照。 |
| GPT-4 | 2023-03-14 | [OpenAI 公告](https://openai.com/index/gpt-4/) |
| Llama 2 | 2023-07-18 | [Meta 公告](https://about.fb.com/news/2023/07/llama-2/)；成绩选用 70B base，不是 Chat。 |
| Gemini 1.0 Pro | 2023-12-06 | [Google Bard 更新公告](https://blog.google/products-and-platforms/products/gemini/google-bard-try-gemini-ai/)；Pro 并非 Ultra。 |
| Claude 3 Opus | 2024-03-04 | [Anthropic 公告](https://www.anthropic.com/news/claude-3-family) |
| Llama 3 70B Instruct | 2024-04-18 | [Meta 公告](https://ai.meta.com/blog/meta-llama-3/) |
| GPT-4o | 2024-05-13 | [OpenAI 公告](https://openai.com/index/hello-gpt-4o/)；分数严格对应 0513 快照。 |
| Claude 3.5 Sonnet | 2024-06-20 | [Anthropic API 记录](https://platform.claude.com/docs/en/release-notes/overview#june-20th-2024)；不是 2024 年 10 月更新版。 |
| Llama 3.1 405B Instruct | 2024-07-23 | [Meta 公告](https://ai.meta.com/blog/meta-llama-3-1/) |
| Qwen2.5 72B Instruct | 2024-09-19 | [Qwen 团队公告](https://qwenlm.github.io/blog/qwen2.5/) |
| DeepSeek-V3 | 2024-12-26 | [DeepSeek 公告](https://api-docs.deepseek.com/news/news1226/)；不是后来的 0324 更新。 |
| DeepSeek-R1 | 2025-01-20 | [DeepSeek 公告](https://api-docs.deepseek.com/news/news250120/)；不是后来的 0528 更新。 |
| GPT-5 | 2025-08-07 | [OpenAI 公告](https://openai.com/index/introducing-gpt-5/)；无收录成绩。 |
| Gemini 3.1 Pro Preview | 2026-02-19 | [Google DeepMind 模型卡](https://deepmind.google/models/model-cards/gemini-3-1-pro/)；AdvancedIF 仅作为 2026-10-24 查阅快照。 |
| Claude Opus 5.5 | 2026-09-22 | [Anthropic 公告](https://www.anthropic.com/claude-opus-5-5)及[官方更新日志](https://support.claude.com/en/articles/12138966-release-notes)；AdvancedIF 仅作为 2026-10-24 查阅快照。 |
| GPT-6.1 Sol | 2026-09-29 | [OpenAI DevDay 公告](https://openai.com/index/devday-2026-recap/)及[模型介绍](https://openai.com/index/introducing-gpt-6-1-sol/)；AdvancedIF 仅作为 2026-10-24 查阅快照。 |
| DeepSeek-V4.1-Flash-1017 | 2026-10-17 | [DeepSeek 公告](https://api-docs.deepseek.com/news/news261017/)；无收录成绩。 |

模型、论文、商标及外部网站分别遵循其原有许可与条款；本仓库的 MIT 许可证不改变外部来源的授权。本项目仅摘录事实性日期、测试数值并撰写简短介绍，不复制论文图表或模型权重。

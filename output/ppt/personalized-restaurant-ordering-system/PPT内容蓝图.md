# PPT内容蓝图

## 场景定稿

- 材料状态：有完整文档、系统实现、数据表与证据截图
- 先补策略：以毕业设计报告为事实主干，以仓库证据复核关键数字；首轮只展开第 1 与第 11 页
- 学科或业务场景：Rust Web system、explainable recommendation、single-restaurant ordering
- 具体主题：Personalized Restaurant Ordering System
- 汇报类型：Final Year Project degree presentation + live demo
- 听众：2 位 FYP supervisors
- 证据密度：学术高证据
- 一句话主线：轻量、可解释的混合推荐可以嵌入手机点餐闭环，并在受控测试中对偏好与共点餐证据作出可观察响应。
- 锁定主题：Personalized Restaurant Ordering System
- 禁止泛化方向：大型外卖平台、生产级商业部署、深度学习推荐、无法由报告支持的经营收益
- 不可改写关键词：ingredient-based filtering、co-order collaborative filtering、hybrid scoring、explainability、mobile-first、single restaurant

## 材料边界

- 来自用户材料：项目目标、系统架构、推荐算法、测试数字、实验结果、系统截图、CSV 数据。
- 需要核验引用：问题背景、推荐系统与可解释性文献，批量生成时从报告参考文献表核验。
- 需要计算或整理：Hit@3、平均隐藏菜品排名、排名变化案例。
- 模拟假设：价格和部分订单状态用于原型演示；不作为经营结论。
- 仍然缺失：真实餐厅用户研究、长期线上 A/B 测试、统计显著性检验。

## 页序定稿

| 页码 | 页面标题 | 页面角色 | 听众问题 | 一句话结论 | 内容关系简报 | 推荐视觉 | 来源状态 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 01 | Personalized Restaurant Ordering System | 开场定位 | 项目是什么？ | 一个面向小型餐厅的手机优先、轻量且可解释的个性化点餐原型。 | 项目身份 + 使用场景 + 技术定位 | 产品系统封面 | 用户材料 |
| 02 | Problem and Research Gap | 问题论证 | 为什么需要它？ | 小型餐厅需要低成本入口、降低选择负担并提高推荐透明度。 | 问题—缺口—设计约束 | 研究问题图 | 用户材料 + 待核验引用 |
| 03 | Research Questions and Objectives | 研究框架 | 要回答什么？ | 三项研究问题分别验证偏好、共点餐与方法比较。 | RQ—objective 对应关系 | 研究问题矩阵 | 用户材料 |
| 04 | Proposed System and User Journey | 场景说明 | 顾客与员工如何使用？ | QR 点餐、推荐、结账和后台处理形成行为数据闭环。 | 双角色流程 | 用户旅程 | 用户材料 |
| 05 | Lightweight Web Architecture | 架构论证 | 系统如何保持轻量和模块化？ | Rust/Axum 模块把界面、算法、订单和数据职责分开。 | 层级与依赖 | 架构图 | 用户材料 |
| 06 | Recommendation Processing Pipeline | 方法解释 | 推荐如何产生？ | 硬过滤后融合内容、共点餐及辅助信号，并进行重排。 | 处理流程 | 数据分析管线 | 用户材料 |
| 07 | Explainability and Evidence Confidence | 可信解释 | 用户为何应相信结果？ | 每条推荐都绑定具体偏好与共点餐证据，并显示证据强度。 | 分数—证据—解释 | 证据解释页 | 用户材料 |
| 08 | Implemented System Outcomes | 系统成果 | 最终实现了什么？ | 客户点餐和管理端关键流程已形成可演示闭环。 | 功能分组与业务闭环 | 系统成果板 | 用户材料 |
| 09 | System Testing Results | 工程验证 | 系统是否按预期工作？ | 115 项 Rust 测试与 55 项报告检查全部通过。 | 覆盖范围—结果—边界 | 测试看板 | 用户材料 |
| 10 | RQ1 and RQ2: Preference and Co-Order Impact | 实验结果 | 排名会随输入改变吗？ | 喜欢、不喜欢与新增共点餐证据产生可解释的排序变化。 | 前后对比案例 | 排名变化对照 | 用户材料 + 计算 |
| 11 | RQ3: Recommendation Method Comparison | 核心比较 | 哪种方法最能恢复隐藏菜品？ | 在 5 个受控案例中，co-order-only 与 fixed hybrid 的 Hit@3 均为 100%，ingredient-only 为 20%。 | 方法—指标—解释—限制 | 数据图 + 案例矩阵 | 用户材料 + 计算 |
| 12 | Contributions, Limitations and Future Work | 边界判断 | 能贡献什么，不能证明什么？ | 原型证明技术闭环可行，但真实有效性仍需更大数据与用户研究。 | 贡献—限制—下一步 | 证据边界页 | 用户材料 |
| 13 | Conclusion and Live Demo | 收束行动 | 最终结论和演示重点是什么？ | 系统把偏好、行为与解释整合进可运行的手机点餐流程。 | 结论—演示路径 | 收束页 | 用户材料 |

## 重点页展开

### 第 01 页：Personalized Restaurant Ordering System

- 为什么重要：第一分钟必须让评审明确系统对象、研究价值和实现形态。
- 听众问题：这是什么系统，它服务谁，核心技术是什么？
- 一句话结论：A lightweight, explainable, mobile-first ordering prototype for small restaurants.
- 内容关系简报：项目身份是主信息；移动端 QR 点餐场景说明“为什么是 Web”；三个推荐信号说明研究技术边界；作者与导师信息承担学术归属。
- 核心内容关系：系统定位与使用场景。
- 信息优先级：项目名称 > 一句话定位 > mobile-first experience + explainable hybrid recommendations > 作者信息。
- 锁定主题：Personalized Restaurant Ordering System。
- 不可改写内容：YEAP CHAN LEONG；22049837；Supervisor: Professor Serge Demidenko；Rust Web Prototype。
- 内容块及功能：
  - 定义：Personalized Restaurant Ordering System。
  - 解释：Lightweight, explainable recommendations for small restaurants。
  - 机制提示：Ingredient preferences + co-order patterns + hybrid scoring。
  - 场景：Scan QR → browse → recommendations → order。
  - 来源：Final Year Project report, 2026。
- 必须出现的变量、证据或案例：不放实验数字；必须出现移动端点餐产品证据。
- 推荐图形：由真实手机菜单界面与简洁推荐信号构成的系统封面主视觉。
- 页脚来源或假设：Final Year Project · 2026。

### 第 11 页：RQ3 — Recommendation Method Comparison

- 为什么重要：这是回答“哪种方法效果更好”的核心证据页，也是评审最可能质疑实验设计的页面。
- 听众问题：三种方法在隐藏菜品恢复任务中表现如何，结论边界是什么？
- 一句话结论：Co-order-only and fixed hybrid recovered the hidden dish in the Top 3 for all five controlled cases; ingredient-only succeeded once.
- 内容关系简报：以 Hit@3 为主证据，以平均隐藏排名为第二证据；用五案例结果矩阵说明结论不是单一案例；用实验限制框定解释范围。
- 核心内容关系：三方法定量对比 + 受控实验边界。
- 信息优先级：Hit@3 20/100/100 > average rank 14.20/1.80/1.80 > 5-case evidence > fixed-profile limitation。
- 锁定主题：RQ3 method comparison。
- 不可改写内容：Ingredient-only 20%；Co-order-only 100%；Fixed hybrid (0.4/0.6) 100%；Average hidden rank 14.20 / 1.80 / 1.80；5 controlled cases；liked preferences rice + chicken；historical baseline retained。
- 内容块及功能：
  - 证据：Hit@3 柱状比较。
  - 证据：平均隐藏菜品排名。
  - 解释：co-order signal dominates these cases because historical pair evidence is strong。
  - 限制：small controlled sample, fixed preference profile, no statistical generalisation。
  - 来源：Section 4.4 method-comparison evidence。
- 必须出现的变量、证据或案例：三种方法名称、Hit@3 数值、平均排名、5-case 边界。
- 推荐图形：大尺度数据图 + 紧凑证据矩阵/注释，结论与限制同屏。
- 页脚来源或假设：Source: FYP report Section 4.4; 5 controlled hidden-dish cases.

## 质量自检

- 主题具体且可讲：是。
- 每页都有页面任务：是。
- 每个内容页都有内容关系简报：是。
- 样张重点页已写出具体内容块：是。
- 所有关键数字均有来源状态：是。
- 避免空洞大字报：是。
- 锁定主题、标题、结论和关键变量：是。
- 未泛化为大型商业推荐平台：是。

## 剩余页面内容块

### 第 02 页：Problem and Research Gap
- 内容块：单餐厅数据稀疏、顾客冷启动、硬限制不能被覆盖、推荐缺乏透明度。
- 结论责任：说明为什么需要轻量、确定性、可解释的混合方法，不声称现有系统一定提升满意度或销售。
- 来源：报告 Sections 1.1–1.2；文献引用沿用报告。

### 第 03 页：Research Questions and Objectives
- 内容块：RQ1 偏好与限制、RQ2 共点餐证据、RQ3 方法比较；RO1–RO6 汇总为 Build、Recommend、Evaluate 三组目标。
- 结论责任：建立研究问题与受控实验的一一对应。
- 来源：报告 Sections 1.3–1.4。

### 第 04 页：Proposed System and User Journey
- 内容块：QR/注册、浏览与偏好、推荐解释、购物车结账、后台处理、完成订单回流历史证据。
- 结论责任：说明系统同时是点餐产品和推荐评估载体。
- 来源：报告 scope、workflow 与系统实现。

### 第 05 页：Lightweight Web Architecture
- 内容块：Customer/Admin browsers、Axum routes/handlers/state、recommendation/search/order/persistence modules、CSV/JSONL/local images。
- 结论责任：说明低耦合、高内聚与无外部运行时推荐服务。
- 来源：报告 Section 3.4 与 `docs/ARCHITECTURE.md`。

### 第 06 页：Recommendation Processing Pipeline
- 内容块：candidate eligibility、hard exclusions、content/co-order/popularity/time signals、adaptive weighting、diversity reranking、explanations/evidence confidence。
- 结论责任：区分硬约束、排序分数和证据置信度。
- 来源：报告 recommendation design and implementation。

### 第 07 页：Explainability and Evidence Confidence
- 内容块：匹配喜欢食材、偏好标签、共点餐影响、硬排除、component scores、evidence confidence。
- 结论责任：解释是由可检查证据构建；置信度不等于购买概率。
- 来源：报告 Sections 1.6、3.4、5.3。

### 第 08 页：Implemented System Outcomes
- 内容块：客户移动菜单与搜索、偏好与推荐、购物车与订单跟踪、后台订单与菜品管理、实验测试器。
- 结论责任：展示已实现闭环，不扩展为生产级平台。
- 来源：报告 Chapter 4 与仓库截图。

### 第 09 页：System Testing Results
- 内容块：115/115 Rust tests、55/55 report checks、12 unit、8 integration、3 end-to-end、8 security、24 responsive；测试边界。
- 结论责任：证明受测功能正确，同时明确不等于生产就绪或用户满意。
- 来源：`SYSTEM_TESTING_RESULTS.md` 与报告 Section 4.3。

### 第 10 页：RQ1 and RQ2 — Observable Ranking Impact
- 内容块：D14 10→1、D30 1→excluded、D05 2→1；D07 7→1 after 3 co-orders；D09 保持 1 但关联增强。
- 结论责任：偏好和共点餐证据按设计改变资格与排序，排名变化取决于原始位置。
- 来源：报告 Sections 4.4.1–4.4.2 与 evidence CSV。

### 第 12 页：Contributions, Limitations and Future Work
- 内容块：轻量整合、可解释混合推荐、受控非破坏实验；小数据、单餐厅、无大规模用户研究；数据库、真实用户评估、在线实验与安全强化。
- 结论责任：清楚区分原型可行性与商业有效性。
- 来源：报告 Sections 1.5、5.3–5.4。

### 第 13 页：Conclusion and Live Demo
- 内容块：移动点餐、可解释推荐、订单回流；演示四步：customer entry、preferences/reasons、checkout/status、admin history/tester。
- 结论责任：用可运行流程收束研究贡献并进入 demo。
- 来源：用户材料与系统实现。

# 任务调度器

## 简介

任务调度器用于对要执行的任务进行调度，任务调度器的主要功能包括：

- 检查任务间的依赖关系，确保依赖关系满足后才能执行任务。
- 对任务进行拓扑排序，确保构建任务能够按照正确的顺序执行。
- 当具有相同依赖关系的任务同时被提交时，只执行一次任务。
- 当任务存在环形依赖关系时，为用户提供友好的错误提示：找到环形依赖关系并打印出来，以便用户进行修复。
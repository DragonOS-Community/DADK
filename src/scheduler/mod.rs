use std::{
    collections::BTreeMap,
    fmt::Debug,
    path::PathBuf,
    rc::Rc,
    sync::atomic::{AtomicI32, Ordering}, process::exit,
};

use log::{error, info};

use crate::{console::Action, executor::Executor, parser::task::DADKTask};

/// # 调度实体
#[derive(Debug)]
pub struct SchedEntity {
    /// 任务ID
    id: i32,
    file_path: PathBuf,
    /// 任务
    task: DADKTask,
}

impl SchedEntity {
    #[allow(dead_code)]
    pub fn id(&self) -> i32 {
        self.id
    }

    #[allow(dead_code)]
    pub fn file_path(&self) -> &PathBuf {
        &self.file_path
    }

    #[allow(dead_code)]
    pub fn task(&self) -> &DADKTask {
        &self.task
    }

    #[allow(dead_code)]
    pub fn task_mut(&mut self) -> &mut DADKTask {
        &mut self.task
    }
}

/// # 调度实体列表
///
/// 用于存储所有的调度实体
#[derive(Debug)]
pub struct SchedEntities {
    /// 调度实体列表
    entities: Vec<Rc<SchedEntity>>,
    /// 任务ID到调度实体的映射
    id2entity: BTreeMap<i32, Rc<SchedEntity>>,
    /// 任务名和版本到调度实体的映射
    name_version_2_entity: BTreeMap<(String, String), Rc<SchedEntity>>,
}

impl SchedEntities {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            id2entity: BTreeMap::new(),
            name_version_2_entity: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, entity: Rc<SchedEntity>) {
        self.entities.push(entity.clone());
        self.id2entity.insert(entity.id, entity.clone());
        self.name_version_2_entity.insert(
            (entity.task.name.clone(), entity.task.version.clone()),
            entity,
        );
    }

    #[allow(dead_code)]
    pub fn get(&self, id: i32) -> Option<Rc<SchedEntity>> {
        self.id2entity.get(&id).cloned()
    }

    pub fn get_by_name_version(&self, name: &str, version: &str) -> Option<Rc<SchedEntity>> {
        self.name_version_2_entity
            .get(&(name.to_string(), version.to_string()))
            .cloned()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Rc<SchedEntity>> {
        self.entities.iter()
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.entities.clear();
        self.id2entity.clear();
        self.name_version_2_entity.clear();
    }

    pub fn topo_sort(&self) -> Vec<Rc<SchedEntity>> {
        let mut result = Vec::new();
        let mut visited = BTreeMap::new();
        for entity in self.entities.iter() {
            if !visited.contains_key(&entity.id) {
                let r = self.dfs(entity, &mut visited, &mut result);
                if r.is_err() {
                    let err = r.unwrap_err();
                    error!("{}", err.display());
                    println!("Please fix the errors above and try again.");
                    std::process::exit(1);
                }
            }
        }
        return result;
    }

    fn dfs(
        &self,
        entity: &Rc<SchedEntity>,
        visited: &mut BTreeMap<i32, bool>,
        result: &mut Vec<Rc<SchedEntity>>,
    ) -> Result<(), DependencyCycleError> {
        visited.insert(entity.id, false);
        for dep in entity.task.depends.iter() {
            if let Some(dep_entity) = self.get_by_name_version(&dep.name, &dep.version) {
                if let Some(&false) = visited.get(&dep_entity.id) {
                    // 输出完整环形依赖
                    let mut err = DependencyCycleError::new();

                    err.add(
                        entity.file_path.clone(),
                        format!(
                            "{} ({})",
                            dep_entity.task.name_version(),
                            dep_entity.file_path.display()
                        ),
                    );
                    return Err(err);
                }
                if !visited.contains_key(&dep_entity.id) {
                    let r = self.dfs(&dep_entity, visited, result);
                    if r.is_err() {
                        let mut err = r.unwrap_err();
                        err.add(
                            entity.file_path.clone(),
                            format!(
                                "{} ({})",
                                dep_entity.task.name_version(),
                                dep_entity.file_path.display()
                            ),
                        );
                        return Err(err);
                    }
                }
            } else {
                error!(
                    "Dependency not found: {} -> {}",
                    entity.task.name_version(),
                    dep.name_version()
                );
                std::process::exit(1);
            }
        }
        visited.insert(entity.id, true);
        result.push(entity.clone());
        return Ok(());
    }
}

/// # 任务调度器
#[derive(Debug)]
pub struct Scheduler {
    /// DragonOS sysroot在主机上的路径
    dragonos_dir: PathBuf,
    /// 要执行的操作
    action: Action,
    /// 调度实体列表
    target: SchedEntities,
}

pub enum SchedulerError {
    TaskError(String),
    DependencyNotFound(Rc<SchedEntity>, String),
    RunError(String),
}

impl Debug for SchedulerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TaskError(arg0) => {
                write!(f, "TaskError: {}", arg0)
            }
            SchedulerError::DependencyNotFound(current, msg) => {
                write!(
                    f,
                    "For task {}, dependency not found: {}. Please check file: {}",
                    current.task.name_version(),
                    msg,
                    current.file_path.display()
                )
            }
            SchedulerError::RunError(msg) => {
                write!(f, "RunError: {}", msg)
            }
        }
    }
}

impl Scheduler {
    pub fn new(
        dragonos_dir: PathBuf,
        action: Action,
        tasks: Vec<(PathBuf, DADKTask)>,
    ) -> Result<Self, SchedulerError> {
        let entities = SchedEntities::new();

        let mut scheduler = Scheduler {
            dragonos_dir,
            action,
            target: entities,
        };

        let r = scheduler.add_tasks(tasks);
        if r.is_err() {
            error!("Error while adding tasks: {:?}", r);
            return Err(r.err().unwrap());
        }

        return Ok(scheduler);
    }

    /// # 添加多个任务
    ///
    /// 添加任务到调度器中，如果任务已经存在，则返回错误
    pub fn add_tasks(&mut self, tasks: Vec<(PathBuf, DADKTask)>) -> Result<(), SchedulerError> {
        for task in tasks {
            self.add_task(task.0, task.1)?;
        }

        return Ok(());
    }

    /// # 添加一个任务
    ///
    /// 添加任务到调度器中，如果任务已经存在，则返回错误
    pub fn add_task(&mut self, path: PathBuf, task: DADKTask) -> Result<(), SchedulerError> {
        let id: i32 = self.generate_task_id();
        let entity = Rc::new(SchedEntity {
            id,
            task,
            file_path: path.clone(),
        });
        let name_version = (entity.task.name.clone(), entity.task.version.clone());

        if self
            .target
            .get_by_name_version(&name_version.0, &name_version.1)
            .is_some()
        {
            return Err(SchedulerError::TaskError(format!(
                "Task with name [{}] and version [{}] already exists. Config file: {}",
                name_version.0,
                name_version.1,
                path.display()
            )));
        }

        self.target.add(entity.clone());

        info!("Task added: {}", entity.task.name_version());
        return Ok(());
    }

    fn generate_task_id(&self) -> i32 {
        static TASK_ID: AtomicI32 = AtomicI32::new(0);
        return TASK_ID.fetch_add(1, Ordering::SeqCst);
    }

    /// # 执行调度器中的所有任务
    pub fn run(&self) -> Result<(), SchedulerError> {
        // 检查是否有不存在的依赖
        let r = self.check_not_exists_dependency();
        if r.is_err() {
            error!("Error while checking tasks: {:?}", r);
            return r;
        }

        // 对调度实体进行拓扑排序
        let r: Vec<Rc<SchedEntity>> = self.target.topo_sort();

        crate::executor::prepare_env(&self.target)
            .map_err(|e| SchedulerError::RunError(format!("{:?}", e)))?;

        for entity in r.iter() {
            let executor = Executor::new(entity.clone()).map_err(|e| {
                error!(
                    "Error while creating executor for task {} : {:?}",
                    entity.task().name_version(),
                    e
                );
                exit(-1);
            }).unwrap();

            executor.execute().map_err(|e| {
                error!(
                    "Error while executing task {} : {:?}",
                    entity.task().name_version(),
                    e
                );
                exit(-1);
            }).unwrap();
        }
        return Ok(());
    }

    /// # 检查是否有不存在的依赖
    ///
    /// 如果某个任务的dependency中的任务不存在，则返回错误
    fn check_not_exists_dependency(&self) -> Result<(), SchedulerError> {
        for entity in self.target.iter() {
            for dependency in entity.task.depends.iter() {
                let name_version = (dependency.name.clone(), dependency.version.clone());
                if !self
                    .target
                    .get_by_name_version(&name_version.0, &name_version.1)
                    .is_some()
                {
                    return Err(SchedulerError::DependencyNotFound(
                        entity.clone(),
                        format!("name:{}, version:{}", name_version.0, name_version.1,),
                    ));
                }
            }
        }

        return Ok(());
    }
}

/// # 环形依赖错误路径
///
/// 本结构体用于在回溯过程中记录环形依赖的路径。
///
/// 例如，假设有如下依赖关系：
///
/// ```text
/// A -> B -> C -> D -> A
/// ```
///
/// 则在DFS回溯过程中，会依次记录如下路径：
///
/// ```text
/// D -> A
/// C -> D
/// B -> C
/// A -> B
pub struct DependencyCycleError {
    dependencies: Vec<(PathBuf, String)>,
}

impl DependencyCycleError {
    pub fn new() -> Self {
        Self {
            dependencies: Vec::new(),
        }
    }

    pub fn add(&mut self, path: PathBuf, dependency: String) {
        self.dependencies.push((path, dependency));
    }

    #[allow(dead_code)]
    pub fn dependencies(&self) -> &Vec<(PathBuf, String)> {
        &self.dependencies
    }

    pub fn display(&self) -> String {
        let mut ret = format!("Dependency cycle detected: \nStart ->\n");
        for entity in self.dependencies.iter() {
            ret.push_str(&format!(
                "->\t{}\t--depends-->\t{}\n",
                entity.0.display(),
                entity.1
            ));
        }
        ret.push_str("-> End");
        return ret;
    }
}

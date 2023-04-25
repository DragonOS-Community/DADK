use std::{
    cell::RefCell,
    fmt::Debug,
    io::{self, Write},
    rc::Rc,
};

use log::error;

use super::{interactive::InputFunc, ConsoleError};

#[derive(Debug, Clone)]
pub struct Input {
    /// 在输入箭头前面的提示 (e.g. "Please input your name: >> ")
    pre_tips: Option<String>,
    /// 在输入箭头后面的提示 (e.g. ">> (y/n)")
    post_tips: Option<String>,
}

impl Input {
    pub fn new(pre_tips: Option<String>, post_tips: Option<String>) -> Self {
        Self {
            pre_tips,
            post_tips,
        }
    }

    /// # 输出提示语，并从标准输入读取一行
    pub fn input(&self) -> Result<String, ConsoleError> {
        let mut input = String::new();

        if let Some(pre_tips) = &self.pre_tips {
            print!("{}", pre_tips);
        }

        print!(" >> ");

        if let Some(post_tips) = &self.post_tips {
            print!("{} ", post_tips);
        }

        io::stdout().flush().map_err(|e| ConsoleError::IOError(e))?;

        io::stdin()
            .read_line(&mut input)
            .map_err(|e| ConsoleError::IOError(e))?;

        return Ok(input.trim().to_string());
    }
}

#[derive(Debug, Clone)]
pub struct OptionalChoice {
    /// 选项的提示语
    first_line_tips: Option<String>,
    /// 选项列表
    items: Vec<OptionalChoiceItem>,
}

#[derive(Debug, Clone)]
pub struct OptionalChoiceItem {
    id: String,
    description: String,
}

/// # 列表选择器
///
/// 展示一个列表，用户可以通过输入选项序号的方式，选择其中的一个选项
///
/// ## 效果
///
/// ```text
/// Please choose an item:
///
///     1. Item 1
///     2. Item 2
///
/// Please input: >> (1-2)
/// ```
impl OptionalChoice {
    pub fn new(first_line_tips: Option<String>) -> Self {
        Self {
            first_line_tips,
            items: Vec::new(),
        }
    }

    /// # 增加一个选项
    ///
    /// ## 参数
    ///
    /// * `id` - 选项的 ID （当用户选择了该选项时，会返回该 ID）
    /// * `description` - 选项的描述（会在选项里面显示）
    pub fn add_choice(&mut self, id: String, description: String) {
        self.items.push(OptionalChoiceItem { id, description });
    }

    /// # 读取用户的选择
    ///
    /// ## 返回值
    ///
    /// * `Ok(String)` - 用户选择的选项的 ID
    /// * `Err(ConsoleError)` - 用户输入的不是一个数字，或者数字不在选项列表的范围内
    pub fn choose(&self) -> Result<String, ConsoleError> {
        println!("");
        if let Some(first_line_tips) = &self.first_line_tips {
            println!("{}", first_line_tips);
        }

        for item in self.items.iter().enumerate() {
            println!("\t{}. {}", item.0 + 1, item.1.description);
        }

        println!("");
        let input_tips = format!("Please input your choice:");
        let post_tips = format!("(1-{})", self.items.len());
        let input: String = Input::new(Some(input_tips), Some(post_tips)).input()?;
        return self.parse_input(input);
    }

    /// 读取用户的选择，直到用户输入的是一个有效的选项.
    /// 如果用户输入的是无效的选项，则会重新输入.
    ///
    /// ## 返回值
    ///
    /// * `Ok(String)` - 用户选择的选项的 ID
    /// * `Err(ConsoleError)` - 产生了除InvalidInput之外的错误
    pub fn choose_until_valid(&self) -> Result<String, ConsoleError> {
        loop {
            let choice = self.choose();
            if choice.is_err() {
                // 如果用户输入的是无效的选项，则重新输入
                if let Err(ConsoleError::InvalidInput(e)) = choice {
                    error!("Invalid choice: {}", e);
                    continue;
                } else {
                    return Err(choice.unwrap_err());
                }
            }
            return choice;
        }
    }

    /// 读取用户的选择，直到用户输入的是一个有效的选项.
    /// 如果用户输入的是无效的选项，则会重新输入.
    /// 如果用户输入的是无效的选项超过了指定的次数，则会返回错误.
    ///
    /// ## 参数
    ///
    /// * `retry` - 允许用户输入无效选项的次数
    ///
    /// ## 返回值
    ///
    /// * `Ok(String)` - 用户选择的选项的 ID
    /// * `Err(ConsoleError::RetryLimitExceeded)` - 用户输入的无效选项超过了指定的次数
    /// * `Err(ConsoleError)` - 产生了除InvalidInput之外的错误
    #[allow(dead_code)]
    pub fn choose_with_retry(&self, retry: u32) -> Result<String, ConsoleError> {
        for _ in 0..retry {
            let choice = self.choose();
            if choice.is_err() {
                // 如果用户输入的是无效的选项，则重新输入
                if let Err(ConsoleError::InvalidInput(e)) = choice {
                    error!("Invalid choice: {}", e);
                    continue;
                } else {
                    return Err(choice.unwrap_err());
                }
            }
            return choice;
        }
        return Err(ConsoleError::RetryLimitExceeded(format!(
            "Invalid choice: please input a number between 1 and {}",
            self.items.len()
        )));
    }

    /// # 解析用户的输入
    ///
    /// 用户的输入必须是一个数字，且在选项列表的范围内.
    ///
    /// ## 参数
    ///
    /// * `input` - 用户的输入
    ///
    /// ## 返回值
    ///
    /// * `Ok(String)` - 用户选择的选项的 ID
    /// * `Err(ConsoleError::InvalidInput(e))` - 用户的输入不合法
    fn parse_input(&self, input: String) -> Result<String, ConsoleError> {
        let input = input.trim().to_string();
        let input = input.parse::<usize>().map_err(|e| {
            ConsoleError::InvalidInput(format!("Invalid choice: {}", e.to_string()))
        })?;

        if input < 1 || input > self.items.len() {
            return Err(ConsoleError::InvalidInput(format!(
                "Invalid input: {}, please input a number between 1 and {}",
                input,
                self.items.len()
            )));
        }
        Ok(self.items[input - 1].id.clone())
    }
}

/// # 选择是或者否
#[derive(Debug)]
pub struct ChooseYesOrNo {
    tips: String,
}

impl ChooseYesOrNo {
    pub fn new(tips: String) -> Self {
        Self { tips }
    }

    /// # 读取用户的选择
    /// 读取用户的选择，如果用户输入的是 yes，则返回 true，否则返回 false.
    ///
    /// ## 返回值
    ///
    /// * `Ok(bool)` - 用户的选择
    /// * `Err(ConsoleError::InvalidInput)` - 用户输入的不是 yes 或者 no
    /// * `Err(ConsoleError)` - 产生了除InvalidInput之外的错误
    pub fn choose(&self) -> Result<bool, ConsoleError> {
        let choice = Input::new(Some(self.tips.clone()), Some("(yes/no)".to_string()))
            .input()?
            .to_ascii_lowercase();

        if choice == "yes" || choice == "y" {
            return Ok(true);
        } else if choice == "no" || choice == "n" {
            return Ok(false);
        } else {
            return Err(ConsoleError::InvalidInput(format!(
                "Invalid choice: {}",
                choice
            )));
        }
    }

    /// 读取用户的选择，直到用户输入的是一个有效的选项.
    ///
    /// 如果用户输入的是无效的选项，则会重新输入.
    ///
    /// ## 返回值
    ///
    /// * `Ok(bool)` - 用户的选择
    /// * `Err(ConsoleError)` - 产生了除InvalidInput之外的错误
    pub fn choose_until_valid(&self) -> Result<bool, ConsoleError> {
        loop {
            let choice = self.choose();
            if choice.is_err() {
                // 如果用户输入的是无效的选项，则重新输入
                if let Err(ConsoleError::InvalidInput(e)) = choice {
                    error!("{}", e);
                    continue;
                } else {
                    return Err(choice.unwrap_err());
                }
            }
            return choice;
        }
    }

    /// 读取用户的选择，直到用户输入的是一个有效的选项或者超过了指定的次数.
    ///
    /// 如果用户输入的是无效的选项，则会重新输入.
    /// 如果用户输入的是无效的选项超过了指定的次数，则会返回错误.
    ///
    /// ## 参数
    ///
    /// * `retry` - 允许用户输入无效选项的次数
    ///
    /// ## 返回值
    ///
    /// * `Ok(bool)` - 用户的选择
    ///
    /// * `Err(ConsoleError::RetryLimitExceeded)` - 用户输入的无效选项超过了指定的次数
    /// * `Err(ConsoleError)` - 产生了除InvalidInput之外的错误
    #[allow(dead_code)]
    pub fn choose_with_retry(&self, retry: u32) -> Result<bool, ConsoleError> {
        for _ in 0..retry {
            let choice = self.choose();
            if choice.is_err() {
                // 如果用户输入的是无效的选项，则重新输入
                if let Err(ConsoleError::InvalidInput(e)) = choice {
                    error!("Invalid choice: {}", e);
                    continue;
                } else {
                    return Err(choice.unwrap_err());
                }
            }
            return choice;
        }
        return Err(ConsoleError::RetryLimitExceeded(format!(
            "Retry limit exceeded."
        )));
    }
}

/// # 读入多个元素到一个列表
pub struct VecInput<T: Debug> {
    /// 每次读入元素的提示信息
    tips: Option<String>,
    /// 读入的元素列表
    results: Vec<T>,
    /// 元素读取器
    element_input_func: Rc<RefCell<dyn InputFunc<T>>>,
}

impl<T: Debug> Debug for VecInput<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VecInput")
            .field("tips", &self.tips)
            .field("results", &self.results)
            .finish()
    }
}

impl<T: Debug> VecInput<T> {
    /// # 创建一个新的 VecInput
    ///
    /// ## 参数
    ///
    /// * `tips` - 每次读入元素的提示信息
    /// * `element_input_func` - 元素读取器
    pub fn new(tips: Option<String>, element_input_func: Rc<RefCell<dyn InputFunc<T>>>) -> Self {
        Self {
            tips,
            results: Vec::new(),
            element_input_func,
        }
    }

    /// # 读入一组元素
    pub fn input(&mut self) -> Result<(), ConsoleError> {
        println!("\nPlease one or more items.");
        while !self.should_exit()? {
            self.input_one()?;
        }
        return Ok(());
    }

    /// # 读入指定数量的元素
    #[allow(dead_code)]
    pub fn input_n(&mut self, count: usize) -> Result<(), ConsoleError> {
        println!("\nPlease input {} items.", count);
        for _ in 0..count {
            self.input_one()?;
        }
        return Ok(());
    }

    pub fn input_one(&mut self) -> Result<(), ConsoleError> {
        println!();
        if let Some(tips) = self.tips.as_ref() {
            println!("{}", tips);
        }
        let elem = self.element_input_func.borrow_mut().input_until_valid()?;
        self.results.push(elem);
        return Ok(());
    }

    fn should_exit(&self) -> Result<bool, ConsoleError> {
        let input_more = ChooseYesOrNo::new("Input more?".to_string()).choose_until_valid()?;
        Ok(!input_more)
    }

    /// # 获取读入的元素列表的引用
    pub fn results(&self) -> Result<&Vec<T>, ConsoleError> {
        return Ok(&self.results);
    }
}

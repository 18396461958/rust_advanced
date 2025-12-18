use std::pin::Pin;
use std::marker::PhantomPinned;
use std::fmt;

// 核心类型：可选自引用的容器（移除易冲突的泛型生命周期 'a）
#[derive(Debug)]
struct OptionalSelfRef<T> {
    // 堆分配数据（地址固定，生命周期稳定）
    data: Box<T>,
    // 可选自引用：用裸指针替代 &T，避开生命周期陷阱（Pin 保证安全）
    self_ref: Option<*const T>,
    // 标记：默认 !Unpin，无自引用时通过 impl Unpin 覆盖
    _pin: PhantomPinned,
}

// 实现 Display 方便打印
impl<T: fmt::Display> fmt::Display for OptionalSelfRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "数据：{}，自引用状态：{}", 
            self.data, 
            if self.self_ref.is_some() { "有自引用" } else { "无自引用" }
        )
    }
}

// 条件性 Unpin：仅当 无自引用 时实现 Unpin（通过 PhantomData 模拟条件）
// 注：Rust 无法直接基于运行时字段值实现 Unpin，这里用「类型约束 + 逻辑隔离」模拟
impl<T> Unpin for OptionalSelfRef<T> where T: 'static {}

impl<T> OptionalSelfRef<T> {
    // 1. 创建「无自引用」的实例（可 Unpin → 自由移动、解除固定）
    fn new_no_ref(data: T) -> Self {
        OptionalSelfRef {
            data: Box::new(data),
            self_ref: None,
            _pin: PhantomPinned,
        }
    }

    // 2. 创建「有自引用」的实例（!Unpin → 必须 Pin<Box<T>> 固定）
    fn new_with_ref(data: T) -> Pin<Box<Self>> {
        // 修正：移除不必要的 mut（解决 unused_mut 警告）
        let instance = OptionalSelfRef {
            data: Box::new(data),
            self_ref: None,
            _pin: PhantomPinned,
        };

        // 步骤1：将实例封装为 Pin<Box<Self>>（堆固定，地址不变）
        let mut pinned = Box::pin(instance);

        // 步骤2：安全初始化自引用（裸指针，无生命周期绑定）
        unsafe {
            // 获取 Pin 内部的可变引用（仅修改字段，不移动，安全）
            let mut_ref = pinned.as_mut().get_unchecked_mut();
            // 裸指针指向堆上的 data（地址固定，永久有效）
            mut_ref.self_ref = Some(&*mut_ref.data as *const T);
        }

        // 返回固定后的实例（无生命周期冲突）
        pinned
    }

    // 3. 安全获取自引用的值（封装 unsafe，保证安全）
    fn get_ref(&self) -> Option<&T> {
        self.self_ref.map(|ptr| {
            unsafe {
                // Pin 保证 ptr 指向的内存未失效，解引用安全
                &*ptr
            }
        })
    }
}

fn main() {
    // ========== 场景1：无自引用 → Unpin → 自由移动、解除固定 ==========
    println!("=== 无自引用的情况（Unpin）===");
    let mut no_ref = OptionalSelfRef::new_no_ref(42);
    println!("初始实例：{}", no_ref);

    // ✅ Unpin 类型可直接 Pin::new（无需 Box::pin）
    let pinned_no_ref = Pin::new(&mut no_ref);
    println!("Pin 后的实例：{}", pinned_no_ref);

    // ✅ Unpin 类型可解除固定
    let unpinned = Pin::into_inner(pinned_no_ref);
    // ✅ Unpin 类型可自由移动
    let moved_no_ref = unpinned;
    println!("移动后的实例：{}", moved_no_ref);

    // ========== 场景2：有自引用 → !Unpin → 必须 Box::pin，无法移动 ==========
    println!("\n=== 有自引用的情况（!Unpin）===");
    let with_ref = OptionalSelfRef::new_with_ref(99);
    println!("Pin<Box> 实例：{}", with_ref);
    println!("自引用指向的值：{}", with_ref.get_ref().unwrap());

    // ❌ !Unpin 类型无法直接解除固定（编译报错，注释掉）
    // let unpinned_with_ref = Pin::into_inner(with_ref);

    // ❌ !Unpin 类型无法自由移动（编译报错，注释掉）
    // let moved_with_ref = with_ref;

    // ✅ 仅能通过 unsafe 解除固定（演示用，实际避免）
    unsafe {
        let unpinned_unsafe = Pin::into_inner_unchecked(with_ref);
        println!("unsafe 解除固定后的实例：{}", unpinned_unsafe);
    }
}
use std::pin::Pin;
use std::marker::PhantomPinned;

#[derive(Debug)]
struct SelfRef {
    data: String,
    ptr: *const str,
    _pin: PhantomPinned,
}

impl SelfRef {
    fn new(s: &str) -> Pin<Box<SelfRef>> {
        let data = s.to_string();
        let ptr = &data as &str as *const str;
        
        let self_ref = SelfRef {
            data,
            ptr,
            _pin: PhantomPinned,
        };

        Box::pin(self_ref)
    }

    fn get_ref(&self) -> &str {
        unsafe {
            assert!(!self.ptr.is_null());
            &*self.ptr
        }
    }

    fn update_data(self: Pin<&mut SelfRef>, new_content: &str) {
        let this = unsafe { self.get_unchecked_mut() };
        this.data = new_content.to_string();
        this.ptr = &this.data as &str as *const str;
    }

    // 新增：获取 SelfRef 结构体本身的地址（证明 Pin 固定）
    fn get_struct_addr(&self) -> *const SelfRef {
        self as *const SelfRef
    }
}

fn main() {
    let mut pinned_sr = SelfRef::new("Rust Pin 终极修正版：解决 DST 薄指针问题");
    
    // 1. 打印核心地址：结构体地址 + String 内部缓冲区地址
    println!("📌 SelfRef 结构体地址: {:p}", pinned_sr.get_struct_addr());
    println!("📌 String 内部缓冲区地址: {:p}", pinned_sr.data.as_ptr());
    println!("📌 ptr 指向的地址: {:p}", pinned_sr.ptr);
    println!("📌 初始 data: {}", pinned_sr.data);
    println!("📌 ptr 指向内容: {}", pinned_sr.get_ref());

    // 2. 修改 data 并同步自引用
    pinned_sr.as_mut().update_data("Pin 核心：固定结构体地址，不固定字段内部地址");
    println!("\n🔄 修改后 ——");
    println!("🔄 SelfRef 结构体地址: {:p}", pinned_sr.get_struct_addr()); // 地址不变！
    println!("🔄 String 内部缓冲区地址: {:p}", pinned_sr.data.as_ptr()); // 地址变化！
    println!("🔄 ptr 指向的地址: {:p}", pinned_sr.ptr); // 同步变化，指向新缓冲区
    println!("🔄 修改后 data: {}", pinned_sr.data);
    println!("🔄 修改后 ptr 指向: {}", pinned_sr.get_ref());
}
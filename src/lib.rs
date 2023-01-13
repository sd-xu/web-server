use std::thread;
use std::sync::{mpsc, Arc, Mutex};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

// 有着 execute 接收到的闭包类型的 trait 对象的类型别名
type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job), // 是否有 Job 运行
    Terminate,   // 应该停止监听并退出无限循环的信号
}

impl ThreadPool {

    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        // 创建消息通道
        let (sender, receiver) = mpsc::channel();

        // "多生单消", 线程安全智能指针Arc<Mutex<T>>
        // Arc 使多个 worker 拥有接收端, Mutex 确保一次只有一个 worker 能从接收端得到任务
        let receiver = Arc::new(Mutex::new(receiver));

        // Vec::new, 但为 vector 预先分配空间
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            // 传递通道的接收端
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender,
        }
    }

    // trait bound: 
    //   FnOnce(): 处理请求的线程只会执行一次闭包(一个没有参数也没有返回值的闭包)
    //   Send: 将闭包从一个线程转移到另一个线程
    //   'static: 生命周期
    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);  // 让Box指针指向闭包

        self.sender.send(Message::NewJob(job)).unwrap(); // 将任务从通道的发送端发出
    }
}

// graceful shutdown
impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        // 向每个 worker 发送一个 Terminate 消息
        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            // join 需要其参数的所有权: 将 thread 移动出拥有其所有权的 Worker:
            //   Option 上的 take 方法取出 Some 而留下 None
            // if let 解构 Some 并得到线程, join消费这个线程
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv().unwrap();

                match message {
                    Message::NewJob(job) => {
                        println!("Worker {} got a job; executing.", id);

                        job();
                    },
                    Message::Terminate => {
                        println!("Worker {} was told to terminate.", id);

                        break;
                    },
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

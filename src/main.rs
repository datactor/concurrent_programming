// 비동기 함수 test해보기
use std::{sync::Arc, time};
use std::thread::sleep;
use futures::TryFutureExt;
use tokio::sync::Mutex; // lock을 획득한 상태에서 await을 수행하려면 비동기 라이브러리가 제공하는 Mutex 사용
const NUM_TASKS: usize = 8;

// lock을 하고 공유 변수를 증가시키기만 하는 Task.
async fn lock_only(v: Arc<Mutex<u64>>) {
    let mut n = v.lock().await;
    *n += 1;
}

// lock 상태에서 await을 수행하는 task
async fn lock_sleep(v: Arc<Mutex<u64>>) {
    let mut n = v.lock().await;
    let ten_secs = time::Duration::from_secs(10);
    tokio::time::sleep(ten_secs).await; // 문제가 되는 위치. 공유 변수 lock을 획득한 상태에서 await을 수행한다.
    *n += 1;
}

#[tokio::main]
async fn main() -> Result<(), tokio::task::JoinError> {
    let val = Arc::new(Mutex::new(0));
    let mut v = Vec::new();

    // lock_sleep Task 생성
    let t = tokio::spawn(lock_sleep(val.clone()));
    v.push(t);

    // sleep(time::Duration::from_secs(5));

    for i in 0..NUM_TASKS {
        let n = val.clone();
        let t = tokio::spawn(lock_only(n)); // lock_only Task 생성
        v.push(t);
    }

    for i in v {
        i.await?;
    }

    Ok(())
}
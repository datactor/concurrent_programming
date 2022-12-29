// c예제 -> rust로 치환(C언어에 경험이 없어 의도와 다른 번역이 있을 수 있음)

// ch04 동시성 프로그래밍 특유의 버그와 문제점
// 개요
// deadlock, livelock, starvation 등 동기 처리에서의 기본적인 문제들을 먼저 알아본 후, recursive lock,
// 의사 각성과 같은 고급 동기 처리에 관한 문제까지 살펴보자. 동시성 프로그래밍에서 시그널을 다루는 문제 역시 살펴보자.
// 마지막으로 CPU의 out-of-order 실행을 설명하고, 동시성 프로그래밍에서의 문제점 및 memory barrier를 이용한
// 해결법을 익혀보자

use std::sync::Arc;
use crate::ch03_synchronous_processing01::{spinlock_acquire, spinlock_release};

/// 4.1 deadlock(전이 대상이 없는 상태)
/// 식사하는 철학자 문제
/// 철학자는 포크 2개를 동시에 들어야 본인 앞의 음식을 먹을 수 있음.
///
///     철학자 1───────┬───────철학자 4
///      │  (음식)  포크 1  (음식)  │
///      ├── 포크 2       포크 4 ──┤
///      │  (음식)  포크 3  (음식)  │
///     철학자 2───────┴───────철학자 3
/// 1) 왼쪽 포크가 비기를 기다렸다가 왼쪽 포크를 사용할 수 있는 상태가 되면 포크를 듬
/// 2) 오른쪽 포크가 비기를 기다렸다가 오른쪽 포크를 사용할 수 있는 상태가 되면 포크를 듬
/// 3) 식사를 함.
/// 4) 포크를 테이블에 놓음.
/// 5) 단계 1로 돌아감.
///
/// 이때 포크를 드는 타이밍에 따라 모든 철학자가 사용할 수 없는 상태가 되어 더이상 처리가 진행되지 않을 수 있음.
/// (모든 철학자가 왼쪽 포크를 동시에 들었을 경우 deadlock. 더이상 누구도 진행할 수 없음)
/// 이처럼 서로 자원(포크)이 비는 것을 기다리며 더 이상 처리가 진행되지 않는 상태를 deadlock이라 함.
/// 좀 더 엄밀하게 생각해보면, 식사하는 철학자 문제는 state machine에서의 상태 전이(state transition)으로 간주할
/// 수 있으며, 철학자가 2명일 때의 상태 전이는 다음과 같음.
/// NOTE_ 스테이트 머신은 내부에 상태를 가지고 있는 추상적인 기계이며, 입력에 따라 내부 상태가 바뀐다(전이).
/// 예를 들어 자동 판매기도 스테이트 머신이며, 초기 상태의 자동 판매기에 동전을 넣으면 '초기 상태'에서 '동전이 투입된
/// 상태'로 전이된다. enum
#[test]
pub fn func_144p() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    // 포크를 나타내는 2개의 뮤텍스 생성
    let c0 = Arc::new(Mutex::new(()));
    let c1 = Arc::new(Mutex::new(()));

    let c0_p0 = c0.clone(); // Arc::clone()은 clone()이 아니라 참조횟수를 증가시킴.
    let c1_p0 = c1.clone();


    // 철학자 1
    let p0 = thread::spawn(move || {
        for _ in 0..100_000 {
            let _n1 = c0_p0.lock().unwrap(); // 포크를 들고 식사 중이라 표시
            let _n2 = c1_p0.lock().unwrap();
            println!("0: eating");
        }
    });

    // 철학자 2
    let p1 = thread::spawn(move || {
        for _ in 0..100_000 {
            let _n1 = c1.lock().unwrap();
            let _n2 = c0.lock().unwrap();
            println!("1: eating");
        }
    });

    p0.join().unwrap();
    p1.join().unwrap();
}
// 철학자 두명 모두 식사할 수도 있지만, c0과 c1을 서로 가져갔을 경우 데드락이 발생한다.
// 주의 Arc::clone()은 deep copy 아닌 참조 횟수를 증가 시킴.

/// RW락은 특히나 데드락을 주의해야 함. 다음 예제는 B.Quin 등이 보고한 데드락을 발생시키는 예다.
/// Rust로 구현된 앱에서실제로 발견된 버그이기도 하다.
#[test]
pub fn func_145p() {
    use std::sync::{Arc, RwLock};
    use std::thread;

    let val = Arc::new(RwLock::new(true));

    let t = thread::spawn(move || {
        let flag = val.read().unwrap(); // Read락 획득
        if *flag {
            *val.write().unwrap() = false; // Read락 획득 상태에서 Write 획득, 데드락
            println!("flag is true");
        }
    });

    t.join().unwrap();
}
// Read락과 Write락은 동시에 획득할 수 없기 때문에 당연히 데드락 상태가 됨.

/// func_145p()의 RW락을 회피하기 위한 방법 중 하나
#[test]
pub fn func_146p() {
    use std::sync::{Arc, RwLock};
    use std::thread;

    let val = Arc::new(RwLock::new(true));

    let t = thread::spawn(move || {
        let flag = *val.read().unwrap(); // Read락을 획득하고 값을 꺼낸 후 즉시 Read락을 해제함.
                                               // RwLockReadGuard로 감싸져 있던 bool값을 deref로 가져오니,
                                               // 감싸고 있던 락은 파기됨.
        if flag {
            *val.write().unwrap() = false; // Write락 획득
            println!("flag is true");
        }
    });

    t.join().unwrap();
}
// 위 코드에서는 Read락을 얻는 즉시 락이 해제된다. 실제 145p와 146p의 코드 차이는 미미하므로 이처럼 라이프타임 작동을
// 파악하는 것은 일반적으로 어려움. Rust는 잘 설계되었지만 위와 같은 작동이 발생할 수 있으며 이를 '라이프타임의 어둠'
// 이라고 부른다.

/// RwLock을 사용한 데드락의 다른 예
#[test]
pub fn func_147p_1() {
    use std::sync::{Arc, RwLock};
    use std::thread;

    let val = Arc::new(RwLock::new(true));

    let t = thread::spawn(move || {
        let _flag = val.read().unwrap(); // Read락의 값을 _flag에 저장
        *val.write().unwrap() = false; // Write락 획득시 데드락
    });

    t.join().unwrap();
}

/// 147p_1에선 Read락(LockResult)에서 반환된 RwLockReadGuard을 _flag에 저장함. 그러므로 이 변수의 스코프를 벗어날
/// 때까지 락이 반환되지 않으며 Write락을 획득하려하면 데드락이 발생함. 다음처럼 구현하면 데드락이 발생하지 않음.
#[test]
pub fn func_147p_2() {
    use std::sync::{Arc, RwLock};
    use std::thread;

    let val = Arc::new(RwLock::new(true));

    let t = thread::spawn(move || {
        let _ = val.read().unwrap(); // Read락의 값이 즉시 파기되고 락이 해제됨. Rust는 _라는 변수에 저장된 값을 즉시 파기함.
        *val.write().unwrap() = false; // Write락 획득
        println!("not deadlock");
    });

    t.join().unwrap();
}

// 4.2 livelock & starvation
// 식사하는 철학자 문제 알고리즘을 조금 수정해 왼쪽 포크를 들고 약간 기다렸다가 오른쪽 포크를 획득하지 못하면 들고 있던
// 포크를 내려놓도록 하면? 이렇게 변경한 알고리즘은 다음과 같다.
// 1) 왼쪽 포크가 비기를 기다렸다가 왼쪽 포크를 사용할 수 있는 상태가 되면 포크를 든다.
// 2) 오른쪽 포크가 비기를 기다렸다가 오른쪽 포크를 사용할 수 있는 상태가 되면 포크를 든다. 어느 정도 기다려도 오른쪽
//     포크를 들 수 있는 상태가 되지 않으면 왼쪽 포크를 내려놓고 단계 1로 돌아간다.
// 3) 식사를 한다.
// 4) 포크를 테이블에 놓는다.
// 5) 단계 1로 돌아간다.
//
// 이 알고리즘은 잘 작동할 것처럼 보이지만 역시 타이밍에 따라 처리가 진행되지 않는 상황이 발생함.
// 예) 철학자가 동시에 왼쪽 포크를 들고 있다가 왼쪽 포크를 내려놓고 왼쪽 포크를 다시 동시에 드는 작동이 반복된다...
//     -> 위 작동의 반복이 끝날때까지 처리가 진행되지 않음.
//
// 이렇게 리소스를 획득하는 처리는 수행하지만 리소스를 획득하지 못해 다음 처리를 진행하지 못하는 상태를 livelock이라
// 부른다. 라이브락을 예로 들어보면 어떤 두 사람이 좁은 길을 엇갈려 지나 갈때 서로 같은 방향으로 피하는 상태와 같다.
// 어떤 상태에서도 다음 전이 대상이 없는 데드락이 발생하지는 않지만, 라이브락의 발생 가능성이 있다.
//
// 라이브락이 발생하는 스테이트 머신의 정의: 특정한 리소스를 획득하는 상태에는 도달하지만 그 외의 상태에는 절대 도달
// 하지 못하는 무한 전이 사례가 존재함.
// 즉 라이브락이란 상태 전이는 수행되지만, 어떤 리소스 획득 상태로도 전이하지 않는 상태이며, starvation이란 특정
// 프로세스만 리소스 획득 상태로 전이하지 못하는 상태에 있는 것을 말한다.
//
// starvation의 정의: 특정 프로세스만 리소스 획득하지 못함(특정 프로세스만 라이브락 상태). 항상 리소스 획득 상태로
// 도달 가능하지만, 그 상태에 결코 도달하지 못하는 무한 전이 사례가 존재하거나 데드락이 되는 스테이트 머신.
//
// 라이브락: 시스템 전체에 관한 문제, 굶주림: 부분 노드에 관한 문제
//
// NOTE_ 엄밀하게 생각하면 데드락이 발생하는 스테이트 머신도 굶주림을 일으키는 스테이트 머신에 해당함

/// 4.3 은행원 알고리즘
/// 데드락을 회피하기 위한 알고리즘으로 Dikstra가 고안한 Banker's algorithm이 유명하다. 은행원 알고리즘을 살펴보고
/// 식사하는 철학자 문제에 은행원 알고리즘을 적용해 데드락이 발생하지 않고 철학자들이 식사를 마칠 수 있음을 증명해보자.
/// 은행원 알고리즘에서는 은행원이 리소스 배분 방법을 결정한다.
///
/// e.g.) 보유 자금이 2000만원인 은행원이 두 기업(기업 A: 1500만원의 자금이 필요함, B: 2000만원의 자금이 필요함)
///       에게 1000만원씩 동시에 대출해준다면 더이상 보유자금이 없고 두 기업 모두 이익 실현하지 못해 대출상환할 수 없어 데드락 발생.
///
/// 이 예를 보면 은행원의 자본과 각 기업이 필요로 하는 자본을 미리 알고 있는 경우 어떻게 대출하면 데드락이 되는지
/// 예측할 수 있음. 은행원 알고리즘에서는 데드락이 발생하는 상태로 전이하는지 시뮬레이션을 통해 판정함으로써 데드락을 회피함.
#[test]
pub fn func_152p() {
    struct Resource<const NRES: usize, const NTH: usize> {
        available: [usize; NRES],           // 이용 가능한 리소스. available[j]는 j번째 리소스.
        allocation: [[usize; NRES]; NTH],   // allocation[i][j]는 스레드 i가 현재 확보하고 있는 리소스 j의 수
        max: [[usize; NRES]; NTH],          // max[i][j]는 스레드 i가 필요로 하는 리소스 j의 최대값
    }

    impl<const NRES: usize, const NTH: usize> Resource<NRES, NTH> {
        fn new(available: [usize; NRES], max: [[usize; NRES]; NTH]) -> Self {
            Resource {
                available,
                allocation: [[0; NRES]; NTH],
                max,
            }
        }
        // 현재 상태가 데드락을 발생시키지 않는지 확인. 안전한 경우 true, 아닐 경우 false
        fn is_safe(&self) -> bool {
            let mut finish = [false; NTH]; // 스레드 i는 리소스 획득과 반환에 성공했는가를 보기 위한 인스턴스
                                                   // finish[i] = true일 때 스레드 i가 리소스를 확보해
                                                   // 처리를 수행하고, 그 후 모든 리소스를 반환할 수 있음을 나타냄
            let mut work = self.available.clone(); // 이용 가능한 리소스의 시뮬레이션 값. work[j]는
                                                            // 시뮬레이션상에서의 은행원이 보유한 리소스 j의 수를 나타냄
                                                            // clone()하는 이유? 시뮬레이션 중
                                                            // self값을 오염시키지 않기 위함인듯
            loop {
                // 모든 스레드 i와 리소스 j에 대해
                // finish[i] == false && work[j] >= (self.max[i][j] - self.allocation[i][j])
                // 를 만족하는 스레드를 찾는다(아직 시뮬레이션상에서 리소스를 확보하지 못한 스레드로, 원하는 리소스를
                // 은행원이 보유하고 있는 스레드 중에서 찾음). 즉 반환된 리소스 중, 할당했을때 리소스가 work[j]이상
                // 남는 스레드를 찾음.
                let mut found = false;
                let mut num_true = 0;
                for (i, alc) in self.allocation.iter().enumerate() {
                    if finish[i] {
                        num_true += 1;
                        continue
                    }

                    // need[j] = self.max[i][j] - self.allocation[i][j]를 계산하고 모든 리소스 j에 대해
                    // work[j] >= need[j]인지 판정한다(스레드 i가 원하는 리소스를 은행원이 보유하고 있는지 검사).
                    let need = self.max[i].iter().zip(alc).map(|(m, a)| m - a);
                    let is_avail = work.iter().zip(need).all(|(w, n)| *w >= n); // work[j] >= need[j]인지 판정(모든 j들이 만족하는지)
                    if is_avail {
                        // 스레드 i가 리소스 확보 가능
                        found = true;
                        finish[i] = true;
                        for (w, a) in work.iter_mut().zip(alc) {
                            *w += *a // 스레드 i가 리소스를 확보할 수 있다면 스레드 i는 처리를 수행한 뒤
                                     // 현재 확보하고 있는 리소스 반환(work(avail)에 할당량(alc)을 돌려놓음).
                        }
                        break
                    }
                }

                if num_true == NTH {
                    // 모든 스레드가 리소스를 확보할 수 있는 방법이 존재하면 안전하다는 표시로 is_safe에 true를 반환
                    return true;
                }

                if !found {
                    // 모든 스레드가 리소스를 확보할 수 있는 대출 방법이 없음(리소스를 확보할 수 없는 스레드가 있음).
                    break
                }
            }
            false
        }

        // id번째 스레드가 resource를 하나만 확보하도록 하기 위한 함수. 단, 확보한 상태를 시뮬레이션해서 안전하다고 판단된
        // 경우에만 실제로 리소스를 확보한다.
        fn take(&mut self, id: usize, resource: usize) -> bool {
            // 스레드 번호, 리소스 번호 검사
            if id >= NTH || resource >= NRES || self.available[resource] == 0 {
                return false;
            }

            // 리소스 확보를 시험해 본다(id 스레드가 resource를 1개 확보한 상태를 만듬)
            self.allocation[id][resource] += 1;
            self.available[resource] -= 1;

            if self.is_safe() { // is_safe()가 true인 경우, resource를 확보함(take에 true를 반환함) 그렇지 않으면 원상태로 복원.
                true // 리소스 확보 성공
            } else {
                // 리소스 확보에 실패했으므로 상태 복원
                self.allocation[id][resource] -= 1;
                self.available[resource] += 1;
                false
            }
        }

        // id번째 스레드가 resource를 하나 반환하는 함수.
        fn release(&mut self, id: usize, resource: usize) {
            // 스레드 번호, 리소스 번호 검사
            if id >= NTH || resource >= NRES || self.allocation[id][resource] == 0 {
                return;
            }

            self.allocation[id][resource] -= 1;
            self.available[resource] += 1;
        }
    }
    // 여기서 중요한 것은 is_safe함수로, 대출 가능한 리소스와 대출을 시뮬레이션 함(iter로 전체 돌리고 찾을 경우 break).
    // 즉, 대출 가능한 경우 필요한 스레드에 대출하고, 해당 스레드의 리소스를 모두 반환 받는 조작을 반복할 때 모든 스레드가
    // 리소스를 확보할 수 있는지 검사함. take 함수는 리소스를 1개 확보(은행원 입장에서는 대출해줌)하는 함수이며, 확보를
    // 가정했을 때 안전한지 검사하여 안전하다고 판단되었을 때만 리소스를 확보함. 여기서 '안전하다(is_safe() == true)'란
    // 모든 스레드가 리소스를 확보할 수 있는 상태를 가리킴.
    // 이 알고리즘의 핵심은 필요한 리소스를 대출할 수 있는 스레드는 처리를 마치는 즉시 리소스를 반환한다는 제약임.
    // 은행원은 보유한 리소스와 각 스레드에서 필요로 하는 리소스를 비교하여 대출할 수 있는 스레드에 리소스를 분배한 뒤
    // 현재 대출한 리소스를 포함한 모든 리소스를 반환받는 상황을 반복해 예측함. 만약 대출할 수 없는 상황에 빠지면
    // 데드락이나 starvation상태가 된다고 예측할 수 있음.

    // Resource struct를 Arc와 Mutex로 감싼 Banker struct 및 식사하는 철학자 문제를 Banker's 알고리즘으로 구현
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    pub struct Banker<const NRES: usize, const NTH: usize> {
        resource: Arc<Mutex<Resource<NRES, NTH>>>,
    }

    impl<const NRES: usize, const NTH: usize> Banker<NRES, NTH> {
        pub fn new(available: [usize; NRES], max: [[usize; NRES]; NTH]) -> Self {
            Banker {
                resource: Arc::new(Mutex::new(Resource::new(available, max))),
            }
        }

        pub fn take(&self, id: usize, resource: usize) -> bool {
            let mut r = self.resource.lock().unwrap();
            r.take(id, resource)
        }

        pub fn release(&self, id: usize, resource: usize) {
            let mut r = self.resource.lock().unwrap();
            r.release(id, resource)
        }
    }

    // 156p
    use std::thread;

    const NUM_LOOP: usize = 100_000;

    // 이용 가능한 포크 수, 철학자가 이용하는 포크 최대 수 설정. 철학자 2명, 리소스 2개로 초기화. 첫번째 인수
    // [1, 1]은 철학자가 가진 포크 수이며, 두번째 인수 [[1, 1], [1, 1]]는 철학자 1과 2가 필요로하는 포크의 최대 값.
    let banker = Banker::<2, 2>::new([1, 1], [[1, 1], [1, 1]]);
    let banker0 = banker.clone();

    let philosopher0 = thread::spawn(move || {
        for _ in 0..NUM_LOOP {
            // 포크 0과 1을 확보. 철학자는 take 함수로 포크를 얻으며 포크를 얻을 수 있을때까지 spin함.
            // 리소스 확보가 성공했을 때는 true가 반환되어 while문을 빠져나옴. 이때 취득할 스레드 번호(id)와
            // 포크 번호(resource)를 take 함수에 전달함.
            while !banker0.take(0, 0) {}
            while !banker0.take(0, 1) {}

            println!("0: eating");

            // 포크 0과 1을 반환. 포크를 얻으면 식사를 한 뒤 release 함수로 포크를 반환함.
            banker0.release(0, 0);
            banker0.release(0, 1);
        }
    });

    let philosopher1 = thread::spawn(move || {
        for _ in 0..NUM_LOOP {
            // 포크 0과 1을 확보
            while !banker.take(1, 1) {}
            while !banker.take(1, 0) {}

            println!("1: eating");

            // 포크 1과 0을 반환
            banker.release(1, 1);
            banker.release(1, 0);
        }
    });

    philosopher0.join().unwrap();
    philosopher1.join().unwrap();
}
// 각 스레드는 리소스를 얻어 처리하고, 리소스를 반환하는 동작을 반복한다. 이 코드는 데드락에 빠지지 않고 마지막까지 처리를
// 진행한다. 이처럼 은행원 알고리즘을 이용해 데드락을 회피할 수 있음. 그러나 은행원 알고리즘을 사용하기 위해서는 사전에
// 작동하는 스레드 수와 각 스레드가 필요로 하는 리소스의 최대값을 파악하고 있어야 하는 단점이 있음. 데드락을 감지하는
// 다른 방법으로는 리소스 확보에 대한 플래그를 생성해 순환적인 리소스 확보를 하고 있지 않은지 검사하는 방법이 알려져 있음.

/// 4.4 recursive lock
/// 재귀락 정의: 락을 획득한 상태에서 프로세스가 그 락을 해제하기 전에 다시 그 락을 획득하는 것.
/// 재귀락이 발생했을 때 일어나는 일은 3.3절 'Mutex'에서 봤던 것처럼 단순한 뮤텍스 구현에 대해 재귀락을 수행하면 데드락
/// 상태가 된다.
/// 한편 재귀락을 수행해도 처리를 계속할 수 있는 락을 재진입 가능(reentrant)한 락이라고 부른다.
///
/// 재진입 가능한 락의 정의: 재귀락을 수행해도 데드락 상태에 빠지지 않으며 처리를 계쏙할 수 있는 락 메커니즘
#[test]
pub fn func_158p() {
    // 재진입 가능한 Mutex용 type.
    struct ReentLock {
        lock: bool, // 락용 공용 변수(스핀락을 이용하는 변수)
        id: i32, // 형재 락을 획득 중인 스레드 ID. 0이 아니면 락 획득 중임. 즉 각 스레드에 할당된 스레드 ID는 0이 아니어야함.
        cnt: i32, // 재귀락 수행 횟수 카운트.
    }

    // 재귀락 획득 함수
    pub fn reentlock_acquire(mut reent_lock: ReentLock, id: i32) {
        // 락 획득 중이고 동시에 자신이 획득 중인지 판정함. 자신이 락을 획득한 상태라면 카운트를 증가하고 처리 종료.
        if reent_lock.lock && reent_lock.id == id {
            reent_lock.cnt += 1;
        } else { // 어떤 스레드도 락을 획득하지 않았거나, 다른 스레드가 락 획득한 상태면 락을 획득하고 락용 변수에
                 // 자신의 스레드 ID를 설정한 뒤 카운트를 증가
            spinlock_acquire(reent_lock.lock);
            // 락을 획득하면 자신의 스레드 ID를 설정하고 카운트 증가
            reent_lock.id = id;
            reent_lock.cnt += 1;
        }
    }

    pub fn reentlock_release(mut reent_lock: ReentLock) {
        // 카운트를 감소하고, 해당 카운트가 0이 되면 락 해제
        reent_lock.cnt -= 1;
        if reent_lock.cnt == 0 {
            reent_lock.id = 0;
            spinlock_release(reent_lock.lock);
        }
    }
}
// 락을 해제할 때는 재귀락의 카운트를 감소하고 카운트가 0이 되면 실제 락을 해제함. 스핀락 함수는 3.3절 Mutex에서
// 살펴본 함수를 이용한 것으로 사용해보자. 160p 참조.
// 160p의 함수는 기존의 함수와 락을 수행하는 위치가 다름. 단순한 Mutex구현에서는 이런 호출을 수행하면 데드락이 되지만
// reentrant Mutex에서는 처리가 계속됨.
// Pthreads에서는 속도가 빠르지만 재진입 불가능한 뮤텍스,
// 재진입 가능한 뮤텍스, 재진입 시도 시 에러가 발생하는 뮤텍스의 세가지 종류를 이용할 수 있음.
//
/// Rust에서는 재귀락의 작동을 정의하지 않음. Rust에서 재귀락을 수행하는 코드는 상당히 의도적으로 작성하지 않으면
/// 일어나지 않는 것처럼 보임(권장되지 않는 방법인 듯 하다) Rust에서는 락용 변수와 리소스가 강하게 결합되어 있으며,
/// 락용 변수는 명시적으로 클론해야 복제할 수 있기 때문이다. Rust에서 재귀락을 수행하는 예를 살펴보자.
/// 이러한 코드는 의도적으로 작성하지 않는한 만들어지지 않을 것이다.
///
/// 러스트의 idioms에 따르면 다른 스레드에 공유 리소스를 전달할때만 클론하고, 동일 스레드 안에서는
/// 클론해서 이용하지 않는다는 규칙이 있으니 지키자.
#[test]
fn func_161p() {
    use std::sync::{Arc, Mutex};

    // Mutex를 Arc로 작성하고 클론
    let lock0 = Arc::new(Mutex::new(0));
    let lock1 = lock0.clone(); // 144p 봤지만 Rc와 마찬가지로 Arc의 클론은 참조 카운터를 증가시키기만 함.

    let a = lock0.lock().unwrap();
    let b = lock1.lock().unwrap(); // 데드락
    println!("{}", a);
    println!("{}", b);
}

// 4.5 의사 각성
// 3.5절 CondVar에서 조건 변수를 설명할 때 spurious wakeup을 간단히 다뤘었다.
//
// 의사각성의 정의: 특정한 조건이 만족될 때까지 대기 중이어야 하는 프로세스가 해당 조건이 만족되지 않았음에도 불구하고
// 실행 상태로 변경되는 것
//
// 3.5절 106p에서는 의사 각성이 일어나도 문제 없도록 wait 뒤에 반드시 조건이 만족되었는지 확인하도록 했었다. 여기서는
// 의사 각성이 어떤 경우에 일어나는지 알아보자. 전형적으로 wait 안의 시그널에 의한 인터럽트가 그 원인이 되어 의사각성이
// 일어난다. 163p C예제 참조. 요약하면 notify가 wait에서 돌아오는 조건일 경우, 아무도 notify하지 않았을 경우에는
// 돌아오지 않고 대기해야하지만, 리눅스 커널 버전 2.6.22 이전 버전에서는 SIGUSR1 시그널이 송신되고 프로그램이 종료됨.
// 리눅스에서는 wait에 futex(fast userspace mutex)라는 시스템 콜을 이용함. futex 시그널에 의해 의사 각성이 발생했었음.
// 현재는 일어나지않음.

// 4.6 시그널
// 의미전달에 사용되는 대표적인 방법은 메세지와 신호이다. 메세지는 여러가지 의미를 가질 수 있지만 복잡한 대신 신호는
// 1:1로 의미가 대응되어 상대적으로 간단함. 컴퓨터에서 신호, 즉 시그널은 소프트웨어적인 interrupt이다. 인터럽트는
// 하던일 A를 잠시 멈추고 다른일 B를 하고 난 후 다시 A로 돌아와서 멈춘 부분부터 일을 하는 것이라고 할 수 있다.
// 일반적으로 시그널과 멀티스레드는 궁합이 맞지 않음. 왜? 어떤 타이밍에서 시그널 핸들러가 호출되는지 알 수 없기 때문.
// signal의 특성상 signal은 특정 process를 지정해서 보내는 것이지 특정 thread로 전달되는 것이 아니기 때문에
// process 내의 어떤 thread에서 signal을 받아 처리할지 모른다. 예를 들어 main thread와 thread1, thread2가
// 있다고 가정하면 main thread에서 signal handler를 지정하더라도, signal handler에서 처리되지 않고, thread1
// 혹은 thread2로 signal이 전달될 수 있다.
// 164p에서 시그널 핸들러를 사용할 때 데드락이 발생하는 예를 c언어로 작성한 예제를 살펴보자.
//
// 요약하면 시그널 핸들러가 뮤텍스 락을 사용하고(멀티스레드) 처리 후 락을 해제하는 핸들러라고 가정해보자.
// 매인 스레드에서 락을 얻고 해제하는 프로그램이 락을 얻어 처리 중에 있다고 하자. 처리 중에 시그널이 들어온다면
// 시그널 핸들러도 락을 얻으려 할 것이기 때문에 데드락 발생함.
//
// 이런 상태에 빠지는 것을 방지하기 위해 시그널을 수신하는 전용 스레드를 이용할 수 있다. 전용 스레드에서 시그널을 수신하는
// 예제를 구현한 코드를 165p에서 살펴보자.
// 요약하자면,
// 1) 시그널 핸들러 함수 내부에 루프를 돌려 sigwait라는 함수로 시그널을 대기하고,
// 2) 시그널을 받으면, 뮤텍스락을 걸고 처리작업을 한다.
// 3) 워커스레드용 함수도 정의하여 락을 걸고 처리작업을 한다. 그렇지만 이 워커 스레드와 시그널용 스레드는 다른 스레드이며
//    워커 스레드 실행 후에는 시그널 핸들러가 실행되지 않으므로 데드락은 발생하지 않는다.
// 4) main 함수에서 sigaddset 함수로 수신한 시그널의 종류를 지정하고(SIGUSR1), 해당 시그널을 pthread_sigmask 함수를
//    이용해 block으로 설정한다. 어떤 시그널을 블록으로 설정하면(block함) 해당 시그널이 프로세스에 송신되어도 시그널 핸들러가 실행되지
//    않는다(때문에 워커스레드 실행 후에는 시그널 핸들러가 실행되지 않아 데드락이 발생하지 않음).
// 중요한 점은 pthread_sigmask 함수에서 시그널을 block하는 것과 시그널 수신용 스레드를 준비해 시그널 핸들러에서
// sigwait 함수에서 '동기'로 시그널을 수신하는 것이다. 이렇게 하면 어떤 타이밍에 시그널이 발생해도 데드락 상태가 되지 않는다.
// 이 때 유의할 사항은 main thread에서 반드시 signal mask 설정을 한 뒤,
// worker thread를 생성해야 한다.
// https://noritor82.tistory.com/entry/multithread-%ED%99%98%EA%B2%BD%EC%97%90%EC%84%9C-signal-%EC%82%AC%EC%9A%A9

use std::error::Error;
/// Rust에서는 signal_hook이라고 하는 crate가 있으며, 시그널을 다룰 때는 이 크레이트를 이용할 것을 권장한다.
#[test]
pub fn func_167p() -> Result<(), Box<dyn Error>>{
    use signal_hook::{iterator::Signals, SIGUSR1};
    use std::{process, thread, time::Duration};

    // process ID 표시
    println!("pid: {}", process::id());

    let signals = Signals::new(&[SIGUSR1])?; // 수신 대상 시그널인 SIGUSR1을 지정해 SIGNALS type을 생성.
    thread::spawn(move || {
        // receive signals
        for sig in signals.forever() { // forever 함수를 호출해 시그널을 동기적으로 수신한다.
            println!("received signal: {:?}", sig);
        }
    });

    // sleep 10 secs
    thread::sleep(Duration::from_secs(10));
    Ok(())
}
// signal_hook crate를 사용하면 시그널 수신 스레드를 쉽게 작성할 수 있다. 시그널을 수신한 것을 여러 스레드에 알리고
// 싶을 때는 crossbeam-channel crate를 이용하면 좋다. crossbeam-channel은 multi-producer, multi-consumer의
// 송수신을 실현하는 crate. 공식 문서를 살펴보자

/// 4.7 Memory barrier
/// 현대 CPU에서는 반드시 기계어 명령 순서대로 처리를 수행하지는 않는다. 이런 실행 방법을 out-of-order실행이라 부른다.
/// out-of-order 실행 이유는 파이프라인 처리 시 '단위 시간당 실행 명령 수(Instructions-Per-Second: IPS)'를 높이기
/// 위해서이다.
///
/// 예를 들어 A와 B를 다른 메모리 주소로 하고 read A, read B라는 순서의 기계어가 있을 때 A는 메모리상에만 존재하고,
/// B는 캐시 라인상에 존재한다고 하자. 이때 A를 읽는 것을 완료한 뒤 B를 읽는 것보다
/// A를 읽는 것을 완료하기 전의 메모리 패치 중에 B를 읽으면 지연을 줄일 수 있다.
///
/// 이렇게 아웃 오브 오더 실행은 IPS 향상에 기여하지만 몇 가지 다른 문제도 일으킨다. out-of-order 실행에 관한 여러
/// 문제에서 시스템을 보호하기 위한 처리가 memory barrier이다. memory barrier는 memory fence라 부르기도 한다.
/// Arm에서는 메모리 배리어, Intel에서는 메모리 펜스라 부른다.
///
/// 다음 그림은 lock용 명령을 넘어 out-of-order 실행이 되었다고 했을 때 일어나는 문제를 보여준다.
///
///               ┆      lock 획득        ┆
///               ┆<-------------------->┆
///               ┆ read v    write(v+1) ┆
/// 프로세스 A   ------------------------------------------------->
///                    ↑         │
///                    │         │
///             0      │         ↓1
/// 공유 변수 v  ------------------------------------------------->
///                             ╲                 ↑1
///                               ╲               │
///                                 ↘             │
/// 프로세스 B   ------------------------------------------------->
///                               ┆ read v     write(v+1) ┆
///                               ┆<--------------------->┆
///                               ┆       lock 획득        ┆
///
/// 여기서 프로세스 A와 B는 공유 변수에 접근하기 위해 lock을 획득하고, lock 획득 중에 공유 변수를 증가한다고 가정한다.
/// 여기서 만약 프로세스 B의 read 명령이 lock용 명령 이전에 실행되면 그림에서 보는 것과 같이 시간적으로 이전의 공유 변수의
/// 값을 획득하게 된다. 그러면 최종적으로는 공유 변수의 값이 2가 되어야 하지만 1이 되어 레이스 컨디션이 된다.
/// 단, 실제로는 락용 명령을 사용해도 이런 일은 일어나지 않는다. 이것은 메모리 읽기 쓰기 순서를 보증하는 명령이 사용되고
/// 있기 때문이며 그 명령이 바로 메모리 배리어 명령이 된다.
/// memory ordering은 메모리 읽기 쓰기를 수행하는 순서를 말하며, 기계어로 쓰인 순서와 다른 순서로 실행되는 것을
/// reordering이라 부른다. 리오더링이 발생하는 명령 순서는 읽기 쓰기 순서나 CPU 아키텍처에 따라 달라진다.
///
/// 예) 리오더링 패턴: [W->W, W->R, R->R, R->W],
/// 아키텍처: [AArch64: 전부 가능, x86-64: R->W만 가능, RISC-V(WMO): 전부 가능, RISC-V(TSO): R->W만 가능]
/// WMO(Weak Momory Ordering)모드나 AArch64에서는 읽기 쓰기 순서에 관계 없이 리오더링 가능함.
///
/// Rust에서는 아토믹 변수를 읽고 쓸 때 메모리 배리어의 방법을 지정해야 하며, 이때 Ordering type을 이용한다.
/// 예) Relaxed: 제한없음, Acquire: 이 명령 이후의 메모리 읽기 쓰기 명령이 이 명령 이전에 실행되지 않는 것을 보증(ldar).
/// 메모리 읽기 명령에 지정 가능, Release: 이 명령 이전의 메모리 읽기 쓰기 명령이 이 명령 이후에 실행되지 않는 것을 보증(stlr).
/// 메모리 쓰기 명령에 지정 가능, AcqRel: 읽기의 경우는 Acquire, 쓰기의 경우에는 Release로 한다,
/// SecCst: 앞 뒤의 메모리 읽기 쓰기 명령 순서 유지(dmb sy)
/// AcqRel을 Compare And Swap 명령으로 지정한 경우 쓰기에 성공하면 Acquire + Release가 되지만 실패하면 Acquire가 된다.
/// SecCst는 풀 배리어로 AArch64의 dmb sy에 해당한다. 단 실제로는 dmb sy가 아니라 dmb ish 등의 유사 명령을 사용한다.
/// ish는 inner shareable domain의 약어로 동일한 CPU 내 코어 사이에서의 메모리 배리어가 된다.
///
/// 3.8.1절 'Mutex'에서 봤던 것처럼 Rust의 Mutex는 다음과 같은 특징을 가지며 Pthreads에서 발생하는 문제를 방지할 수 있다.
/// - 보호 대상 데이터에는 락 후에만 접근할 수 있다.
/// - 락 해제는 자동으로 수행된다(스코프를 넘어가면 drop됨).
/// Rust의 atomic var를 이용한 스핀락의 구현을 기반으로 위의 특징을 어떻게 구현하는지 알아보자.
#[test]
pub fn func_172p() {
    use std::cell::UnsafeCell; // UnsafeCell type. 이 타입은 Rust의 차용 룰을 파기(정확히 말하면 컴파일에 파기)
                               // Mutex 등의 메커니즘을 구현하기 위해서는 반 필수적으로 사용.
    use std::ops::{Deref, DerefMut}; // Deref trait을 구현하면 애스터리스크를 이용해 참조 제외를 수행할 수 있음.
                                     // Rust의 Mutex는 lock했을 때 Guard용 객체를 반환함. 가드용 객체는 참조를
                                     // 제외함으로써 보호 대상 데이터를 읽고 쓸 수 있음.
    use std::sync::atomic::{AtomicBool, Ordering}; // AtomicBool은 아토믹 읽기 쓰기를 수행하는 논리값 type,
                                                   // Ordering은 메모리 배리어 방법을 나타내는 type
    use std::sync::Arc;

    const NUM_THREADS: usize = 4;
    const NUM_LOOP: usize = 100_000;

    // spinlock용 type 정의. lock용 공유 변수와 lock 대상 데이터를 유지함. 유지 대상 데이터는 여러 스레드가
    // Mutable하게 접근할 가능성이 있으므로 UnsafeCell type으로 감싼다.
    struct SpinLock<T> {
        lock: AtomicBool, // lock용 공유 변수
        data: UnsafeCell<T>, // 보호 대상 데이터
    }

    // 락 해제 및 락 중에 보호 대상 데이터를 조작하기 위한 type. 이 타입의 값이 스코프로부터 제외되었을 때 자동적으로
    // 락이 해제되지만 락을 해제하기 위해 SpinLock type의 참조(라이프타임)를 유지하고 있다.
    struct SpinLockGuard<'a, T> {
        spin_lock: &'a SpinLock<T>,
    }

    impl<T> SpinLock<T> {
        fn new(v: T) -> Self {
            SpinLock {
                lock: AtomicBool::new(false),
                data: UnsafeCell::new(v),
            }
        }

        // lock을 수행하는 lock 함수. TTAS에 의해 lock용 공유 변수가 false가 되어 lock이 해제되는 것을 기다린다.
        // 공유 변수가 false인 경우에는 memory ordering에 Acquire를 지정하여 아토믹하게 공유 변수를 true로 설정한다.
        fn lock(&self) -> SpinLockGuard<T> {
            loop {

                // lock용 공유 변수(SpinLock의 AtomicBool)가 false가 될 때까지 대기
                while self.lock.load(Ordering::Relaxed) {}

                // lock용 공유 변수를 아토믹하게 씀
                if let Ok(_) = self.lock
                    .compare_exchange_weak( // 현재 값이 current 인자와 같으면 bool에 값을 저장함.
                        false, // false면
                        true, // true를 쓴다.
                        Ordering::Acquire, // 성공시의 order
                        Ordering::Relaxed)  // 실패시의 order
                {
                    break; // self.lock.compare_exchange_weak()가 failure(Err)가 아닌 success(Ok)일 때
                }          // loop를 끝냄. falure라면 루프를 다시 돌음.
            }
            SpinLockGuard { spin_lock: self } // lock 획득에 성공하면 루프를 벗어나 SpinLockGuard type의 값에
        }                                     // 자신의 참조를 전달해 lock 획득 처리를 종료한다.
    }
    // SpinLock type은 스레드 사이에서 공유 가능하도록 지정
    unsafe impl<T> Sync for SpinLock<T> {} // SpinLock type은 스레드 사이에서 공유할 수 있다고 지정.
                                           // 이 지정은 Rust의 Mutex type 등에도 수행되고 있음.
    unsafe impl<T> Send for SpinLock<T> {} // Send trait을 구현하면 채널을 통해 값을 송신할 수 있게 됨.

    // 락 획득 후 자동으로 해제되도록 Drop trait 구현. 여기에서는 SpinLockGuard type의 변수가 스코프에서 제외되었을 때
    // 자동으로 락 해제. 락 해제를 잊을 경우 방지. 락 해제에 필요한 memory ordering은 Release이므로 false 기록시 지정됨.
    impl<'a, T> Drop for SpinLockGuard<'a, T> {
        fn drop(&mut self) {
            self.spin_lock.lock.store(false, Ordering::Release); // drop되면 Release ordering
        }                                                                 // 방식으로 false를 store함.
    }

    // SpinLockGuard type에 Deref trait을 구현하여 보호 대상 데이터의 immutable한 참조 deref 가능하게 하기.
    // 여기서는 보호 대상 데이터로의 참조를 취득하도록 한다. 이렇게 함으로써 lock 보호 시에 얻어진 SpinLockGuard type의
    // 값을 통해 보호 대상 데이터의 읽기 쓰기가 가능해짐. 이같은 작업은 Rust의 MutexGuard type에서도 수행된다.
    impl<'a, T> Deref for SpinLockGuard<'a, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            unsafe { &*self.spin_lock.data.get() }
        }
    }
    // 보호 대상 데이터의 mutable한 참조 역시 deref 가능하게 하기.
    impl<'a, T> DerefMut for SpinLockGuard<'a, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            unsafe { &mut *self.spin_lock.data.get() }
        }
    }

    let lock = Arc::new(SpinLock::new(0));
    let mut v = Vec::new();

    for _ in 0..NUM_THREADS {
        let lock0 = lock.clone(); // 참조 카운트 증가
        // 스레드 생성
        let t = std::thread::spawn(move || {
            for _ in 0..NUM_LOOP {
                // lock
                let mut data = lock0.lock(); // 락 해제 및 락 중에 보호 대상 데이터를 조작하기 위해 Guarding.
                *data += 1; // SpinLockGuard type에 deref trait을 구현했으므로 deref 가능
            }
        });
        v.push(t);
    }

    for t in v {
        t.join().unwrap();
    }

    println!("COUNT = {} (expected = {})", *lock.lock(), NUM_LOOP * NUM_THREADS);
}
// 이렇게 스핀락에서는 락 획득과 해제시 Acquire(커스텀 lock()을 걸때 compare_exchange_weak()함수로)와
// Release(drop으로)를 지정한다. 이렇게 함으로써 위의 그림(프로세스 A가 락을 획득 중에 write이 끝나기 전에 프로세스 B가
// lock을 획득 했다면 A에서 write해서 바뀐 값이 공유 변수에 반영되지 않는 문제)를 회피할 수 있음.
// 락 함수에서는 compare_exchange_weak 함수를 호출하고 Atomic하게 테스트와 대입을 수행한다.
// 이 함수는 atomic 변수와 첫 번째 인수(current자리)의 값이 같은지 테스트하고, 같은 경우 두 번째 인수(new자리)의 값을
// Atomic 변수에 대입한다. 테스트에 성공한 경우의 Ordering은 세번째 인수(success자리), 실패한 경우의 Ordering은
// 네번째 인수(failure자리)에 지정한다. compare_exchange_weak() 함수는 테스트에 성공한 경우라도 대입에 실패하면
// 재시도하지 않는다(그렇지만 Result라 Err를 반환하긴함). 이것은 예를 들어 LL/SC 명령(A가 읽기 쓰기 중에 다른
// CPU(프로세스 B)에서 값을 읽고 쓴다면 A가 하던 쓰기가 실패함)을 이용해 Atomic 명령을 구현한 경우 테스트에 성공해도
// 다른 CPU에서 같은 값을 써넣는 경우 배타적 쓰기가 실패하기 때문에 발생한다(때문에 compare_exchange_weak()을 사용한
// 위의 lock()메서드에서는 loop를 돌려 실패한 경우 재시도한다.).
// Rust에서는 weak이 아닌 compare_exchange 함수도 제공하고 있으며, 이는 테스트에 성공해 쓰기에 실패한 경우 재시도한다.
// 그렇기 때문에 스핀락의 구현에서는 오버헤드가 발생할 가능성이 있다.



// ############################################################################################
// 이와 같이 Rust에서는 Atomic 변수의 읽기 쓰기는 Memory Ordering을 지정해서 수행한다.
// 이것은 얼핏 보기에는 번거로워 보이지만 Memory Ordering을 지정함으로써 서로 다른 CPU 아키텍처에서
// '최적의 코드'를 컴파일러가 생성해주므로 Assembly를 쓰는 것보다 범용성이 높아진다!!!!!
// ############################################################################################
// 이 예에서는 스핀락을 이용했지만 단순히 횟수를 셀 뿐이라면 Relaxed로 충분한 것을 알 수 있다.
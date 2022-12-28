// ch04 동시성 프로그래밍 특유의 버그와 문제점
// 개요
// deadlock, livelock, starvation 등 동기 처리에서의 기본적인 문제들을 먼저 알아본 후, recursive lock,
// 의사 각성과 같은 고급 동기 처리에 관한 문제까지 살펴보자. 동시성 프로그래밍에서 시그널을 다루는 문제 역시 살펴보자.
// 마지막으로 CPU의 out-of-order 실행을 설명하고, 동시성 프로그래밍에서의 문제점 및 memory barrier를 이용한
// 해결법을 익혀보자

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
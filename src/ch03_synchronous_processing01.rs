// 3. 동기 처리 1
// 학습 개요
// 여러 프로세스 사이에 타이밍 동기화, 데이터 업데이트 등을 협조적으로 수행하는 처리를 synchronous processing이라 부름.
// 동기처리에 관해 하드웨어 관점의 메카니즘부터 알고리즘까지 살펴보자.
//
// 동기처리가 필요한 이유인 race condition, 그리고 atomic 연산 명령과 atomic 처리, mutex, semaphore, 조건 변수,
// barrier synchronization, Readers-Writer lock, Pthread에 관해서 익혀 볼 것(C, assembly).
//
// Rust에서는 동기 처리에서 놓치기 쉬운 실수를 타입 시스템을 이용해 방지할 수 있음. C와 Rust의 동기 처리 기법을
// 비교 학습함으로써 Rust의 선진적인 동기 처리 기법을 깊이 이해해보자. 그리고 아토믹 명령에 의존하지 않는 알고리즘인
// bakery algorithm에 대해서도 살펴보자.
// *NOTE_ Rust의 동기 처리 lib은 내부적으로 Pthreads를 사용함. 그러므로 동시성 프로그래밍의 구조를 이해하기 위해
// 먼저 C의 Pthread부터 숙지하는 것이 좋다.
// 여기에서 말할 스레드나 OS 프로세스들은 모두 프로세스라고 표현될 예정. OS에 한정하지 않을 것이기 때문. 실제로
// 여기의 아토믹 명령이나 spinlock은 스레드나 OS 프로세스 뿐 아니라 커널 공간에도 적용됨.


// 3.1 Race condition
// 레이스 컨디션은 경합 상태라 불리며, 여러 프로세스가 동시에 공유하는 자원에 접근함에 따라 일어나는 예상치 않은 상태를 말함.
// 동시성 프로그래밍에서는 레이스 컨디션을 일으키지 않는 것이 매우 중요함.
// 레이스 컨디션의 예로 공유 메모리 상에 있는 변수를 여러 프로세스가 증가시키는 상황을 가정. 또한 메모리에 읽기와 쓰기를
// 동시에 수행할 수 없고, 각기 다른 타이밍에 수행해야 한다고 가정해보자. 다음은 프로세스 A와 B가 공유변수 v를 증가시키는 예.
//            read v   write (v+1)       read v    write (v+1)
// 프로세스 A --------------------------------------------------------------->
//               ↑        │                 ↑         │
//               │        │                 │         │
//          0   0│        ↓1               2│         ↓3
// 공유변수 v --------------------------------------------------------------->
//                            1│        ↑2      2│        ↑3
//                             │        │        │        │
//                             ↓        │        ↓        │
// 프로세스 B --------------------------------------------------------------->
//                          read v   write (v+1)      write (v+1)
//                                             read v(경합 발생)
// 레이스 컨디션을 일으키는 프로그램 코드 부분을 critical section이라 함.


// 3.2 atomic operation
// atom은 고대 그리스 철학자 Democritus가 발명한 용어로 이 세상은 분할할 수 없는 단위의 물질로 구성되어 있다는 생각에서
// 출발했음. 마찬가지로 아토믹 처리란? 불가분 조작 처리라 불리며 처리로 더 이상 나눌 수 없는 처리 단위를 의미함.
// 엄밀하게 생각하면 CPU의 add나 mul 같은 명령도 아토믹 처리로 생각할 수 있지만 일반적으로 아토믹 처리는 여러 번의
// 메모리 접근이 필요한 조작이 조합된 처리를 말하며 덧셈이나 곱셈 등 단순한 명령을 의미하지는 않음.
//
// 정의 - 아토믹 처리의 성질
// 어떤 처리가 아토믹하다. => 해당 처리의 도중 상태는 시스템적으로 관측할 수 없으며,
// 만약 처리가 실패하면 처리 전 상태로 완전 복원됨.
//
// 현대 컴퓨터 상에서의 동기 처리 대부분은 아토믹 명령에 의존함. 아토믹 처리를 익혀 동시성 프로그래밍의 구조를 깊게 이해해보자.
//
//

/// 3.2.1 Compare and Swap
/// CAS(Compare and Swap)은 동기 처리 기능의 하나인 세마포어(semaphore), lock-free, wait-free한 데이터 구조를
/// 구현하기 위해 이용하는 처리다.
fn compare_and_swap(mut p: u64, val: u64, newval: u64) -> bool {
    if p != val {
        return false
    } p = newval;
    true
}
// 이 프로그램은 아토믹하다고 할 수 없음. 실제로 2행의 p != val은 4행의 p = newval과 별도로 실행됨. 위 함수가
// 컴파일되어 어셈블리 레벨에서도 여러 조작을 조합해 구현됨. rust에도 이와 같은 조작을 아토믹으로 처리하기 위한 내장함수인
// compare_and_swap() 함수가 있음.
use std::sync::atomic::{AtomicUsize, Ordering};

pub fn compare_and_swap2() {
    let some_var = AtomicUsize::new(5);

    assert_eq!(some_var.compare_and_swap(5, 10, Ordering::Relaxed), 5);
    assert_eq!(some_var.load(Ordering::Relaxed), 10);

    assert_eq!(some_var.compare_and_swap(6, 12, Ordering::Relaxed), 10);
    assert_eq!(some_var.load(Ordering::Relaxed), 10);

    assert_eq!(some_var.compare_and_swap(99, 100, Ordering::Relaxed), 10);
    assert_eq!(some_var.load(Ordering::Relaxed), 10);
}
// compare_and_swap() 연산은 특정 메모리위치의 값이 주어진 값과 동일하다면 해당 메모리 주소를 새로운 값으로 대체함.
// 이 연산은 atomic이기 때문에 새로운 값이 최신의 정보임을 보장한다. 만약 값 비교 와중에 다른 스레드에서 그 값이
// 업데이트 되면 쓰기는 실패한다. 연산의 결과는 쓰기가 제대로 이루어졌는지를 나타낸다. 간단히 bool을 리턴하기도
// 하고(compare-and-set), 메모리 위치에서 읽은 값(쓰인 값이 아님)을 리턴하기도 한다.


// 3.2.2 Test and Set
fn test_and_set(mut p: bool) -> bool {
    if p {
        return true
    } else {
        p = true;
        return false
    }
}
// 이 함수는 p의 값이 true면 true를 그대로 반환하고, false면 p의 값을 true로 설정하고 false로 반환한다. TAS도
// CAS와 마찬가지로 아토믹 처리의 하나이며, 값의 비교와 대입이 아토믹하게 실행되며 스핀락 등을 구현하기 위해 이용된다.
// *spin-lock? 이름 그대로 만약 다른 스레드가 lock을 소유하고 있다면 그 lock이 반환될 때까지 계속 확인하며 기다리는 것이다.
// '조금만 기다리면 바로 쓸 수 있는데 굳이 Context Switching으로 부하를 줄 필요가 있나?'라는 컨셉으로 개발된 것으로
// Critical Section에 진입이 불가능할 때 컨텍스트 스위칭을 하지 않고 잠시 루프를 돌면서 재시도를 하는것을 말함.
// http://itnovice1.blogspot.com/2019/09/spin-lock.html
//
//
// 3.2.3 Load-Link/Store-Conditional
// x86등의 cpu 아키텍처에서는 lock 명령 접두사를 사용해 메모리에 읽고 쓰기를 배타적으로 수행하도록 지정했음.
// ARM, RISC-V, POWER, MIPS등의 cpu에서는 Load-Link/Store-Conditional(LL/SC)명령을 이용해 아토믹 처리를 구현한다.
// LL? 로드 링크는 메모리 읽기를 수행하고 배타적 엑세스를 위해 메모리 위치를 내부적으로 등록함.
// SC? Store-Conditional 명령은 이전 로드 링크 명령 이후 메모리 위치에 대한 쓰기가 없는 경우에만 메모리 쓰기를 수행함.
//
// AArch64의 LL/SC명령(A/L은 load-Acquire와 store-reLease 명령) 표 3-2에서 자세히보기
//                      LL              SC              클리어 명령
// 32 또는 64비트       ldxr            stxr                clrex
// 32 또는 64비트(A/L)  ldaxr           stlxr               clrex
//
// LL명령은 메모리 읽기를 수행하는 명령이지만 읽을 때 메모리를 배타적으로 읽도록 지정한다. SC 명령은 메모리 쓰기를
// 수행하는 명령이며, LL 명령으로 지정한 메모리로의 쓰기는 다른 CPU가 수행하지 않는 경우에만 쓰기가 성공한다.
//            load-link v          store-conditional (v+1)
// 프로세스 A --------------------------------------------------------------->
//               ↑                         │
//               │                         │
//          0   0│                         ↓fail
// 공유변수 v --------------------------------------------------------------->
//                      0│        ↑1
//                       │        │
//                       ↓        │
// 프로세스 B --------------------------------------------------------------->
//                    read v   write (v+1)
// 1) 먼저 프로세스 A가 LL명령을 이용해 공유 변수 v의 값을 읽는다.
// 2) 이어서 다른 프로세스 B가 공유 변수 v에서 값을 읽고, 그 후 어떤 값을 써넣음.
// 3) 다음으로 프로세스 A가 SC명령을 이용해 값을 써넣지만 프로세스 A의 LL명령과 SC명령 사이에 공유 변수 v로의
//    쓰기가 발생하므로 이 쓰기는 실패한다.
// 4) 쓰기가 실패한 경우에는 다시 한번 읽기와 쓰기를 수행함으로써 실질적으로 아토믹하게 증가시킬 수 있다.
// A.3절 메모리 읽기 쓰기(부록) 를 참조하면 읽기 수행 명령은 읽기 쓰기 수행 크기에 따라 다르므로 각각에 대응한
// LL/SC명령을 제공한다. 그리고 ldaxr 같이 명령 중에 a가 있는 LL명령은 load-Acquire를 의미하고, stlxr같이
// 명령 중에 l이 있는 SC 명령은 store-reLease를 의미한다.
// load-Acquire 명령에 이어지는 명령은 반드시 이 명령이 종료된 후 실행되는 것을 보증하며,
// store-reLease 명령어 이전의 명령은 이 명령 실행 전에 반드시 모두 실행됨을 보증한다.
// 이는 CPU의 out-of-data 실행을 제어하기 위한 것으로 자세한 내용은 4.7절 '메모리 배리어'에서 살펴보자.
// clrex 명령은 클리어 명령이라 불리는 명령어로 ldxr 명령 등에서 배타적으로 읽기를 수행한 메모리 상태를 배타 접근
// 상태에서 open access 상태로 되돌리는 명령어다.
//
// AArc64 아키텍처는 LL/SC 명령을 이용해서 다른 cpu로부터의 쓰기 여부를 검출할 수 있으며 이는 x86-64의 lock 명령
// 접두사와 크게 다른 점이다. x86-64 아키텍처에서 이를 검출하려면 hazard pointer라 불리는 기법 등을 이용해야함.
// 이에 관해서는 7.3.2절 'ABA 문제'를 볼때 유심히 보도록 하자
// *NOTE_ Arm v8.1부터 CAS 명령 등이 추가되었기 때문에 LL/SC를 사용하지 않고 아토믹 처리를 구현할 수 있음!
fn tas_release(mut p: bool) {
    p = false
} // 여기선 단순하게 lock을 false로 돌려놓는다



// 3.3 mutex(MUTual EXecution)
// 배타 실행(Exclusive Execution)이라고도 불리는 동기 처리 방법. 이름 그대로 뮤텍스는 critical section을
// 실행할 수 있는 프로세스 수를 최대 1개로 제한하는 동기처리다. 배타적 실행을 위해 공유 변수로 사용할 플래그를 준비하고
// 해당 플래그가 true면 크리티컬 섹션을 실행하고 그렇지 않으면 실행하지 않는 처리를 고려할 수 있음.
pub struct Lock {
    inner: bool,
}

impl Lock {
    pub fn new() -> Self {
        Self {
            inner: false,
        }
    }
}

pub fn mutex01() {
    let mut lock = Lock::new(); // 공유 변수 1

    some_func(lock.inner);
}

pub fn some_func(mut lock: bool) {
    if !lock { // 2
        lock = true; // lock 획득
        // critical section
    } else {
        some_func(lock)
    }
    lock = false; // lock-freed
}
// 1) 각 프로세스에서 공유되는 변수를 정의함(공유 변수라 가정) 초깃값은 false
// 2) lock은 공유 변수이기 때문에 false라면 free상태) false라면(아무 프로세스도 크리티컬 섹션을 실행하고 있지 않음)
//    critical section을 실행중이라는 것을 나타내기 위해 공유 변수 lock에 true를 대입하고 크리티컬 섹션 실행.
//    반대로 true일 경우(다른 프로세스가 크리티컬 섹션을 실행중일 경우) 재시도.
// 3) 공유 변수 lock에 false를 대입하고 처리 종료.
// *NOTE_ critical section 실행 권한을 얻는 것을 'lock을 획득한다'고 하며, 획득한 권한을 반환하는 것을 'lock을
// 해제한다'고 말함
//
// 이 함수는 여러 프로세스에서 동시에 호출되며, lock변수는 모든 프로세스에서 공유됨. 이 프로그램은 얼핏 잘 작동할 것처럼
// 보이지만 여러 프로세스가 크리티컬 섹션을 동시에 실행하게 될 가능성이 있다. 다음 그림[3-3]에서 배타 실행이 되지 않는 예를 보자
//                   if(!lock)   lock = true
// 프로세스 A   ---------------------------------------------------->
//                         ↑           │
//                         │           │           레이스 컨디션
//            false   false│           ↓true  <------------------->
// lock 변수 v ---------------------------------------------------->
//                           false│           ↑true
//                                │           │
//                                ↓           │
// 프로세스 B   ---------------------------------------------------->
//                             read v     lock = true
// 베타 실행이 되지 않는 예
// 1) 프로세스 A가 크리티컬 섹션을 진입하기전에 lock이 freed 된걸 확인하고 진입함
// 2) 프로세스 B가 락이 잠기기 전에 크리티컬 섹션으로 진입해버림.
// 3) 프로세스 B가 진입하고 락을 잠근시점 부터는 A와 B 둘다 크리티컬 섹션으로 진입경쟁을 할 것이기 때문에 레이스 컨디션.
pub fn mutex02() {
    let mut lock = Lock::new();

    some_func2(lock.inner);
}

pub fn some_func2(mut lock: bool) {
    if !test_and_set(lock) { // 검사 및 락 획득
        // critical section
    } else {
        some_func2(lock)
    }
    tas_release(lock);
}
// 이걸 만든사람은 천재가 아닐까? 아토믹 버전의 TAS함수를 이용해 검사와 값설정을 수행함. 위의 some_func()는 검사와
// 값 설정이 여러 조작으로 만들어져 있어, 이것이 올바르게 배타제어가 되지 않는 원인이었음. 그래서 여기에서는 TAS를
// 이용해 아토믹하게 검사와 값 설정을 하도록 수정했음.
//                   if(!TAS(&lock))
// 프로세스 A   ---------------------------------------------------->
//                         ↑
//                         │
//            false   false↓true
// lock 변수 v ---------------------------------------------------->
//                                   true│
//                                       │
//                                       ↓
// 프로세스 B   ---------------------------------------------------->
//                                  if(!TAS(&lock))
//
// TAS를 이용함으로써 lock 변수에 읽기와 쓰기를 동시에 수행할 수 있게 됨. 그리고 TAS에서 이용되는 xchg명령은 캐시
// 라인을 배타적으로 설정하므로 같은 메모리에 대한 TAS는 동시에 실행되지 않는다.
// 이 논리라면 tas_release는 상수로 구현해도 문제가 없을까?
//
//
// 3.3.1 spinlock
// 위의 fn mutex02()에서는 락을 얻을 수 있을때까지 루프(재귀)를 반복했음. 이렇게 리소스가 비는 것을 기다리며(polling)
// 확인하는 락 획득 방법을 spinlock이라 부른다. 전형적으로 스핀락용 API는 lock 획득용과 lock 해제용 함수 두가지가
// 제공되며 이들은 다음 코드와 같이 기술된다. 이 알고리즘에서는 bool type의 공유변수 lock을 하나 이용하며 초깃값은 false이다.
fn spinlock_acquire(mut lock: bool) {
    while test_and_set(lock) {} // 1
}

fn spinlock_release(mut lock: bool) {
    tas_release(lock); // 2
}
// 1) 공유 변수에 대한 포인터를 받아 TAS를 이용해 락을 획득할 때까지 루프를 돌림
// 2) 단순히 공유 변수를 인수로 tas_release 함수를 호출함.
//
// 코드는 정상작동하지만 일반적으로 아토믹 명령은 실행 속도상의 페널티가 큼. 그래서 TAS를 호출하기 전에 검사를 하고 나서
// TAS를 수행하도록 개선할 수 있으며 개선한 결과는 다음 코드와 같음.
fn spinlock_acquire2(mut lock: bool) { // c에서는 인자를 volatile 키워드를 붙여 최적화를 막음
    loop {
        while lock {}; // 1
        if !test_and_set(lock) {
            break;
        }
    }
}

fn spinlock_release2(mut lock: bool) {
    tas_release(lock);
}
// 1) lock 변수가 false가 될때까지 루프를 돌기 때문에 아토믹 명령을 불필요하게 호출하는 횟수를 줄임.
// 이렇게 TAS 전에 테스트를 수행하는 방법을 Test and Test and Set(TTAS)라고 한다.
// 스핀락에서는 락을 획득할 수 있을 때까지 루프에서 계속해서 공유변수를 확인하기 때문에 critical section 안에서의
// 처리량이 많은 경우에는 불필요한 CPU 리소스를 소비하게 됨. 그래서 lock을 획득하지 못한 경우에는 context switch로
// 다른 프로세스에 CPU 리소스를 명시적으로 전달해 계산 자원 이용을 효율화하는 경우가 있음. 그리고 크리티컬 섹션 실행
// 중에 OS scheduler에 의해 OS 프로세스가 할당되어 대기 상태가 되어버린 경우에는 특히 페널티가 크다. 하지만
// userland app에서는 OS에 의한 할당을 제어하기 어렵기 때문에 단일 스핀락 이용은 권장하지 않으며 다음에 살펴 볼
// Pthread 또는 프로그래밍 언어 라이브러리가 제공하는 mutex를 이용하거나 스핀락과 이들 lib를 조합해 이용해야함.
// 다음 코드는 스핀락의 이용 례
fn some_func3(mut lock: bool) {
    loop {
        spinlock_acquire2(lock); // lock acquisition 1
        // Critical Section 2
        spinlock_release2(lock); // lock free 3
    } // 반납하더라도 계속 spin?
}
//
//
// 3.3.2 Pthreads의 Mutex
// 일반적인 프로그램의 경우 스핀락은 직접 구현하는 것보다 라이브러리에서 제공하는 mutex를 이용하는 것이 나음.
//
//
// 3.3.2 Pthreads의 Mutex
// 일반적인 프로그램의 경우 스핀락은 직접 구현하는 것보다 라이브러리에서 제공하는 mutex를 이용하는 것이 나음.
// rust는 Arc::new(Mutex::new()); 로 생성후 lock을 걸면됨.


// 3.4 세마포어(semaphore)
// Mutex에서는 락을 최대 1개 프로세스까지 획득할 수 있었지만 세마포어를 이용하면 최대 N개 프로세스까지 동시에 락을
// 획득할 수 있다. 여기서 N은 프로그램 실행 전에 임의로 결정할 수 있는 값. 즉, 세마포어는 Mutex를 보다 일반화한
// 것 또는 Mutex를 세마포어의 특수한 버전이라고 생각할 수 있음.
// 다음 코드는 세마포어 알고리즘. 여기서 N은 동시에 락을 획득할 수 있는 프로세스 수의 상한.
// usize 타입의 공유 변수 cnt를 하나씩 이용하며 초깃값은 0이다.
use std::sync::{Arc, Mutex};
use std::thread;

static NUM: AtomicUsize = AtomicUsize::new(4);

pub fn semaphore_acquire(mut cnt: AtomicUsize) -> AtomicUsize { // 1
    loop {
        while cnt.load(Ordering::SeqCst) >= NUM.load(Ordering::SeqCst) {}; // 2
        cnt.fetch_add(1, Ordering::SeqCst); // 3
        println!("{}", cnt.load(Ordering::SeqCst));
        if cnt.load(Ordering::SeqCst) <= NUM.load(Ordering::SeqCst) { // 4
            return cnt
            // break;
        }
        cnt.fetch_sub(1, Ordering::SeqCst); // 5
        println!("{}", cnt.load(Ordering::SeqCst));
        return cnt
    }
}
pub fn semaphore_release(mut cnt: AtomicUsize) -> AtomicUsize {
    cnt.fetch_sub(1, Ordering::SeqCst); // 6
    println!("{}", cnt.load(Ordering::SeqCst));
    cnt
}
// 1) 인수로 AtomicUsize 타입의 공유 변수를 받고 뮤텍스의 경우 락이 이미 획득되어 있는지만 알면 되므로 bool 타입
//    공유 변수를 이용했지만 세마포어에서는 다수의 프로세스가 락을 획득했는지 알아야 하므로 num type을 이용한다.
// 2) 공유 변수값이 최대값 NUM 이상이면 스핀하며 대기한다.
// 3) NUM 미만이면 공유 변수값을 아토믹하게 증가한다.
// 4) 증가한 공유 변수값이 NUM 이하인지 검사하여 이하라면 루프를 벗어나 락을 얻는다.
// 5) 아니라면 여러 프로세스가 동시에 락을 획득한 것이므로 공유 변수값을 감소하고 다시 시도한다.
// 6) 락을 반환한다. 공유 변수값을 아토믹하게 감소시킨다.
//
// 세마포어는 물리적인 계산 리소스 이용에 제한을 적용하고 싶은 경우 등에 이용할 수 있다. 항공기 등의 이용은 좌석 수에
// 제한이 있기 때문에 이용자 수에 제한을 거는 것과 같다. 당연하지만 주의할 점은 세마포어에서는 여러 프로세스가 lock을
// 획득할 수 있으므로 Mutex에서는 피할 수 있었던 시뮬레이션을 피할 수 없는 경우가 많으므로 주의해야한다.
// 다음 코드는 세마포어 이용 례
pub fn some_func4() {
    let mut cnt = AtomicUsize::new(0); // 공유 변수라 가정
    loop {
        cnt = semaphore_acquire(cnt);
        // Critical Section
        cnt = semaphore_release(cnt);
        break;
    };
}
// 이용 방법은 mutex와 같으며 락 반환을 잊지 않도록 주의하자
//
//
// 3.4.1 LL/SC 명령을 이용한 구현
// LL/SC 명령을 이용한 세마포어 구현을 알아보자. 위의 semaphore_acquire, release 예제에서는 락 획득을 실패한
// 경우에도 아토믹하게 공유 변수를 감소시켜야 했는데, 이는 락 획득 시 값을 검사하지 않고 아토믹하게 증가시켰기 때문이다.
// 한편 LL/SC 명령을 이용하면 공유 변수를 검사해 필요한 경우에만 증가시키는 처리를 아토믹하게 수행할 수 있으므로
// semaphore_acquire 함수 안에서 감소 처리할 필요가 없다.
// AArch64의 LL/SC 명령을 이용한 세마포어의 락 획득 함수부터 살펴보자(page 102)
//
//
// 3.4.2 POSIX 세마포어
// 세마포어의 표준 구현인 POSIX 세마포어를 살펴보자.
// page 102 - 105
// POSIX 세마포어예는 named semaphore와 unnamed semaphore가 있음. named semaphore는 '/'로 시작해 null문자열로
// 끝나는 문자열로 식별되며, 이 문자열은 OS 전체에 적용되는 식별자가 된다.
// named semaphore는 위 페이지의 예제와 같이 파일로 공유 리소스를 지정할 수 있으며 생성과 열기, 닫기와 파기를 수행한다.
// (sem_close로 닫는 것은 핸들러를 닫는 것 뿐이므로 OS측에는 세마포어용 리소스가 남아 있음. 이를 완전히 삭제하려면
// sem_unlink함수를 호출해서 삭제). 그렇기 때문에 named semaphore를 이용하면 메모리를 공유하지 않는 프로세스
// 사이에서도 편리하게 세마포어를 구현할 수 있음. 한편 unnamed semaphore를 생성하면 공유 메모리 영역이 필요하며
// 공유 메모리상에 sem_init으로 생성하고, sem_destroy로 파괴한다.


// 3.5 조건 변수
// 어떤 조건을 만족하지 않는 동안에는 프로세스를 대기 상태로 두고, 조건이 만족되면 대기 중인 프로세스를 실행하고 싶을
// 때가 있음(굉장히 빈번함). 실생활의 규칙적으로 신호에 따라 움직이는 신호등에서 이 신호에 해당하는 것을 concurrency
// programming에서는 조건 변수라고 부르며 조건 변수를 기반으로 '프로세스의 대기를 수행'함.
// 106p의 조건 변수를 C로 풀어낸 예제가 있으니 살펴보자.
// Pthreads에서가 아닌 커스텀 조건변수 ready를 정의하는 이유? producer 함수를 이용한 데이터 생성이 consumer
// 스레드 생성 이전에 실행될 가능성이 있기 때문. Pthreads의 wait는 의사 각성이(spurious wakeup)라는 불리는 현상이 일어날 가능성이 있음.
// 4.5절에서 보게 되겠지만 그전에 구글링해보자.
// spurious wakeup은 아무 이유 없이 깨어난 것처럼 보이기 때문에 위와 같은 이름으로 불리지만, 실제로는 이유가 있음.
// 일반적으로 조건 변수가 신호를 받은 타이밍과 대기 중인 스레드가 마지막으로 실행될 타이밍 사이에 다른 스레드가
// 실행되어 조건을 변경했기 때문에 발생함. 예를 들어 멀티 프로세스 시스템에서 signal을 받았을 때 조건 변수에서 대기 중인
// 스레드가 여러 개 있는 경우 시스템이 스레드를 모두 깨우기로 결정하고 하나의 스레드를 깨우는 broadcast로 모든 signal을
// 처리해 signal, wakeup의 1:1 관계가 깨짐. 10개의 스레드가 대기 중인 경우 하나만 스레드 경합조건을 이겨 깨어나고
// 나머지 9개는 spurious wakeup을 경험하게됨.
//
// 조건 변수에서 중요한점?
// 1) 조건 변수로의 접근은 반드시 락을 획득한 후에 수행해야 한다는 것
// 2) pthread_cond_t type의 조건 변수 외에도 실행 가능 여부를 나타내는 조건 변수를 준비해야 한다는 것(조건 변수가 여러개 필요).
// 책에서 구현한 예제와 같이 producer-consumer 모델은 변수로의 접근 주체가 명확하게 되므로 일반적으로 공유 변수로의
// 접근의 상태관리 복잡성이 어느정도 해결되어 간략하게 구현할 수 있음.


// 3.6 배리어 동기(barrier synchronization)
// 단체 생활의 이동을 생각해 보자. 이동은 반드시 클래스 전체가 모였는지 확인한 후 진행한다. 이렇게 모두 모인 후에
// 실행 동기를 구현하는 것이 barrier synchronization이다.
//
//
/// 3.6.1 spinlock 기반 barrier synchronization
/// 배리어 동기의 개념?
/// 1) 공유 변수를 준비하고, 프로세스가 어던 지점에 도달한 시점에 해당 공유 변수를 증가 시킴.
/// 2) 공유 변수가 계속 증가되어 어떤 일정한 수에 도달하면 배리어를 벗어나 처리를 수행.
/// 간단하다. 한 클래스의 학생이 30명이라고 할 때 각 학생이 공유 변수를 준비하고, 준비가 되면 각자 공유 변수를 증가하고
/// 그 값이 30이 되면 이동을 시작하는 것과 같음.
// fn barrier01(mut cnt: AtomicUsize, mut max: AtomicUsize) { // 1
//     cnt.fetch_add(1, Ordering::SeqCst); // 2
//     while cnt < max {}; // 3
// }
// 1) 공유 변수에 대한 값 cnt와 최댓값 max를 받음.
// 2) 공유 변수 cnt를 아토믹하게 증가시킴.
// 3) cnt가 가리키는 값이 max가 될 때까지 대기.
// 배리어 동기 예제는 110p 참고


// 3.6.2 Pthreads를 이용한 배리어동기
// 스핀락을 이용한 배리어 동기에서는 대기 중에도 루프 처리를 수행하므로 불필요하게 CPU리소스를 소비함.
// 그러므로 Pthreads의 조건 변수를 이용해 배리어 동기를 수행하는 방법을 알아보자.
// 111p의 예제 참고. spinlock 버전과의 차이는 cnt값이 max가 될때까지 루프를 돌리는 것이 아닌 대기한다는 것


// 3.7 Readers-Writer락
// 레이스 컨디션이 발생하는 근본적인 원인? Write 처리 때문. 쓰기만 배타적으로 수행한다면 문제가 발생하지 않음.
// Mutex와 Semaphore에서는 프로세스에 특별한 역할을 설정하지 않았지만 Readers-Writer락(RW락)에서는 읽기만
// 수행하는 프로세스(Reader)와 쓰기만 수행하는 프로세스(Writer)로 분류하고 다음 제약을 만족하도록 베타제어를 수행한다.
// - lock을 획득 중인 Reader는 같은 시각에 다수(0 이상) 존재할 수 있다.
// - lock을 획득 중인 Writer는 같은 시각에 1개만 존재할 수 있다.
// - Reader와 Writer는 같은 시각에 락 획득 상태가 될 수 없다.
// *NOTE_ Readers-Writer락은 Reader-Writer락이나 Read-Write락으로도 표기하니 새로운 의문을 갖지 않아도됨.
// Rust에는 Reader-Writer락, Pthreads 매뉴얼에는 Read-Wrtie락으로 표기되어 있음.
// 이 책에서는 Reader가 다수라는 것을 명확하게 하기 위해 Readers-Writer락 이라고 표기할 것이라고 함!!
//
//
// 3.7.1. 스핀락 기반 RW락
// 스핀락 기반의 RW락 알고리즘(112p-114p).
// Reader수를 나타내는 rcnt(초기값 0), Writer 수를 나타내는 wcnt(초기 0), Writer용 락 변수 lock(초기값 false)의
// 3개 공유 변수를 이용해 베타제어를 수행하는 알고리즘이다. 또한 Reader용 락 획득과 반환 함수, Writer용 락 획득과
// 반환함수는 별도의 인터페이스로 되어 있어 실제 이용할 때는 공유 리소스의 읽기만 수행할지 쓰기만 수행할지 판단해서 이용해야함.
// RW락을 사용해야 할 상황은 대부분 읽기 처리이며, 쓰기는 거의 읽어나지 않을 것임. 위의 페이지에서 소개한 알고리즘은
// Writer를 우선하도록 설정되어 있으므로 그런 상황에서는 잘 작동하지만 쓰기가 빈번하게 일어난다면 읽기를 전혀 실행하지
// 못하게 되므로 주의해야함. 쓰기도 많이 수행되는 처리인 경우에는 뮤텍스를 이용하는 편이 실행 속도와 안정성 측면에서 좋다.
// RW락 사용예제 114p 참고
// 사용 방법은 뮤텍스와 거의 동일하지만 구현할 때는 읽기만의 처리인지 또는 쓰기도 수행하는 처리인지 파악해야 함.
//
//
// 3.7.2 Pthreads의 RW락
// Pthreads에서도 RW락용 API를 제공함. 115p 참고
//
// 3.7.3 실행 속도 측정
// RW락의 실행 속도 측정, 락의 실행 속도를 비교하는 코드 작성. 락을 획득해 HOLDTIME만 루프를 해제해서 락을 해제하는
// 작동을 수행하는 worker thread를 N개 실행하고, 이 일련의 작동을 지정한 시간 동안 몇 번 수행할 수 있는지 측정
// 116p - 121p 참조


// 3.8 Rust 동기 처리 라이브러리
// Rust에서는 기본적인 동기 처리 라이브러리를 표준 라이브러리(std::sync)로 제공함. 러스트의 동기 처리 라이브러리는
// 크리티컬 밖에서의 보호 대상 객체의 접근과 락 미해제를 타입시스템으로 방지하는 특징을 갖고 있음!!!!!!
//
/// 3.8.1 Mutex
fn some_func5(lock: Arc<Mutex<u64>>) { // 2
    loop {
        // 락을 하지 않으면 Mutex type 안의 값은 참조 불가
        let mut val = lock.lock().unwrap(); // 3
        *val += 1;
        println!("{}", *val);
    }
}

pub fn my_func() {
    // Arc는 thread safe한 rc 타입의 스마트 포인터(Atomic reference counter)
    // rc가 선행되어야 함 https://rinthel.github.io/rust-lang-book-ko/ch15-04-rc.html
    let lock0 = Arc::new(Mutex::new(0)); // 4

    // 참조 카운터가 증가될 뿐이며 내용은 클론되지 않음.
    let lock1 = lock0.clone(); // 5

    // thread 생성, 클로저 내 변수로 이동
    let th0 = thread::spawn(move || { //
        some_func5(lock0);
    });

    // thread spawn, 클로저 내 변수로 이동
    let th1 = thread::spawn(move || {
        some_func5(lock1);
    });

    // 약속(rust에선 future)
    th0.join().unwrap();
    th1.join().unwrap();
}
// 2) Arc<Mutex<u64>> type의 값을 받는 스레드용 함수
// 3) lock 함수를 호출해 락을 걸어 보호 대상 데이터의 참조를 얻는다(락을 걸지 않으면 얻을 수 없음)
// 4) Mutex용 변수를 저장하는 스레드 세이프한 RC type의 스마트 포인터를 생성. Mutex용 변수는 이미 값을 저장하고
//    있으므로 초기 값을 0으로 설정
// 5) Arc type의 값은 클론해도 RC만 증가될 뿐, 내부 데이터는 복사되지 않는다.
// 6) move 지정자는 클로저 안의 변수 캡처 방법을 지정함. move가 지정되면 소유권이 이동하고, 지정되지 않으면 참조가
//    전달됨.
// Rust에서 Mutex용 변수는 보호 대상 데이터를 보존하도록 되어 있어 락을 하지 않으면 보호 대상 데이터에 접근할 수 없다.
// C에서는 보호 대상 데이터는 락을 하지 않아도 접근할 수 있지만 그런 코드는 레이스 컨디션이 될 가능성이 있다. 한편
// Rust에서는 이와 같이 컴파일 시에 공유 리소스로의 부정한 접근을 방지할 수 있도록 설계되어 있다. 또한 보호 대상
// 데이터가 스코프를 벗어나면 자동으로 락이 해제된다. 그러므로 Pthreads로 발생한 락의 취득과 해제를 잊어버리는 것을
// 방지할 수 있다.
// lock 함수는 LockResult<MutexGuard<'_, T>>라는 type을 반환하며, LockResult type의 정의는 아래와 같음.
// type LockResult<Guard> = Result<Guard, PoisonError<Guard>>;
// 즉, 락을 획득할 수 있는 경우에는 MutexGuard라는 type에 보호 대상 데이터를 감싸 반환하고, 이 MutexGuard 변수의
// 스코프를 벗어날 때 자동으로 락을 해제(rust의 drop)하는 구조가 구현되어 있음.
// 또한 어떤 스레드가 락 획득 중에 패닉에 빠지는 경우 해당 뮤텍스는 poisoned 상태에 있따고 간주되어 락 획득에 실패함.
// 여기서는 이 체크를 간단하게 unwrap으로 실행하며, 락을 획득할 수 없는 경우(unwrap 실패시 panic!) 종료하도록 했다.
// lock의 유사함수로 try_lock 함수가 있음. try_lock 함수는 락의 획득을 시험해서 획득가능하면 락을 걸지만
// 그렇지 않으면 처리를 되돌림. 이와 같은 함수는 Pthreads에도 있음.
//
/// 3.8.2 조건 변수
/// Rust의 조건 변수는 Condvar type이며, 이용 방법은 Pthreads의 경우와 거의 같다. 락을 획득한 뒤 조건 변수를
/// 이용해 wait 또는 notify를 수행함.
use std::sync::Condvar; // 1

// Condvar type의 변수가 조건 변수이며 Mutex와 Condvar를 포함하는 튜플이 Arc에 포함되어 전달된다.
fn child(id: u64, p: Arc<(Mutex<bool>, Condvar)>) { // 2
    let &(ref lock, ref cvar) = &*p;
    // & vs ref
    // In patterns, & destructures a borrow, ref binds to a location by-reference rather than by-value.
    // https://users.rust-lang.org/t/ref-keyword-versus/18818

    // 먼저 Mutex lock을 수행한다.
    let mut started = lock.lock().unwrap(); // 3
    // while !*started { // Mutex 안의 공유 변수가 false인 동안 루프
    //     // wait으로 대기
    //     started = cvar.wait(started).unwrap(); // 4
    // }

    // 다음과 같이 wait_while을 사용할 수도 있음.
    cvar.wait_while(started, |started| !*started).unwrap();

    println!("child {}", id);
}

fn parent(p: Arc<(Mutex<bool>, Condvar)>) { // 5
    let &(ref lock, ref cvar) = &*p;

    // 먼저 뮤텍스락을 수행한다. 6
    let mut started = lock.lock().unwrap();
    *started = true; // 공유 변수 업데이트
    cvar.notify_all(); // 알림
    println!("parent");
}

pub fn some_func6_125p() {
    // Mutex와 cond var 작성
    let pair0 = Arc::new((Mutex::new(false), Condvar::new()));
    let pair1 = pair0.clone();
    let pair2 = pair0.clone();

    let c0 = thread::spawn(move || { child(0, pair0) });
    let c1 = thread::spawn(move || { child(1, pair1) });
    let p = thread::spawn(move || { parent(pair2) });

    c0.join().unwrap();
    c1.join().unwrap();
    p.join().unwrap();
}
// 2) 대기 스레드용 함수 정의. 스레드 고유의 번호를 받는 id 변수 및 Mutex type 변수와 Condvar type 변수의 튜플을
//    Arc로 감싼 값을 받음.
// 3) Arc type 내부에 포함된 Mutex 변수와 조건 변수를 꺼낸다.
// 4) 알림이 있을 때까지 대기한다.
// 5) 알림 스레드용 함수
// 6) 락을 한 뒤 공유 변수값을 true로 설정하고 알림
// child 함수 내부에서는 Mutex로 보호된 논리값이 true가 될 때까지 루프를 돈다. notify하는 스레드가 먼저 실행된
// 경우 및 의사 각성에 대처하기 위해서임. wait_while 함수에서는 두 번째 인수로 전달되는 술어(값)가 false가 될 때까지
// 대기한다(As long as the value inside the `Mutex<bool>` is `true`, we wait.). wait 계열의 함수도
// lock함수와 마찬가지로 대상 Mutex가 poisoned 상태가 되었을 때 실패하며, 여기서는 unwrap으로 대처.
//
// 대기 함수에는 타임아웃 가능한 wait_timeout 계열 함수도 있으며, 이 함수에서는 wait하는 시간을 지정할 수 있음.
// 즉, 지정한 시간 내에 다른 스레드로부터 notify가 없는 경우 해당 함수는 대기를 종료하고 반환한다. 타임아웃 가능한
// wait함수에 관해서는 매뉴얼 등을 참조해보자.
//
//

/// 3.8.3 RW락
/// Rust의 RW락은 Mutex와 거의 같으므로 일단은 락 구조만 간단히 살펴보자
use std::sync::RwLock; // 1

pub fn some_func7_126p() {
    let lock = RwLock::new(10); // 2
    {
        // immutable 참조를 얻음 3
        let v1 = lock.read().unwrap();
        let v2 = lock.read().unwrap();
        println!("v1 = {}", v1);
        println!("v2 = {}", v2);
    }

    {
        // mutable 참조를 얻음 4
        let mut v = lock.write().unwrap();
        *v = 7;
        println!("v = {}", v);
    }
}
// 2) RW락용 값을 생성하고, 보호 대상 값의 초깃값인 10을 지정한다.
// 3) read 함수를 호출해 Read락을 수행. Read락은 몇번이든 수행할 수 있음.
// 4) write 함수를 호출해 Write락을 수행.
// Rust에선 간단하게 read는 immutable, write는 mutable이라고 보면 될 듯 하다.
// Read락을 수행하는 read함수를 호출하면 뮤텍스락과 마찬가지로 보호 대상 이뮤터블 참조(RwLockReadGuard type으로
// 감싼 참조)를 얻을 수 있으며, 이 참조를 통해 값에 읽기 접근만 가능하게 됨. 뮤텍스락과 마찬가지로 이 참조의 스코프를
// 벗어나면 자동적으로 Read락이 해제됨.
// write함수의 경우 보호 대상 mutable 참조(RwLockWriteGuard type으로 감싼 참조)를 얻을 수 있다. 그러므로
// 보호 대상 데이터에 쓰기와 읽기 접근 모두 가능. RW락에도 뮤텍스와 마찬가지로 try계열 함수가 있음. 메뉴얼 참조
//
//
/// 3.8.4 배리어 동기
/// Rust에는 배리어 동기용 표준 라이브러리도 있음.
use std::sync::Barrier; // 1

pub fn some_func8_127p() {
    // 스레드 핸들러를 저장하는 벡터
    let mut v = Vec::new(); // 2

    // 10 스레드 만큼의 배리어 동기를 Arc로 감쌈
    let barrier = Arc::new(Barrier::new(10)); // 3

    // 10 스레드 실행
    for _ in 0..10 {
        let b = barrier.clone();
        let th = thread::spawn(move || {
            b.wait(); // 배리어 동기 4
            println!("finished barrier");
        });
        v.push(th);
    }

    for th in v {
        th.join().unwrap();
    }
}
// 2) 나중 join을 수행하기 위해 스레드 핸들러를 보존하는 벡터를 정의. 이 Vec type은 동적 배열 객체를 다루는 데이터 컨테이너(힙 데이터 컨테이너).
// 3) 배리어 동기용 객체를 생성. 인수 10은 10스레드로 promise(future)를 수행하기 위해서임.
// 4) 배리어 동기
//
//
/// 3.8.5 세마포어
/// Rust에서는 세마포어를 표준으로 제공하지 않음. 그렇지만 Mutex와 Condvar를 이용해서 세마포어를 구현할 수 있음.
/// Semaphore type을 정의하고 그 type으로 세마포어용 함수인 wait와 post함수를 구현해보자.
use std::time::Duration;
pub struct Semaphore {
    mutex: Mutex<isize>,
    cond: Condvar,
    max: isize,
}

impl Semaphore {
    pub fn new(max: isize) -> Self { // 2
        Semaphore {
            mutex: Mutex::new(0),
            cond: Condvar::new(),
            max,
        }
    }

    pub fn wait(&self) {
        // 카운터가 최대값 이상이면 대기 3
        let mut cnt = self.mutex.lock().unwrap();
        while *cnt >= self.max {
            cnt = self.cond.wait(cnt).unwrap();
        }

        // 다음과 같이 wait_while을 사용할 수도 있음.
        cnt = self.cond.wait_while(cnt, |cnts| *cnts >= self.max).unwrap();
        // let (mut cnt, result) = self.cond.wait_timeout_while(
        //     cnt,
        //     Duration::from_millis(100),
        //     |cnts| *cnts >= self.max
        // )
        //     .unwrap();


        *cnt += 1; // 4
        // println!("critical section")
    }

    pub fn post(&self) {
        // 카운터 감소 5
        let mut cnt = self.mutex.lock().unwrap();
        *cnt -= 1;
        if *cnt <= self.max {
            self.cond.notify_one();
        }
    }
}
// 1) Semaphore type 정의, Mutex와 상태 변수 및 동시에 락을 획득할 수 있는 프로세스의 최대 수를 저장.
// 2) 초기화 시에 동시에 락을 획득할 수 있는 프로세스의 최대 수를 설정
// 3) 락을 해서 카운터가 최대값 이상이면 조건 변수의 wait함수로 대기함.
// 4) 카운터를 증가한 뒤 Critiacal Section으로 이동함.
// 5) 락을 해서 카운터를 감소, 이후 카운터가 최대값 이하면 조건 변수로 대기 중인 스레드에 알림.
// 이렇게 Semaphore type의 변수는 현재 Critical Section을 실행 중인 프로세스 수를 세고, 그 수에 따라 대기나
// 알림을 수행함. 카운터의 증가와 감소는 Mutex로 락을 획득한 상태에서 수행되므로 배타적 실행을 보증함.
// 세마포어의 코드를 테스트해보자
const NUM_LOOP: usize = 100000;
const NUM_THREADS: usize = 8;
const SEM_NUM: isize = 4;

static mut CNT: AtomicUsize = AtomicUsize::new(0);

pub fn some_func9_129p() {
    let mut v = Vec::new();
    // SEM_NUM만큼 동시 실행 가능한 세마포어
    let sem = Arc::new(Semaphore::new(SEM_NUM));

    for i in 0..NUM_THREADS {
        let s = sem.clone();
        let t = thread::spawn(move || {
            for _ in 0..NUM_LOOP {
                s.wait();

                // atomic하게 증가 및 감소
                unsafe { CNT.fetch_add(1, Ordering::SeqCst) };
                let n = unsafe { CNT.load(Ordering::SeqCst) };
                println!("semaphore: i = {}, CNT = {}", i, n);
                assert!((n as isize) <= SEM_NUM);
                unsafe { CNT.fetch_sub(1, Ordering::SeqCst) };

                s.post();
            }
        });
        v.push(t);
    }

    for t in v {
        t.join().unwrap();
    }
}
// 여기서는 스레드를 NUM_THREADS(8)만큼 만들고, SEM_NUM(4) 스레드만큼 동시에 Critical Section을 실행할 수 있는
// Semaphore를 만들었음. 그러므로 wait과 post 사이는 4 스레드 이내로 제한됨. 확인해보자.
// AtomicUsize 아토믹 변수를 이용해 스레드 안에서 증가와 감소를 수행해 그 수를 확인한다. fetch_add, fetch_sub
// 명령으로 아토믹 덧셈 뺄셈을 할 수 있으며 load가 읽기 명령. Ordering::SeqCst는 메모리 배리어 방법을 의미,
// SeqCst는 가장 제한이 엄격한(순서를 변경할 수 없는) 메모리 베리어 지정이 된다. 메모리 배리어는 4.7절에서 더 자세히 보자.
// 이 테스트 코드를 실행하면 SEM_NUM보다 CNT값이 커졌을 때 assert 매크로가 실패해야 하지만 그런 일은 일어나지 않음.
// 세마포어를 이용하면 queue의 크기가 유한한 채널을 구현할 수 있다. channel은 프로세스 사이에서 메시지 교환을 수행하기
// 위한 추상적인 통신로다. Rust에서는 채널이 송신단과 수신단으로 나뉘어 있으므로 그에 맞춰 구현해보자.
/// 다음은 송신단용 Sender type
use std::collections::LinkedList;


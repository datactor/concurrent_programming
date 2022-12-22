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
// 3.2.1 Compare and Swap
// CAS(Compare and Swap)은 동기 처리 기능의 하나인 세마포어(semaphore), lock-free, wait-free한 데이터 구조를
// 구현하기 위해 이용하는 처리다. CAS의 의미를 나타낸 예
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
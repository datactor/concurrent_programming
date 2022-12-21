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
//                          read v   wrtie (v+1)      write (v+1)
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
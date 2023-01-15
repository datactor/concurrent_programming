// ch06
// 실제 CPU, 특히 프로세스 수가 CPU 수보다 많은 상황에서 물리적으로 어떻게 작동할까?
// 멀티태스크와 동시성은 거의 같은 의미이며 멀티태스크는 프로세스를 동시에 작동시킨다.
// 멀티태스크 또는 멀티태스킹이란 여기서는 이들을 단일 CPU상에서 여러 프로세스를 동시에 작동시키기 위한 기술을
// 나타내는 것으로 설명한다.
// 멀티태스크, 멀티태스킹의 개념적의미를 명확히 하고, 주변 용어를 숙지하자.
// 그 뒤 Rust를 이용해 AArch64 아키텍처를 대상으로 하는 유저랜드 구현 스레드(green thread)를 구현해보자.
// 이 구현에서는 간소하기는 하지만 OS 프로세스, 스레드, Erlang이나 Go의 작동 원리를 명확하게 이해할 수 있을 것이다.
// 마지막으로 앞서 작성한 green thread 상에서 간단한 actor model을 구현해보자.


// 6.1 multi-task
// 6.1.1 지킬박사와 하이드
// 다중인격을 가진 지킬 박사는 어느 날 자신의 정신을 선을 대변하는 지킬과 악을 대변으로 하이드로 나누는데 성공하지만
// 결과적으로 비극을 맞게 된다. 여기에서는 의학적인 관점에서 인체와 뇌의 기저에 관해 설명하기보다는 이런 다중 인격을
// 어떻게 구현할지에 관한 관점에서 상상해보자.
// 다음 그림은 뇌의 기억 영역에 읽기 쓰기를 할 수 있는 기계 즉, 뇌 IO 장치를 연결한 모습이다.
// 뇌 IO 장치 = [[지킬용 메모리], [하이드용 메모리]]
// 뇌 IO 장치를 이용하면 외부 기억 장치와 뇌 사이에서 기억을 읽고 쓸 수 있다고 가정하자. 이 뇌 IO 장치에 외부 기억 장치로
// 지킬의 메모리와 하이드의 메모리가 연결되어 있다. 지킬과 하이드의 인격을 교대할 때는 일단 뇌에서 작동하고 있던 현재의
// 인격을 외부 저장 장치(ssd or hdd)에 저장한 뒤 다른 인격을 외부 기억장치에서 읽어 뇌에 쓴다고 생각할 수 있다.
// 어떻게 하면 이 뇌 IO 장치를 이용해 인격을 교대할 수 있는지 생각해보자. 다음 그림은 인격 교대의 runtime 예다.
//                        지킬 저장               지킬 복원
//  지킬용 메모리  ------------------------------------------------->
//                            ↑                    |
//                 (식사중)    |                    |
//                지킬 활동 중  |                    ↓ 지킬 활동 중
//            뇌 -------------->------------------->-------------->
//                             ↑   하이드 활성 중   |    └>식사 중이었을 텐데?(식사중이던 지킬이 놀고 있어 놀람)
//                             |    (놀이 시작)    |
//                             |                 ↓
// 하이드용 메모리 ------------------------------------------------->
//                         하이드 복원         하이드 저장
// 그림 6-2 지킬 박사와 하이드의 런타임
//
// 그림에서는 먼저 지킬이 활동 중으로 식사하고 있고 식사 도중에 인격 교대가 일어난다. 인격을 교대하려면 우선
// 1) 뇌의 정보를 지킬용 메모리에 저장하고,
// 2) 이후 하이드용 메모리를 복원한다.
// 다시 지킬로 교대하려면
// 3) 뇌의 정보를 하이드용 메모리에 저장하고,
// 4) 이후 지킬용 메모리를 복원한다.
//
// 다음은 컴퓨터에서 앞의 지킬 앤 하이드를 실제로 구현한 예이다.
//
//                       레지스터 저장            레지스터 복원
// 프로세스 A의 메모리 ------------------------------------------------->
//                              ↑                    |
//                              |                    |
//                   프로세스 A  |                    ↓   프로세스 A
//              CPU ------------>------------------->-------------->
//                               ↑     프로세스 B   |
//                               |                |
//                               |                ↓
// 프로세스 B의 메모리 ------------------------------------------------->
//                          레지스터 복원      레지스터 저장
// 그림 6-3 context switch
//
// 뇌의 정보에 해당하는 CPU의 정보는 레지스터의 값이 된다. 즉, 어떤 프로세스가 CPU에서 실행 중일 때 레지스터를 메모리에
// 저장함으로써 프로세스의 특정 시점의 상태가 저장된다. 그리고 저장한 레지스터를 CPU로 복원하면 저장했던 상태로 되돌릴 수 있다.
// 이런 레지스터(또는 스택 정보) 등의 프로세스 상태에 관한 정보를 context라 부르며, context의 저장과 복원이라는
// 일련의 처리를 context switch라 부른다. context switch는 간단히 다음과 같이 정의할 수 있다.
// - 정의 context switch: 어떤 프로세스에서 다른 프로세스로 실행을 전환하는 것
//
// 우리가 평소 사용하고 있는 컴퓨터나 스마트폰 등의 CPU 수는 몇개 또는 몇십개 정도이지만, 실행할 수 있는 애플리케이션의 수는
// CPU 수보다 훨씬 많이 실행할 수 있다. 이것은 OS가 OS 프로세스의 context switch를 빈번하게 수행해 앱 전환을
// 하고 있기 때문이다. context switch를 전혀 수행하지 않는 OS도 존재하는데 이런 OS는 싱글 태스크 OS라 부르며
// context switch를 수행하는 여러 OS 프로세스를 동시에 작동시키는 것이 가능한 OS는 멀티 태스크 OS라 부른다.
// 윈도우, 리눅스, BSD 계열 OS 등 주류 OS는 multi-task OS이고, 싱글 태스크 OS는 윈도우의 전신인 MS-DOS가 유명하다.
//
// multi-task가 가능한 실행 환경이란 여러 프로세스를 실행할 수 있는 환경으로 명백하지만, 이런 실행 환경을 만드는 것은
// 잘 생각해보면 사실 어렵다. 예를 들어 CPU가 4개인 환경에서 최대 4개까지의 프로세스를 동시에 실행할 수 있는 4코어 OS가
// 있다고 하자. 이 OS는 여러 프로세서를 실행할 수 있지만 실질적으로 싱글태스크 OS와 다르지 않다. 또한 3개, 4개, 5개의
// 프로세스를 충분히 생성할 수 있지만, last in, first out과 같이 가장 마지막에 생성한 프로세스가 종료될 때까지
// 이전에 생성된 프로세스가 실행되지 않는 경우에도 실질적으로는 싱글 태스크 OS와 다르지 않다(동시에 실행하는 프로세스의
// 개수가 하나라면 여러개 실행하더라도 싱글태스크와 다름 없다). 그래서 이 책에서는 multi-task 실행 환경을 다음과 같이
// 정의한다. 실행 환경이란? OS, 의사 머신, 언어 처리 시스템을 총칭한다.
// - 정의 multi-task 실행 환경: 어떤 실행 환경이 multi task 가능하다. <-> 임의의 시점에서 새로운 프로세스를
//   생성할 수 있고, 계산 도중 상태에 있는 프로세스가 공평하게 실행된다.
//
// 공평성은 다음 두 가지로 정의한다.
// - 정의 weak 공평성: 어떤 실행 환경이 약한 공평성을 만족한다. <-> 어떤 프로세스가 특정 시각 이후 실행 가능한
//   대기 상태가 되었을 때 그 프로세스가 실행된다.(메모리 엑세스 순서에 대한 보장이 없는 메모리 모델. 메모리
//   엑세스가 잘못된 순서로 발생하거나 특정 메모리 엑세스가 완전히 손실될 수 있음을 의미함)
//
// - 정의 strong 공평성: 어떤 실행 환경이 강한 공평성을 만족한다. <-> 어떤 프로세스가 특정 시각 이후 실행 가능한
//   대기 상태와 실행 불가능한 대기 상태의 전이를 무한히 반복할 때 최종적으로 그 프로세스는 실행된다.
//   (메모리 엑세스 순서에 대한 보장이 있는 메모리 모델. 메모리 엑세스가 항상 요청된 순서대로 발생하는 것으로 손실되지
//   않음. std::sync::atomic 및 std::sync::atomic_weak 모듈을 통해 지원됨)
// 이들 정의는 선형 시제 논리(Linear Temporal Logic; LTL 또는 Propositional Temproal Logic; PTL이라고도 부름)
// 로 정식화된다. 선형 시제 논리는 논리학에서 선형 이산 시간에 대한 여러 양상을 갖춘 시제 논리중 하나다. 예를 들어보자
// 약한 공평성: 어떤 시각 이후 특정 시점에 놀러 가고 싶다고 생각하는 사람이 있을 때 그 사람을 데리고 갈 수 있는 환경.
// 강한 공평성: 어떤 시각 이후 특정 시점에 놀러 가고 싶지만 다른 시점에는 집안에 틀어박혀 있고 싶어 하는 귀찮은 사람
//            이라도 언젠가는 밖으로 데리고 놀러 갈 수 있는 환경이다.
// 현실적인 시스템에서는 약한 공평성 구현이 필수이며 강한 공평성을 실현하는 것은 쉽지 않다.
// 한편 현실의 구현에서는 latency와 CPU 시간의 배분이라는 관점도 포함해서 공평성에 관해 논의되고 있다. 예를 들어
// 리눅스 커널 버전 2.6.33 이후에서는 완전히 공평한 스케줄링(Completely Fair Scheduling; CFS)라는 프로세스
// 실행 방법을 채택하여 각 프로세스가 공평하게 CPU 시간을 소비 가능하게 변경되었다. 리눅스는 스케줄링 방식을 몇 가지
// 선택할 수 있으며, IO의 deadline 근방의 process를 우선하는 스케줄러 등도 선택할 수 있다. 이렇게 현실적인 시스템에서
// 공평성을 논할 때는 실행 가능성은 물론 리소스 소비의 관점도 고려해야 한다.


// 6.1.2 협조적/비협조적 multi-task
// context switch란 CPU상에서 프로세스의 전환을 수행하는 것이다. context switch를 수행하는 전략으로 협조적, 비협조적
// 수행 방법이 있다. 협조적 전략은 프로세스 자신이 자발적으로 context switch 전환을 수행하는 방법이고, 비협조적 전략은
// allocation 등 외부적인 강제력에 의해 context switch를 수행하는 방법이다. 협조적으로 context switch를 수행하는
// multi-task를 협조적 멀티태스크, 비협조적으로 context switch를 수행하는 multi-task를 비협조적 multi-task라 부른다
// 협조적 multi-task: 프로세스 자신이 능동적으로 context switch 전환을 수행하는 multi-task
// 비협조적 multi-task: allocation 등 외부적인 강제력에 의해 수동적으로 context switch 전환을 수행하는 multi-task
// - 정의 협조적 multi-task: 각각의 프로세스가 자발적으로 context switch를 수행하는 multi-task 방식
// - 정의 비협조적 multi-task: 프로세스와 협조 없이 외부적인 작동에 따라 context switch를 수행하는 multi-task 방식
//
// cooperative multi-tasking은 비선점적 멀티태스킹(non-preemptive multi-tasking)
// non-cooperative multi-tasking은 선점적 멀티태스킹(preemptive multi-tasking)이라 부르기도 한다.
// 선점하다(preempt)라는 단어는 TV의 채널을 바꾸는 등의 의미를 가지고 있어서 이런 이름을 사용하게 된 것이라 생각된다.
// 이와 관련된 용어인 선점(preemption)은 다음과 같이 정의한다.
// - 정의 선점(preemption): process와의 협조 없이 수행하는 context switching
// process의 context switch 방법을 결정하기 위한 module, func, process를 scheduler라 부르며 간단히 다음과
// 같이 정의한다.
// - 정의 scheduler: context switch 전략을 결정하는 process, module, func 등
// scheduler는 공평성을 고려해 다음에 실행할 프로세스를 결정하고, non-cooperative multi-tasking인 경우에는
// 어떤 타이밍에 preemption을 수행할 것인지도 결정해야 한다. scheduler가 process의 실행 순서를 결정하는 것을
// scheduling이라 부른다.
//
// cooperative multi-tasking의 장점과 단점
// 협조적 multi-task의 장점:
// 1) multi-task mechanism을 쉽게 구현할 수 있다는 것을 들 수 있다. 그렇기 때문에
//    초기 multi-tasking OS 대부분이 이를 채용했다. 예를 들어 윈도우 3.1이나 classic mac용 OS에서는
//    협조적 multi-tasking 방식이 채용되어 있다. 그리고 Rust나 Python의 async/await이라는 mechanism은
//    협조적 multi-tasking방식의 일종으로 현재도 이용되고 있다.
// 협조적 multi-task의 단점:
// 1) process가 자발적으로 context switch를 수행해야 한다는 것이다. 예를 들어 어떤 process에 버그가 있어
//    context switch를 수행하지 않고 무한 loop에 빠지거나 정지하게 되면 그 process는 계산 resource를 점유하게 된다.
//    그렇기 때문에 앱 개발자는 협조적 multi-task를 인식하고 이런 버그가 없도록 구현해야 한다. 윈도우 3.1이나
//    classic mac의 경우에는 앱이 깨지면 OS 전체가 깨지며 PC를 재기동해야 하는 일이 빈번하게 일어났다.
//
// Rust나 Python에서의 async/await을 이용한 구현도 같은 문제를 안고 있으며, 이 mechanism을 이용해 구현해도
// 무한 loop가 되거나 처리를 정지시키는 함수(blocking 함수)를 호출하면 context switch가 일어나지 않고 실행속도가
// 낮아지거나 최악의 경우 deadlock에 빠지게 된다. async/await 등의 mechanism을 사용할 때는 협조적 프로그래밍인 것을
// 정확하게 인식하고 구현 및 실행해야 한다.
//
// non-cooperative multi-tasking의 장점과 단점
// 비협조적 multi-task의 장점
// 1) 비협조적 multi-tasking에서는 협조적 multi-tasking에서의 무한 loop, blocking 함수와 관련된 문제는 일어나지
//    않는다. 이들을 실행하거나 호출 중이라 해도 scheduler에 의해 preemption된 다른 process가 실행되기 때문이다.
//    (스케줄러가 process order를 강제하기 때문) 윈도우나 리눅스 등의 현대적인 OS에서는 비협조적 multi-tasking을
//    적용하고 있으므로 앱의 crash가 OS의 crash로 연결되는 경우는 드물다. 비협조적 multi-tasking을 적용한 프로그래밍 언어
//    처리 계열로는 Erlang, Go 등의 언어가 있다.
// 비협조적 multi-task의 단점
// 2) 처리 시스템 구현이 어렵다. 하지만 앱을 구현하는 사람 입장에서는 처리 시스템 구현의 어려움을 크게 신경쓰지 않을
//    것이다. 또한 공평성을 확보하기 위해 빈번하게 context switch를 수행하기도 하므로 협조적 multi-task에 비해
//    다소 overhead가 있다.


/// 6.2 cooperative green thread 구현
/// Rust를 사용해 AArc64상에서 작동하는 간단한 cooperative multi-tasking 구현을 해보자. 이번 구현은 userland
/// thread이며, userland의 software가 독자적으로 제공한 thread mechanism은 일반적으로 green thread라 부른다.
/// green thread는 OS의 스레드와 비교해 스레드 생성과 파기 비용을 줄일 수 있으므로 Erlang, Go, Haskell 같은 동시성
/// 프로그래밍이 뛰어난 처리 계열에서 이용된다. 이들 언어의 green thread 구현은 mulit-thread로 작동하지만 여기서는
/// 간단히 하기 위해 single-thread로 작동하는 green thread를 구현해보자. single-thread 버전의 구현이기는 하지만
/// 이 구현을 확장하면 multi-thread로 만들 수 있다.
///
/// 6.2.1 file 구성과 type, func, var
/// 이 절에서는 이번 구현의 구성과 dependency crate 등을 알아보자. 다음은 이 절과 6.3절 'actor model 구현'에서
/// 이용하는 파일, 함수, 변수를 보여준다.
///
/// 표 6-1 cooperative green thread 구현에 이용하는 파일
/// Cargo.toml          Cargo용 파일
/// build.rs            빌드용 파일
/// asm/context.S       context switch를 수행하기 위한 assembly
/// src/main.rs         main func용 파일
/// src/green.rs        green thread용 파일
///
/// 표 6-2 context switch용 함수(context.S)
/// set_context         현재의 context를 저장
/// switch_context      context switch를 수행
///
/// 표 6-3 context 정보용 type(src/green.rs)
/// Registers           CPU register의 값을 저장하기 위한 type
/// Context             context를 저장하기 위한 type
///
/// 표 6-4 context switch를 수행하기 위한 함수(src/green.rs)
/// spawn_from_main     main함수에서 thread 생성
/// spawn               thread spawn 수행
/// schedule            scheduling 수행
/// rm_unused_stack     불필요한 stack 삭제
/// get_id              자신의 thread ID 획득
/// send                message 송신
/// recv                message 수신
///
/// 표 6-5 global variables(src/green.rs)
/// CTX_MAIN            main 함수의 context
/// UNUSED_STACK        불필요해진 stack 영역
/// CONTEXTS            실행 queue
/// ID                  현재 이용 중인 thread의 ID
/// MESSAGES            message queue
/// WAITING             대기 thread 집합
///
/// CAUTION_ 이번 구현에서는 간단히 하기 위해 global var를 이용하지만 Rust에서는 global var 사용을 권장하지 않음.
///
/// external crates
/// [dependencies]
/// nix = "0.20.0"
/// rand = "0.8.3"
///
/// nix? 유닉스 계열의 OS에서 제공하는 API의 래퍼 라이브러리
/// rand(random) 난수 생성용 crate
fn build_rs() {
    use std::process::Command;

    const ASM_FILE: &str = "asm/context.S";
    const O_FILE: &str = "asm/context.o";
    const LIB_FILE: &str = "asm/libcontext.a";

    fn main() {
        // build-time dependency
        Command::new("cc").args(&[ASM_FILE, "-c", "-fPIC", "-o"])
            .arg(O_FILE)
            .status().unwrap();
        Command::new("ar").args(&["crus", LIB_FILE, O_FILE])
            .status().unwrap();

        // asm을 라이브러리 검색 경로에 추가
        println!("cargo:rustc-link-search=native={}", "asm");
        // libcontext.a라는 정적 라이브러리 링크
        println!("cargo:rustc-link-lib=static=context");
        // asm/context.S라는 파일에 의존
        println!("cargo:rerun-if-changed=asm/context.S");
    }
}
// 이번 구현에서는 어셈블리 파일의 컴파일과 링크도 수행하므로 이와 같이 build.rs 파일을 준비해야 한다. 어셈블리 파일을
// 컴파일하기 위해 cc 명령어와 ar 명령어가 필요하므로 unix환경이 좋다. Ubuntu는 다음 명령을 실행하면 개발도구,
// 라이브러리 및 헤더가 설치된다
// $ sudo apt install build-essential
//
// build.rs vs main.rs?
// build.rs는 패키지에 대한 build-time dependencies 및 custom build scripts를 지정할 수 있음.
// 이 파일은 패키지의 루트에 있으며, main.rs보다 먼저 실행된다.
// main.rs와 build.rs에서 직접 빌드 스크립트를 작성하는 것의 주요 차이점은 build.rs는 build 시에만 실행되는 반면
// main.rs의 코드는 run-time에 실행된다는 것. build.rs를 사용하면 build시간 종속성을 지정하고 build 프로세스를
// 더 자세히 구성하는데 사용할 수 있음.
// build-time dependencies를 지정하거나, custom build scripts를 실행할 필요가 없다면 build.rs를 만들 필요 없다.
// e.g.)
// build-time dependency로 C 라이브러리를 빌드하기 위해 cc crate를 사용하는 예:
// extern crate cc;
//
// fn main() {
//     cc::Build::new()
//         .file("src/lib.c")
//         .compile("libmylib.a");
// }
// 이 script는 소스 파일 src/lib.c에서 libmylib.a라는 C 라이브러리를 빌드.
// 빌드된 라이브러리는 Rust 패키지에 연결됨.
//
// 커스텀 빌드 라이브러리 스크립트:
// extern crate protoc_rust;
//
// fn main() {
//     protoc_rust::Codegen::new()
//         .out_dir("src/")
//         .inputs(&["proto/message.proto"])
//         .run()
//         .expect("protoc");
// }
// proto/message.proto 파일에서 Rust 코드를 생성하기 위해 protoc_rust crate를 사용.
// 생성된 코드는 src/ 디렉토리에 있음.
//
// build-time dependencies?
// 패키지를 빌드하는데 필요하지만 run-time에는 필요하지 않은 종속성.
// custom build scripts?
// 빌드 프로세스 중에 실행되는 스크립트. 이러한 스크립트를 사용해 코드를 생성, 파일을 전처리, 패키지를 빌드하는데 필요한
// 기타 작업을 수행할 수 있음. 커스텀 빌드 스크립트의 일반적인 예는 다른 source file에서 rust code를 생성하는
// 코드 생성기이다.
//
// build.rs는 Cargo에 컴파일 방법을 지정하기 위해 이용하는 파일이며, Cargo는 build.rs의 내용에 기반해 Rust의
// 컴파일을 수행한다. build.rs에 기술된 내용은 다음의 컴파일과 정적 라이브러리를 만드는 명령어와 동일하다.
// $ cc asm/context.S -c -fPIC -o asm/context.o
// $ ar crus asm/libcountext.a asm/context.o
//
// cc는 C 컴파일러이며 일반적으로 gcc 또는 clang이 이용되며, C 및 assembly code를 컴파일하는데 사용된다.
// -c flag는 올바른 컴파일 단계 후에 중지하고 linking을 수행하지 않도록 컴파일러에 지시함
// -fPIC flag는 위치 독립적 코드를 생성하도록 컴파일러에 지시함. PIC는 런타임시 로드되는 공유 라이브러리에 사용되므로
// 컴파일 시 메모리 위치를 알 수 없음.
// -o [arg] flag는 출력 파일의 이름과 경로를 지정함.
//
// ar은 정적 라이브러리 작성이나 정적 라이브러리로부터의 파일 추출을 위한 명령어.
// ar은 archive 명령이며 아카이브 파일을 생성, 수정 및 추출하는데 사용됨.
//
// 표 6-6 ar 명령어의 옵션 목록
// c flag는 아카이버에게 새 아카이브 파일을 생성하도록 지시
// r flag는 아카이버에게 아카이브에서 지정된 객체 파일을 교체하도록 지시. 책장에 파일을 삽입.
//   이미 같은 이름의 파일이 존재하면 치환(overwrite)
// u flag는 아카이버가 이미 아카이브에 있는 버전보다 최신 버전인 경우에만 아카이브에 있는 개체 파일을 교체하도록 지시
// s flag는 아카이버에게 아카이브의 기호 테이블을 업데이트하도록 지시
// d flag는 기호 테이블(색인)을 책장에 써넣음. 색인이 존재하는 경우에는 업데이트
// 즉, asm/context.o를 책이라고 생각하면 asm/libcontext.a는 책장이며, ar은
// asm.libcontext.a라는 책장에 파일(책)을 넣고 빼기 위한 명령어다. 오래전 software는 천공 카드(punch card)라는
// 물리적 종이에 기록되어 책과 같은 형태였다. 그리고 그 책(천공 카드)은 책장에 관리되었으며 software 관리는 실제
// 책장 관리와 동일했다.
//
// build.rs에서는 작성된 asm/libcontext.a를 link해서 컴파일하도록 지정한다.
fn src_green_rs() {
    use nix::sys::mman::{mprotect, ProtFlags};
    use rand;
    use std::alloc::{alloc, dealloc, Layout};
    use std::collections::{HashMap, HashSet, LinkedList};
    use std::ffi::c_void;
    use std::ptr;
}

/// 6.2.2 context
/// context는 process의 실행 상태에 관한 정보이며, 가장 중요한 정보는 register 값이다. [그림 6-4]에 이번 구현에서
/// 저장하는 context와 CPU 및 메모리의 관계를 나타냈다.
///
/// text영역은 실행 명령이 놓인 메모리 영역. 그림에서는 set_context라는 context를 저장하는 함수가 호출되면 caller
/// 저장 register는 컴파일러가 출력한 코드에 따라 stack으로 회피된다. 한편 callee 저장 register는 회피되지 않으므로
/// set_context 함수가 heap상의 확보된 영역에 저장된다.
/// (set_context 호출시: caller 저장 register -> stack으로 회피, callee 저장 register -> 회피불가. heap에 저장됨)
/// ret 명령에서의 반환 위치 주소를 나타내는 link 주소인 x30 register와 stack pointer를 나타내는 sp register도
/// 마찬가지로 저장된다. 그러면 다른 process 실행 후 context에 저장된 register 정보를 복원하고 ret 명령어로 반환하면
/// set_context 함수를 호출한 다음 주소(x30 register가 지정된 주소)에서 실행을 재개함.
// fn f() {
//
// }
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
fn func_() {

}
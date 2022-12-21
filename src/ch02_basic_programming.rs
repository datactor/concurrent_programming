// 2.1 어셈블리 언어
// 어셈블리?? 왜? 어셈블리를 알면 concurrency programming의 원리를 알 수 있고 컴퓨터의 본질을 이해할 수 있게됨.
// AArch64(Arm의 64bit 아키텍처), x86-64(AMD, Intel의 CPU 아키텍처)로 간단하게 학습해보자.
//
//
// 2.1.1 어셈블리 언어 기본
// 우리가 일반적으로 보는 수식 너무 당연하게 여겨지지만 중위 표기법(infix notation)으로 연산자를 중간에 둠.
// 반면 어셈블리 언어에서는 연산자를 맨 앞에 둠.
// 중위 표기법  : x0 = x1 + x2
// 어셈블리     : add x0 x1 x2
// 어셈블리 언어에서는 +와 같은 기호는 사용하지 않으며 모두 영단어로 기술. AArch64 어셈블리에서 다음과 같이 기술됨.
// add x0 x1 x2 ; x0= x1 + x2
// 여기서 add는 mnemonic이라 불리는 명령의 종류를 나타내고 x0, x1, x2는 변수를 나타냄. 또한 ;부터 행의 마지막까지는 주석
//
// 어셈블리 언어의 1명령: 니모닉이라는 명령의 종류(operation code)와 피연산자(operand)라 불리는
// 하나 또는 여러 상수로 레지스터에서 이뤄짐.
// register는 cpu내의 저장장치로 컴퓨터 안에서 가장 접근 속도가 빠르고 용량이 적은 기억 영역.
// 위 식에서 x0, x1, x2의 변수가 레지스터에 해당함. 니모닉을 함수명, 피연산자를 인수라고 생각하면 이해하기 쉬움.
// 어셈블리 언어로 쓰인 프로그램을 어셈블리 코드, 어셈블리 코드를 기계어로 컴파일하는 소프트웨어를 컴파일러라 부름.
//
// 컴파일 시 니모닉은 적절한 기계어로 변환되며, 명령을 나타내는 바이트 코드를 operation code라 부름.
// e.g. add명령을 나타내는 바이트 코드가 0x12라고 가정할 경우 0x12가 오퍼레이션 코드가 됨.
// 때문에 니모닉 자체를 operation 코드라 부르기도 함
//
// 어셈블리 프로그래밍에서는 레지스터와 함께 메모리 접근도 중요.
// e.g. [x1]이 x1 레지스터를 가리키는 주소로의 접근을 나타낸다고 정의하면, 메모리 읽기는 다음과 같이 기술할 수 있음.
// ldr x0, [x1] ; [x1]의 메모리 값을 x0에 읽는다. ldr? load 메모리 읽기 in AArch64
// str x0, [x1] ; [x1]의 메모리에 x0의 값을 쓴다. str? store 메모리 쓰기 in AArch64
// mov x0 x1 ; x1의 값을 x0으로 복사. mov? move 값을 대입 in AArch64
//
//
// 2.1.2 x86-64 어셈블리 기초
// x86-64(windows) 어셈블리의 예도 간단히 살펴보자. x86-64 어셈블리 기술은 2종류가 있음.
// clang이나 gcc 등의 C 컴파일러를 이용할 때 AT&T기법을 사용하기 때문에 AT&T 기법 위주로 보자.
//
// x86-64의 덧셈
// addl %ebx, %ecx ; ebx와 ecs를 더한 결과를 ecx에 저장 -> AArch64에서는 저장 위치의 레지스터를 명시했지만,
// x86-64에서는 읽기와 저장 레지스터가 동일함. addl의 l? operation suffix(suffix)라 불리며 이는 레지스터 크기 지정.
// ebx, ecx의 레지스터는 32비트, rbx, rcx 등의 레지스터는 64비트 레지스터가 된다.
//
// 64비트의 레지스터를 복사하는 명령
// movq %rbx, %rcx ; rbx의 값을 rcx로 복사
// 64비트 레지스터에서의 operation suffix는 q다. 쓰기 대상 레지스터를 destination resgister, 쓰기 원본 레지스터를
// source register라 부르지만 AT&T 기법에서는 AArch64와 소스와 데스티네이션 레지스터의 위치가 반대인 것을 주의!!
//
// x86-64에서는 메모리 읽기와 쓰기도 mov 명령으로 실행 가능. 메모리 읽기와 쓰기 명령의 예
// movq (%rbx), %rax ; rbx가 가리키는 메모리 상의 데이터를 rax로 전송 -> 메모리 읽기 명령
// movq %rax, (%rbx) ; rax의 값을 rbx가 가리키는 메모리로 전송 -> 메모리 쓰기 명령
//
//
// 2.2.3 스택 메모리와 힙 메모리
// 스택 메모리? 함수의 로컬 변수를 저장하기 위한 메모리 영역, 힙 메모리? 함수의 스코프에 의존하지 않는 메모리를
// 동적으로 확보하기 위한 메모리 영역.
pub fn fun1() -> i32 {
    let a = 10; // a의 라이프타임은 함수에서 반환하는 시점까지, 값은 스택에 저장됨.
    return 2 * fun2(a);
}

fn fun2(a: i32) -> i32 {
    let b = 20; // b의 라이프타임은 함수에서 반환하는 시점까지, 값은 스택에 저장됨
    return a * b;
}
// 컴파일러에 의한 최적화가 수행되면 a, b 모두 스택이 아니라 레지스터에 저장되지만 컴파일러에 의한 최적화가 수행되지
// 않는다고 가정한 상태를 나타내 보자(스택 메모리)
//
//       |--------|       |--------|       |--------| 낮은 주소
//       |        |       |        |       |        |
//       |        | sp -> |--------|       |        |
//       |        |       | b = 10 |       |        |
// sp -> |--------|       |--------| sp -> |--------|
//       | a = 10 |       | a = 10 |       | a = 10 |
//       |--------|       |--------|       |--------| 높은 주소
//       fun1 호출 후      fun2 호출 후     fun2에서 반환 후
//           1                2                3
// 스택 메모리는 대부분의 경우 높은 주소에서 낮은 주소 방향으로 진행한다(선입후출). sp? 스택 포인터로 어느 정도까지
// 스택 메모리를 소비했는지 나타내는 레지스터.
// 1) fun1이 호출됨 -> 로컬 변수 a의 정보만 스택에 저장됨.
// 2) fun2가 호출됨 -> 로컬 변수 b의 정보도 스택에 저장됨.
// 3) fun2함수에서 반환 -> 로컬 변수 b는 해제됨.
// *tip - 실제 스택 조작은 스택 포인터 값을 변경하는 것만으로 수행되므로 이 시점에서는 로컬 변수 b의 정보는 스택상에
// 남아 있음. but 여기서는 개념적인 의미에서 생각할 것! 소프트웨어에서는 개념과 구현을 구분해서 생각하는 것이 중요!
//
// 이처럼 로컬 변수는 함수에서 return하면 파기됨. 하지만 힙 메모리를 이용하면 함수의 스코프에 묶이지 않고 변수를
// 정의할 수 있음. 힙 메모리를 사용해보자.

pub fn fun3() {
    let a = 10; // 지점 A
    let b = fun4(a); // 지점 C
                     // std::mem::drop(b); // 힙 메모리 해제.
} // 사실 러스트에선 함수 스코프가 끝나면 드랍됨

fn fun4(a: i32) -> Box<i32> {
    let mut tmp = Box::new(0); // 힙 메모리 확보
    *tmp = a * 20;
    return tmp; // 지점 B
}
//       |--------|       |--------|       |--------| 낮은 주소
//       |        |       |        |       |        |
//       |--------|       |--------|       |--------|
//       |        |       | a * 20<┼---┐   | a * 20<┼---┐
//       |        |       |--------|   |   |--------|   |
//       |        |       |        |   |   |        |   |
//       |        | sp -> |--------|   |   |        |   |
//       |        |       |  *tmp -┼---┘   |        |   |
// sp -> |--------|       |--------| sp -> |--------|   |
//       |   *b   |       |   *b   |       |   *b  -┼---┘
//       |--------|       |--------|       |--------|
//       | a = 10 |       | a = 10 |       | a = 10 |
//       |--------|       |--------|       |--------| 높은 주소
//         지점 A            지점 B            지점 C
//
// 컴퓨터상에서 메모리는 적어도 스택 메모리와 힙 메모리로 역할을 나눠 이용함.
// 지점 A: fun3의 로컬 변수 a와 *b만 스택 메모리에 확보.
// 지점 B: func4의 로컬 변수 *tmp가 스택에, a * 20이라는 값이 힙에 확보됨.
// 지점 C: a * 20이라는 값은 힙에 여전히 그대로 보존됨. 힙메모리의 값은 명시적으로 해제될 때까지 확보상태를 유지하지만,
// 러스트에서는 명시적으로 해제시키지 않아도 스코프가 끝나면 컴파일러가 알아서 해제시킴!

// 2.3 내사랑 Rust. 이 부분은 https://rinthel.github.io/rust-lang-book-ko/ 공식 문서 참조!!
// sharing-nothing, affine, type system, ownership, monomorphism....
// Rust는 type safety system을 제공, dangling pointer나 null pointer exception 등의 포인터 관련 문제가
// 잘 발생하지 않음. unsafe기능을 사용하지 않는 이상 왠만한건 compile때 다 막아줌.
//
//
// 2.3.1 type system
// 기본 타입 - 정수값(i.., 환경의존은 isize), 부호 없는 정수값(u.., 환경의존은 usize)
// , 부동소수점(f..) 논리값(bool), 문자(char, 스트링 슬라이스(리터럴 스트링)), 튜플, 배열
// https://rinthel.github.io/rust-lang-book-ko/ch03-02-data-types.html
//
// 러스트에서 String은 스마트 포인터임 https://rinthel.github.io/rust-lang-book-ko/ch08-02-strings.html
//
// 사용자 정의 type(struct, enum, multiple type) - 구조체(struct)
// 기본적으로 모든 변수는 이용 시 반드시 초기화 해야함. struct의 인스턴스를 먼저 생성하고 난 뒤 값을 정의할 수는 없음.
//
// Generic(T) -> 러스트는 기본적으로 단형성화하기 때문에 런타임 패널티 없음.
// https://rinthel.github.io/rust-lang-book-ko/ch10-01-syntax.html
//
// reference 타입(값을 가리키는 주소값을 변수로 나타내는 포인터)
// https://rinthel.github.io/rust-lang-book-ko/ch04-02-references-and-borrowing.html
// rust는 기본적으로 파괴적 대입 불가(immutable)
// 참조타입은 반드시 라이프타임이 있음(대부분의 경우 컴파일러가 유추해주기 때문에 명시를 생략할 뿐임)
// https://rinthel.github.io/rust-lang-book-ko/ch10-03-lifetime-syntax.html
//
//
// 2.3.2 기본 문법
// 세미콜론이 행의 끝에 있으면 statement(구문)으로 간주. 정확히 말하면 () 빈 튜플을 반환함.
// 세미콜론을 이용해 여러 expression(식)을 연속해서 나열. 끝에 세미콜론이 없는 것은 return하는 식.
// 세미콜론의 sementically한 이야기는 람다 계산 등으로 이뤄짐.
//
// 함수 정의와 호출
// Rust는 타입에 엄격하기 때문에 식은 값을 반환함. 같은 구문에 속한 식은 전부 타입을 일치시켜야함.
// https://rinthel.github.io/rust-lang-book-ko/ch03-03-how-functions-work.html
//
// 함수 포인터
// 러스트에서 참조는 소유권이나 라이프타임에 의해 안정성이 보증되지만 포인터는 그렇지 않음.
// https://rinthel.github.io/rust-lang-book-ko/ch15-00-smart-pointers.html
// 하지만 대부분의 경우 실행 코드가 동적으로 변하지는 않으므로, 함수 포인터에 한정해서는 안전하게 이용할 수 있음.
fn app_n(f: fn(u64) -> u64, mut n: u64, mut x: u64) -> u64 {
    loop {
        if n == 0 {
            return x;
        }
        x = f(x);
        n -= 1;
    }
}

fn mul2(x: u64) -> u64 {
    x * 2
}

pub fn my_func3() {
    println!("app_n(mul2, 4, 3) = {}", app_n(mul2, 4, 3));
}
// f: fn(u64) -> u64 함수값을 받아 반환 타입으로 바꾸는 함수 포인터
//
// closure(heap상에 캡처한 변수와 자유 변수 환경을 배치한 함수)
// 함수 body와 함께 함수 바깥에서 캡처한 자유 변수의 값을 포함함. 클로저의 원 개념은 람다 계산이 출현한 1960년경으로
// 거슬러 올라갈 수 있는데, 당시에는 그저 간단한 이름 없는 함수였음. 하지만 스택 기반의 실행 환경에서 자유변수를
// 캡처하면 스택상에 확보된 값이 파기되는 문제가 종종 발생했음. 그래서 Landin이 1964년에 발표한 SECD 머신에서는
// heap상에 변수와 자유 변수 환경을 배치해 이 문제를 해결하고 이를 클로저라 정의했음.
// *NOTE! 함수 밖에서 정의된 변수를 자유 변수, 함수 내부에서 정의된 변수를 종속 변수라함. 함수형 프로그래밍의 흐름도
// 이어받은 Rust는 함수 내부에서 함수를 정의할 수 있기 때문에 클로저에 관해 생각할 때는 함수 안 또는 밖 어디에서
// 변수가 정의되었는지 구별해야함. C는 함수 내부에서 함수를 정의할 수 없으므로 자유변수는 global var, 종속 변수는
// local var임. 그러나 Rust의 경우 global var은 자유변수지만, local var는 자유변수와 종속 변수 모두 될 수 있다.
fn mul_x(x: u64) -> Box<dyn Fn(u64) -> u64> {
    // 1
    Box::new(move |y| x * y) // 2
} //   args    body

pub fn my_func4() {
    let f = mul_x(3); // 3
    println!("f(5) = {}", f(5)); // 4
}
// 1) u64타입의 값(x)을 받아 Box::<dyn Fn(u64) -> u64> 타입의 값으로 반환하는 함수 mul_x를 정의.
// 2) 클로저를 정의. 클로저는 '|var1, var2...| 식'과 같이 기술할 수 있으며 var1, var2...가 클로저의 인수,
//    식이 클로저의 본체가 됨.
// 3) mul_x에 3을 전달하고 |y| 3 * y라는 클로즈를 힙상에 생성.
// 4) 생성한 클로저를 호출하고 3 * 5를 계산해서 호출.
//
// Box는 기본적인 기능밖에 없는 스마트 포인터 https://rinthel.github.io/rust-lang-book-ko/ch15-01-box.html
// Box는 컨테이너의 일종으로 일반적으로 heap상에 데이터를 배치할 때 이용함. Box type의 변수가 스코프를 벗어나면
// 확보된 데이터(힙)가 자동으로 파기됨. dyn은 trait의 작동이 동적으로 결정되는 것을 나타냄(dynamic dispatch).
// https://rinthel.github.io/rust-lang-book-ko/ch17-02-trait-objects.html
// 즉 dyn trait의 참조는 함수와 값으로의 포인터(환경)을 가지며 이들은 동적으로 할당됨. 클로저는 함수와 자유 변수
// 환경을 가지고 있으므로 dyn이 클로저에 필요함. Fn(u64) -> u64는 함수 포인터와 마찬가지로 u64 type의
// 값을 받아 u64 type의 값을 반환하는 함수. 즉, Box::<dyn Fn(u64) -> u64> type은 heap상에 확보된 함수와
// 값으로의 포인터를 가진 클로저에 대한 스마트 포인터이다(Rust에서 컴파일 시에 크기를 알 수 없는 값이 시그니처에
// 들어갈 수 없다(heap 데이터). 그래서 Box, Vec, String 등의 스마트 포인터에 넣어서 포인터를 타입으로 만들어줘야함)
// |y| x * y라는 클로저는 변수 y가 인수에 나타나 있으므로 y는 종속 변수지만 x는 자유 변수다. 클로저 바깥에서
// 정의된 변수 x가 이 클로저에서 캡처된다. 클로저의 변수 캡처 전략에는 차용(참조를 취득) 혹은 소유권 이동(move)이
// 있음.
// 2행에서 소유권 이동. 왜? 참조를 얻었을 때 변수 x는 함수 mul_x에서 벗어난 시점에서 파기되어 무효한 참조가 되기 때문.
// 즉, Box::new(move |y| x * y)는 heap상에 x * y를 수행하는 클로저를 생성하고, 자유 변수 x는 소유권 이동으로 캡처됨.
//
//
// 2.3.3 ownership https://rinthel.github.io/rust-lang-book-ko/ch04-00-understanding-ownership.html
// 러스트의 소유권 이전에 먼저 선형 논리라는 논리 체계부터 알아보자. Γ(감마)와 Φ(파이)를 논리식의 집합이라고 하면
// Γ ├ Φ 라는 식은 Γ가 옳을 때 Φ가 옳다고 증명할 수 있다는 것을 의미. 다음 식은 전건 긍정(modus ponens)라 불리는
// 논리 법칙이다(긍정 논법 또는 함의 소거라고도 불림. 논리학에서 가언 명제와 그 전제로부터 결론을 유도해내는 추론 규칙.
// '만약 P면 Q다'와 'P다'에서 'Q다'를 추론한다.) *분수가 아님
//      A  A -> B
//      ---------   이는 A이고 A->B(A이면 B)이면 B다 라고 추론할 수 있다는 규칙이다.
//          B       e.g. A: 비가 온다, B: 땅이 젖는다, A->B는 '비가 오면 땅이 젖는다'가 되며 이를 옳다고 하면,
//                   비가 온다. 비가 오면 땅이 젖는다. ├ 땅이 젖는다.
// 이를 기호로 쓰면 다음과 같은 식이 된다. A, A -> B ├ B
// 여기서 C를 '우산이 팔린다', A->C를 '비가 오면 우산이 팔린다', A->C가 옳다고 하면 다음과 같이 나타낼 수 있다.
//                  A, A -> B, A -> C ├ B ∧ C 이상이 고전적인 논리다.
// 여기서  A(비가 내린다)라는 이미 있는 전제를 한 번 더 추가해보자. 그러면 다음과 같이 나타낼 수 있다.
//                  A, A, A -> B, A -> C ├ B ∧ C
// 이는 '비가 내린다'와 같은 말을 2번 반복했을 뿐, 원래 식과 논리적으로는 동일하다. 이 논리가 성립하는 이유는
// 비가 온다는 사실은 이용 횟수에 제한이 없고, 한 번 말한 것은 횟수에 제한 없이 사용할 수 있기 때문이다.
// 그렇다면 다음과 같은 경우는? A를 '사과가 1개다', B를 '배가 부르다', C를 '돈이 늘어난다'라고 한다면?
//                  A, A -> B, A -> C ├ ?
// 사과가 있으면 배가 부르거나 돈이 늘어날 수 있지만 사과가 하나뿐이라면 양쪽을 모두 달성하기 불가능하다.
// 따라서 이 식의 ?에는 B또는 C 둘 중 하나만 가능하다. 이렇게 자원 이용에 제한을 가질 수 있는 논리 체계에
// 선형 논리가 있다. 선형 논리에서는 ->(~라면)을 ⊸으로 표시한다. 그러면 이 식은
//                  A, A ⊸ B, A ⊸ C ├ B 또는
//                  A, A ⊸ B, A ⊸ C ├ C 가 된다.
// 선형 논리를 기반으로 한 type system에 선형 타입이 있으며, 선형 타입을 적용한 프로그래밍 언어에는 Cyclone이 있다.
// Rust는 Cyclone의 영향을 크게 받아 개발된 언어이며, Rust에서는 선형 타입의 자매 격인 affine type을 적용하고 있다.
// *affine type: a fancy name for the combination of Rust's enum and move semantics which ensure
//               that if a type isn't marked as copyable, it can only be moved, not copied.
// 다음 소유권과 move 구문의 예를 살펴보자.
pub struct Apple {} // 1
pub struct Gold {}
pub struct FullStomach {}

pub fn get_gold(a: Apple) -> Gold {
    // 사과를 팔아 돈을 얻는 함수
    Gold {}
}

pub fn get_full_stomach(a: Apple) -> FullStomach {
    FullStomach {}
}

fn my_func5() {
    // 2
    let a = Apple {}; // 사과가 1개 있다고 가정. 3
    let g = get_gold(a); // 사과를 팔아 돈을 얻음. 4

    // let s = get_full_stomach(a); // 5 (사과를 팔아 이미 돈을 얻었으므로 컴파일 에러 발생)
}
// 1) 사과, 돈, 포만감을 나타내는 타입 정의
// 2) 사과를 팔아 돈을 얻는 작동을 나타내는 함수
// 3) 먼저 사과가 1개임을 정의한다. 이때 사과의 소유권은 변수 a에 있음.
// 4) 사과가 get_gold 함수에 전달되고 돈을 얻음. 처음에는 변수 a가 사과를 소유하지만 get_gold함수에 전달함으로써
//    사과의 소유권이 get_gold함수로 이동함. 이 소유권은 이동을 move, 그 의미론을 move semantics라 부른다.
// 5) 소유권이 변수 a에서 get_gold로 이동했으므로 판매한 사과를 이용해 포만감을 얻을 수 없으며 컴파일 에러 발생.
//
// 소유권의 기반인 선형 논리와 함께 생각해보면 그 의미가 명확해짐.
//
//
// 2.3.4 라이프타임 https://rinthel.github.io/rust-lang-book-ko/ch10-03-lifetime-syntax.html
// Rust의 변수는 lifetime이라는 상태를 유지함.
// 라이프타임을 명시하는 예)
struct Foo {
    val: u32,
}

fn add<'a>(x: &'a Foo, y: &'a Foo) -> u32 {
    // 1
    x.val + y.val
}

fn my_func6() {
    let x = Foo { val: 10 }; // 2
    {
        let y = Foo { val: 20 };
        let z = add(&x, &y); // 3
        println!("z = {}", z);
    }
}
// 1) add 함수는 라이프타임을 'a라는 변수로 받고, &'a Foo type의 참조를 변수 x와 y로 받아 u32 type의 값을 반환함.
// 2) x와 y에 Foo 타입의 값을 대입함.
// 3) 그 후 add함수를 참조 전달로 호출함.
// 위 코드는 제네릭의 일종이며 type을 인수로 받는 함수다. 즉, 타입 시스템에서는 type을 받아 type을 return하는 type에
// 관해 살펴봤지만, 이는 type을 받는 '함수'를 return하는 '함수'이다. 사실 lifetime도 type의 일종이며, 라이프타임을
// 받는 인수에는 prefix로 작은따옴표를 붙임. 작은따옴표를 prefix로 붙일 수 없는 var에는 u32나 bool 등 일반적인
// type을 인수로 붙일 수 있음. Rust에서는 라이프타임을 reference에만 명시할 수 있으며 이때는 엠퍼센드뒤에
// lifetime 변수를 기술한다. 즉 &'a Foo가 lifetime을 명시한 참조 타입이 된다.
//
// add 함수의 인수 x와 y의 명시된 'a변수는 같으므로 라이프타임이 같아야함. 실제의 x, y의 라이프타임은 y가 짧다.
// 이 경우 범위가 적은 쪽인 y에 맞춰 참조를 전달함. Rust에서는 이렇게 서로 다른 lifetime이더라도 한쪽으로 합칠 수
// 있으며, 이는 subtyping이라 부르는 기술을 이용해 구현한다(duck typing 참조할 것).
// 서브타이핑은 객체 지향에서 클래스로 다형성(polymorphism)을 구현하기 위한 타입 시스템의 일종이었다. 객체 지향 언어
// 에서는(러스트도 객체지향 성향을 가지고 있으며 구현할 수 있다. 그렇지만 단형성화에 있는 러스트의 장점을 살리기
// 어려울 뿐임. https://rinthel.github.io/rust-lang-book-ko/ch17-03-oo-design-patterns.html) 사과와 배
// 같은 클래스는 같은 기능을 가지고 있으므로, 과일이라는 클래스에서 파생해 양쪽 모두 같은 함수로 조작할 수 있음.
// 과일에서 사과나 배의 클래스를 파생한 것을 과일 < 사과, 과일 < 배라고 과일을 다루는 함수는 양쪽 모두 조작할 수 있기
// 때문에 이를 구현하는 것이 서브타이핑이 된다. 라이프 타임의 경우 x의 라이프타임[10행, 16행 < y의 라이프타임[12행, 15행]
// 을 클래스의 파생이라고 생각하면 [10, 16]은 [12, 15]로 다룰 수 있다고 생각할 수 있어 클래스의 서브타이핑과 같다.
// 단 Rust의 경우는 lifetime의 subtyping을 특별히 lifetime subtyping이라 부른다.
//
//
// 2.3.5 차용
// 차용을 사용하지 않고 move semantics만 사용한다면? 함수에 대한 소유권을 전달해서 계산한 뒤 다른 계산을 할 때
// 해당 함수로부터 소유권을 다시 반환해야함. 코드로 보자
struct Foo2 {
    val: u32,
}
fn add_val(x: Foo2, y: Foo2) -> (u32, Foo2, Foo2) {
    (x.val + y.val, x, y) // 1
}

fn mul_val(x: Foo2, y: Foo2) -> (u32, Foo2, Foo2) {
    (x.val * y.val, x, y) // 2
}

fn my_func7() {
    let x = Foo2 { val: 3 };
    let y = Foo2 { val: 6 };
    let (a, xn, yn) = add_val(x, y); // 3
    let (b, _, _) = mul_val(xn, yn); // 4
    println!("a = {}, b = {}", a, b);
}
// 1) Foo2 안의 멤버 변수 val을 더함. 단, 이들 함수는 일단 변수 x, y에 소유권을 얻은 값을 반환값으로 하여 결괏값과 함께 반환함.
// 2) Foo2 안의 멤버 변수 val을 곱함. 마찬가지로 변수 x, y의 소유권을 함께 반환함.
// 3) add_val 함수에 변수 x와 y가 가진 Foo의 소유권을 전달해 호출함. 결괏값(변수 a)와 함께 반환된 Foo2의 소유권을 변수 xn과 yn에 저장.
// 4) 변수 xn과 yn의 소유권을 mul_val함수에 전달해 호출한 뒤 결과를 변수 b에 넣음.
//
// 위 예시도 올바른 코드지만 매우 장황함. 계산을 할때마다 소유권을 반납하기 까다로움(그럴 필요가 없음에도). 실제로는
// 참조(borrow)를 이용해 구현한다. Rust에서 참조는 차용이라는 개념이며 참조 이용 방법에는 몇가지 제약을 두고 있음.
// 제약(borrow rule)으로 인해 기술의 자유도는 떨어지지만 고속성, 안전성이라는 큰 장점을 얻을 수 있음.
// https://rinthel.github.io/rust-lang-book-ko/ch04-02-references-and-borrowing.html
// 차용(borrow)이 중요한 곳은 파괴적 대입(mutable)이 가능한 객체의 경우이며, 차용은 다음 두 가지를 보증한다.
// - 어떤 객체에 파괴적 대입을 수행할 수 있는 프로세스는 동시에 2개 이상 존재하지 않는다.(mut borrow는 한번만 가능)
// - 어떤 시각에 어떤 객체에 파괴적 대입을 수행할 수 있는 프로세스가 존재하는 경우 그 시각에는 해당 객체의 읽기 쓰기가
//   가능한 프로세스가 더 이상 존재하지 않는다.(굳이 하자면 interior mutability로 비슷한 효과를 낼 수 있음)
// 이를 보증하는 큰 이유 중 하나는 동시성 프로그래밍의 에러를 방지하기 위함. 공유 객체를 여러 프로세스가 보유하고
// 업데이트하면 해당 객체가 어느 타이밍에 업데이트 되는지 완전하게 파악해야함. 그러나 1.4.2절에서 동시 프로세스의
// computation tree에서 경우의 수가 방대해지기 때문에 전체를 파악하는 것이 어렵고 버그의 온상이 될 수 있음.
// 여기서 borrow rule이 대부분의 버그를 방지해 줄 수 있음. 분산 컴퓨팅 세계에서는 고효율의 계산과 고가용성을 구현하기
// 위한 설계 사상으로 shared-nothing이라는 개념이 있는데, 이것은 공유 자원을 모두 가지지 못하도록 분산 시스템을 설계
// 및 구현하며 Rust에서의 소유권과 borrow 역시 shared-nothing에 기반한 사고라 볼 수 있음.
//
// 러스트의 차용? mut var, immut var, &mut var, &var 4종류로 나눌 수 있음. 그림 2-3 참조
// https://rinthel.github.io/rust-lang-book-ko/ch04-02-references-and-borrowing.html
// 예시
pub struct Foo3 {
    val: u32,
}

pub fn my_func8() {
    let mut x = Foo3 { val: 10 }; // mut var 생성 1
    {
        let a = &mut x; // mut var에 대한 mut 참조 2
        println!("a.val = {}", a.val);

        // // x는 '&mut borrow 중' 상태이므로 에러
        // println!("x.val = {}", x.val);

        let b: &Foo3 = a; // b는 이뮤터블 참조 4
                          // a.val = 20; // a는 '& borrow' 상태이므로 에러 5
        let d: &Foo3 = a; // 9
        println!("{}", d.val); // 10
        println!("b.val = {}", b.val); // 6
                                       // 여기서 b가 차용중인 소유권이 반환된다.

        a.val = 30;
    }
    println!("{}", x.val);

    {
        let c = &x; // c는 이뮤터블 참조 7
        println!("c.val = {}", c.val);
        println!("x.val = {}", x.val);

        // let d = &mut x; // x는 '& borrow' 상태이므로 에러, 모든 immutable var가 반환되기 전까진 mut borrow 불가 8
        // d.val = 40;

        let e = &x; // 11
        let f = &x;
        let g = &x;
        println!("{}", e.val);
        println!("{}", f.val);
        println!("{}", g.val);

        println!("{}", c.val);

        let h = &mut x; // 12
        h.val = 40;

        let i = &mut x; // 13
        i.val = 40;

        // println!("c.val = {}", c.val);
    }

    println!("x.val = {}", x.val);
}
// 1) mut var x의 상태는 '초기 상태'.
// 2) mut var x에서 mut 참조를 생성하고 그것을 뮤터블 참조 a가 borrow함. 이때 mut var x는 '&mut borrow 중',
//    뮤터블 참조 a는 '초기 상태'가 됨.
// 3) mut var x에 접근하려고 해도 변수 x가 '&mut borrow'이므로 컴파일 에러 발생.
// 4) mut 참조 a에서 이뮤터블 참조를 생성하고 그 소유권을 이뮤터블 참조 b가 차용함. 이때 mut 참조 a는 '& borrow 중',
//    이뮤터블 참조 b는 '초기 상태'가 됨.
// 5) 뮤터블 참조 a에 파괴적 대입을 수행하면 컴파일 에러 발생.
// 6) 이뮤터블 참조 b가 마지막으로 이용되고, 차용된 참조의 반환은 그 후에 일어남. 그러므로 이 행을 실행한 후 이뮤터블
//    참조 b가 차용했던 소유권이 변수 a로 반환되고, 뮤터블 참조 a는 '초기 상태'로 돌아간다. 그 결과 뮤터블 참조 a
//    에 대해 다시 파괴적 대입이 가능하게 됨.
// 7) mut var x에서 이뮤터블 참조가 생성되고, 그 소유권을 이뮤터블 참조 c가 차용한다. 이때 mut var x는
//    '& borrow 중', 이뮤터블 참조 c는 '초기 상태'가 된다.
// 8) 따라서 mut var x에서 뮤터블 참조를 생성해 파괴적 대입을 수행하려 하면 컴파일에러가 발생한다.
//
// *차용 요약
// 1) mut 변수 x를 a라는 변수에 &mut borrow해 가져가서 사용할 경우 x는 사용 불가(읽기 포함).
// 2) &mut borrow해서 가져간 값(초기상태인 a의 값)을 print만 찍어서 사용해도 반환됨(값을 변경해도 사용, 반환됨)
//    즉 반환되었으면 다시 mut 변수 x를 사용할 수 있다. 헷갈릴 수 있는 부분임. 스코프가 끝나지 않았음에도 반환됨!!
// 3) 이후로 a를 이뮤터블 참조해서 b라는 변수에 가져가면 a의 값은 변경할 수 없음(&mut var를 & borrow했으므로.)
//    그러나 이뮤터블 참조로 차용시켰기 때문에 '변경'할 수 없는 것이지 '사용'은 할 수 있음. 9, 10 참조
// 4) mut 변수 x를 c라는 변수에 & borrow(이뮤터블 참조)해 가져간다면, c가 초기 상태일 경우나 반환되었을 때만
//    x를 &mut borrow할 수 있음. 8, 12, 13
//    그러나 & borrow는 제한없이 가능하다. 11 참조
//
// 소유권과 차용(+타입시스템)이라는 개념을 도입해 얻을 수 있는 장점? 동시성 프로그래밍에서 맞닥뜨리는 문제와,
// Garbage Collection에서 만날 수 있는 문제를 해결할 수 있다는 것.
// e.g. 여떤 객체가 여러 위치에서 참조되는 상황에서는 해당 객체를 참조할 수 없는 타이밍을 감지해 객체를 파기하지 않으면
//      않으면 메모리 누설이 발생한다.
// 이를 수행하는 것이 GC지만 GC는 프로그래머가 관리하기 어렵고 실행 속도에 영향을 미칠 수 있음. 또한
// RC(Reference counter) 등을 포함해 GC에는 일정 이상의 오버헤드가 발생한다. 하지만 소유권과 차용이 있다면?
// 객체를 파기하는 타이밍을 컴파일 시 알 수 있어 그런 문제가 발생하지 않는다.
//
//
// 2.3.6 메서드 정의
// 객체 지향 언어 특) 어떤 객체에 대한 함수를 정의할 수 있으며 이를 method라 부름.
// Rust는 impl 키워드를 이용해 메서드 정의 가능
pub struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    // 1
    fn new(x: f64, y: f64) -> Self {
        // 2
        Vec2 { x, y }
    }

    fn norm(&self) -> f64 {
        // 3
        (self.x * self.x + self.y * self.y).sqrt()
    }

    fn set(&mut self, x: f64, y: f64) {
        // 4
        self.x = x;
        self.y = y;
    }
}

pub fn my_func9() {
    let mut v = Vec2::new(10.0, 5.0); // 5
    println!("v.norm = {}", v.norm());
    v.set(3.8, 9.1);
    println!("v.norm = {}", v.norm());
}
// *TIP self의 타입은 참조뿐만 아니라 참조가 아닌 일반적인 타입, Box, Arc같은 스마트 포인터를 지정할 수도 있다.
// 당연한 얘기지만 참조가 아닌 일반적인 타입으로 지정하면 함수 호출 시 (호출자가) 소유권을 빼앗는다.
// 여기서는 impl로 메서드를 정의하거나 trait 함수를 구현하는 것을 해당 type으로 구현한다고 말할 것임.
//
//
// 2.3.7 trait
// java의 interface + Haskell의 type class를 혼합한 기능. trait으로 구현한 주요 기능 중 Ad-Hoc polymorphism
// 이 있음. 애드혹 다형성은 다른 함수를 같은 함수명으로 정의하고 이용할 수 있는 특성.
// e.g. u32 type 덧셈과 f32 type 덧셈의 실제 처리는 다르지만 동일하게 + 연산자를 이용해서 덧셈을 할 수 있는 것은
//      애드혹 다형성 덕분. 애드혹 다형성이 없는 OCaml? 정수의 덧셈 연산자는 + 부동소수점수의 덧셈 연산자는 +. 이다.
trait Adds<RHS = Self> {
    // 1
    type Output; // 2
    fn adds(self, rhs: RHS) -> Self::Output; // 3
} // std lib인 Add trait을 나타낸 것으로, Add trait을 구현한 타입은 +연산자를 이용할 수 있게 됨.
  // 1) Add trait 정의. 이 trait은 제네릭으로 되어 있어 type arg를 받음. RHS가 type arg이고,
  // Self가 기본 type arg이며, type arg가 지정되지 않으면 RHS는 Add trait을 구현한 type(impl Add<RHS=Self> for [...])과 같다.
  // 2) 이 trait 안에서 이용하는 type을 정의함.
  // 3) 구현할 add 함수 타입을 정의한다.
use std::ops::Add; // 1

#[derive(Copy, Clone)]
struct Vec3 {
    x: f64,
    y: f64,
}

impl Add for Vec3 {
    // 2
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

pub fn my_func10() {
    let v1 = Vec3 { x: 10.0, y: 5.0 };
    let v2 = Vec3 { x: 3.1, y: 8.7 };
    let v = v1 + v2; // + 연산자를 사용할 수 있음. v1과 v2의 소유권은 이동 3
    println!("v.x = {}, v.y = {}", v.x, v.y);
}
// 1) Add trait을 표준 라이브러리에서 import
// 2) Vec3 type을 위한 Add trait 구현, Output type과 add 함수를 정의.
// 3) add 함수 호출
//
// 3에서 소유권 이동이 발생하므로 v1과 v2는 더이상 사용할 수 없음. trait을 구현한 커스텀 타입인 Vec3가 copy트레잇을
// 구현하지 않기 때문(기본 타입들은 대부분 copy trait을 구현함). copy trait을 위와 같이 구현하는 것도 방법이지만
// derive attribute를 통해 간단하게 구현할 수 있음 Vec3 struct에 추가해보자. #[derive(Copy, Clone)]
// Eq를 구현하려면 PartialEq가 필요한 것처럼, Copy를 구현하려면 Clone도 필요함. 같이해줘야함.
//
// 어떤 trait을 구현한 객체를 대상으로 하는 제네릭 함수는 trait constraint(제약)라 불리는 기능을 이용해 구현한다.
// fn add_3times<T: Add<Output = T> + Copy>(a: T) -> T 와 같음
fn add_3times<T>(a: T) -> T
where
    // where 문법은 가독성을 위한 것으로 이렇게 하지않으면 위처럼 해야함
    T: Add<Output = T> + Copy, // 1
{
    a + a + a
}
// 1) where에서 type arg T의 trait constraint를 명시, 여기에서 type T는 Add와 Copy trait을 구현하고 있으며
//    Add trait 안의 Output type은 T로 하고 있음. 이렇게 함으로써 add_3times는 Add와 Copy trait을 구현한
//    타입에만 적용할 수 있음. 즉 Add와 Copy trait을 구현하지 않은 타입은 이 함수의 인자로 쓸 수 없다.
//
//
// 2.3.8. ? operator와 unwrap()
// Rust의 에러 처리는 Option type 또는 Result type를 이용해서 수행하지만, 모든 에러 판정을 패턴 매칭
// 으로 수행하면 코드가 장황해짐. 그래서 간략하게 표기할 수 있도록 ? 연산자와 unwrap()함수를 제공(expect()도 있음).
//
// fn option_result() {
//     let a = get(expr)?; // 1
//
//     // get 함수가 Option type을 반환하는 경우
//     // 위 ? 연산자는 다음 패턴매칭과 동일함.
//     let a = match get(expr) { // 2
//         Some(e) => e,
//         None => return None,
//     };
//
//     // get 함수가 Result type을 반환하는 경우
//     // 위 ? 연산자는 다음 패턴 매칭과 동일함.
//     let b = match get(expr) { // 3
//         Ok(e) => e,
//         Err(e) => return Err(e).unwrap(),
//     };
// }
// 즉, ? 연산자는 match와 reutrn의 syntactic sugar(통사론적 설탕)이다. ? 연산자는 편리하니 기억해둘 것.
//
// Rust에서는 Option type이나 Result type 등에 unwrap() 함수를 구현하는 경우가 대다수이며, 성공해 값을 꺼낼 수
// 있으면 꺼내고, 꺼낼 수 없으면 panic으로 종료시키는 작동을 기술할 수 있음.
// fn exam_unwrap() {
//     let a = get(expr).unwrap(); // 1
//
//     // get 함수가 Option type을 반환하는 경우
//     // 위 unwrap 함수 호출은 다음 패턴 매칭과 동일.
//     let a = match get(expr) { // 2
//         Some(e) => e,
//         None => { panic!() },
//     };
//
//     // get 함수가 Result type을 반환하는 경우
//     // 위 unwrap 함수 호출은 다음 패턴 매칭과 동일.
//     let a = match get(expr) { // 3
//         Ok(e) => e,
//         Err(e) => { panic!() },
//     };
// }
//
//
// 2.3.9 thread
use std::thread::spawn; // 1

fn hello() { // 2
    println!("Hello World!");
}

pub fn my_func11() {
    // spawn(hello()).join(); // 3

    let h = || println!("Hello World!"); // 4
    spawn(h).join();
}
// 3) spawn 함수를 호출해 스레드 생성, spawn 함수의 인수에는 hello라는 함수 포인터를 전달하므로 다른 스레드에서
//    Hello World!가 표시된다. Rust의 스레드는 기본적으로 attach thread이므로 join할 필요는 없지만 join 함수를
//    이용해 스레드가 종료되기까지 대기할 수 있음.
// 4) closure를 이용해도 스레드를 생성할 수 있음.

pub fn my_func12() {
    let v = 10;
    let f = move || v * 2; // 1

    // Ok(10 * 2)를 얻음.
    let result = spawn(f).join(); // 2
    println!("result = {:?}", result); // Ok(20)이 표시됨

    // 스레드가 panic인 경우 Err(패닉값)을 얻을 수 있음.
    match spawn(|| panic!("I'm panicked!")).join() { // 3
        Ok(_) => { // 4
            println!("successed");
        }
        Err(a) => { // 5
            let s = a.downcast_ref::<&str>();
            println!("failed: {:?}", s);
        }
    }
}
// 1) 스레드 생성을 위해 클로저 정의. Rust의 스레드는 값을 반환할 수 있음.
// 2) 정의한 클로저를 spawn함수에 전달해 스레드 생성. 스레드의 반환값은 join함수의 반환값에 포함됨. 단, join함수의
//    반환값은 Result type이므로 실제로는 Ok(20)을 포함해 반환함.
// 3) 스레드가 패닉에 빠져 종료한 예. panic! 매크로를 호출해 스레드를 패닉으로 만드는 클로저를 spawn함수에 전달해
//    스레드를 생성하고 join함.
// 4) 스레드가 올바르게 종료된 경우의 처리
// 5) 스레드가 패닉이 된 경우 join 함수의 반환값에 Result type의 Err에 패닉 시의 값이 포함됨. Err에 포함된 값의
//    type은 어던 type도 될수 있는 Any라 불리는 특수한 type. 이 Any type으로부터 println! 함수에 전달하기 위해
//    &str type으로 cast해서 표시한다.
// 이렇게 함으로써 스레드의 반환값 또는 패닉에 빠졌을 때의 반환값을 얻을 수 있음.
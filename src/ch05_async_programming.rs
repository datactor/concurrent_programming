/// ch05 비동기 프로그래밍
/// 어떤 일을 수행하는 도중에 발생하는 일을 'event' 또는 'interrupt'라고 부른다. Rust나 C같은 소위 절차적 프로그래밍
/// (procedural programming) 언어에서는 기본적으로 처리는 실행 순서대로 기술해야 한다. 처리를 실행 순서대로 기술하지
/// 않으면 전화가 왔을 때 책을 중단하고 전화를 받도록 기술하는 것이 어렵고, 책 읽기를 마친 뒤 전화를 받도록 기술해야 한다.
/// 이렇게 기술하게 되면 중요한 전화를 놓치게 된다.
///
/// 작성한 순서대로 작동하는 프로그래밍 모델을 동기 프로그래밍(synchronous programming)이라 부른다. 비동기 프로그래밍은
/// 독립해서 발생하는 이벤트에 대한 처리를 기술하기 위한 동시성 프로그래밍 기법을 총칭한다. 비동기 프로그램의 기법을
/// 이용함으로써 전화가 울리면 전화를 받고, 택배가 도착하면 택배를 받는 것과 같이 이벤트에 대응한 작동을 기술할 수 있다.
/// 비동기 프로그램에서는 어떤 순서로 실행되는 가는 코드에서 판별할 수 없으며, 처리 순서는 이벤트 발생 순서에 의존한다.
///
/// 비동기 프로그램을 구현하는 방법으로 callback함수나 시그널(interrupt)을 이용하는 방법이 있으나
/// 이 장에서는 특히 OS에 의한 IO 대중화 방법과 현재 많은 프로그래밍 언어에서 채용하고 있는 비동기 프로그래밍 방법인
/// Future, async/await부터 살펴보자. https://rust-lang.github.io/async-book/01_getting_started/02_why_async.html
/// 이후 Rust의 async/await을 이용한 비동기 라이브러리의 '실질적 표준'(std는 아님)인 Tokio을 이용한 비동기 프로그래밍
/// 의 예를 살펴보자
/// 먼저 futures"0.3.13"와 nix"0.20.0" crate를 dependencies에 가져오자

/// 5.1 동시 서버
/// 이 절에서는 반복 서버(interactive server)와 동시 서버(concurrent server, 병행 서버라고도 함)를 알아보고 그 구현을 짚어본다.
/// interactive server: client로부터 요청받은 순서대로 처리하는 서버
/// concurrent server: 요청을 동시에 처리하는 서버
/// 예를 들어 편의점에서 도시락을 데워줄 때를 생각해보면, 일반적으로
/// 편의점 점원은 A고객의 도시락을 데우고, 도시락이 데워지는 동안 다른 고객인 B의 물품을 계산한다.
/// 이렇게 A고객의 업무를 처리하는 동시에 다른 처리를 수행하는 서버를 동시 서버(concurrent server)라 부르며 A고객의
/// 도시락이 데워지는 것을 기다렸다가 도시락이 다 데워진 후 B 고객의 업무를 처리하는 것을 반복 서버(interactive server)라 부른다.
///
/// 다음 코드는 단순한 interactive server를 구현한 예다. 이 서버는 client로부터의 connection request를 받아
/// 1행씩 읽으면서 읽은 데이터를 return하고 connection을 종료하는 작동을 반복한다.
/// 이렇게 읽은 데이터에 대한 응답만 하는 서버를 echo server라 부른다.
#[test]
pub fn func_178p() {
    use std::{
        io::{BufRead, BufReader, BufWriter, Write},
        net::TcpListener
    };

    // TCP 10000번 포트를 listening
    let listener = TcpListener::bind("127.0.0.1:10000").unwrap(); // 1

    // connection request accept(ack)
    while let Ok((stream, _)) = listener.accept() { // 2
        // 읽기, 쓰기 객체 생성
        let stream0 = stream.try_clone().unwrap();
        let mut reader = BufReader::new(stream0);
        let mut writer = BufWriter::new(stream);

        // 1행씩 읽어 echo
        let mut buf = String::new();
        reader.read_line(&mut buf).unwrap();
        println!("1: {}", writer.buffer().len());
        writer.write(buf.as_bytes()).unwrap(); // writer에 byte code로 쓰고
        println!("2: {}", writer.buffer().len());
        writer.flush().unwrap(); // 버퍼링되어 있는 데이터를 모두 송신함
        println!("3: {}", writer.buffer().len());
    }
}
// connection request를 받으면 client로부터 데이터를 수신하고, 송신 처리를 완료하지 않으면 다음 클라이언트의
// 처리를 수행하지 못함(flush로 비워내야함 실패시 에러)
// 즉 먼저 도착한 connection client를 A라고 하면 A의 처리를 종료할 때까지 다음 client인 B의 처리는 아무것도 실행하지 않음.
// 만약 A의 데이터 전송이 B보다 매우 느린 경우에는 B를 먼저 처리하는 편이 전체적으로 처리량을 향상시킬 수 있지만
// 반복서버(interaction server)는 그런 처리를 하지 않음.
//
// 이 서버로의 접속은 telnet 또는 socat을 이용해서 가능함
// $telnet localhost 10000
// Trying 127.0.0.1...
// Connected to localhost.
// Escape character is '^]'.
// hi rust
// hi rust
// Connection closed by foreign host.
//
/// concurrent server는 client로부터의 connection request, data arriving 등의 처리를 event 단위로 세세하게
/// 분류하여 event에 따라 처리를 실행할 수 있다.
///
/// 네트워크 소켓이나 파일 등의 IO event 감시 시스템 콜
/// - 유닉스 계열의 OS: select나 poll - OS에 의존하지 않고 이용할 수 있지만 속도가 느림.
/// - 리눅스: epoll - 속도가 빠르지만 OS에 의존함.
/// - BSD 계열 OS: kqueue - 속도가 빠르지만 OS에 의존함
///
/// IO event 감시는 파일 descriptor를 감시하는 것이다. 예를 들어 여러 TCP connection이 존재할 경우 server는
/// 여러 파일 descriptor를 가진다. 이들 파일 descriptor에 대해 읽기나 쓰기 가능 여부를 select 등의 함수를 이용해
/// 판정할 수 있다. 다음 그림은 epoll, kqueue, select의 동작 개념을 보여준다(180p 그림 5-1).
/// 그림에서는 프로세스(유저랜드)에서 IO event 감시 시스템 콜을 이용해 커널 내부로 들어가 프로세스 관련 파일 descriptor
/// 정보들을 이용해 IO event 감시 시스템 콜을 통한 파일 descriptor 감시를 수행한다. 해당 파일 descriptor를
/// 읽고 쓰기가 가능하게 된 경우 IO event 감시 시트템 콜을 호출하고 반환한다. 그리고 이 함수들은 읽기만 감시, 쓰기만
/// 감시, 읽기와 쓰기 모두 감시 등을 상세히 지정할 수 있다.
///
/// 다음 코드는 epoll(리눅스 IO event 감시 시스템 콜)을 이용한 병렬 서버 구현 예다. 작동상으로는 앞의 코드와 거의
/// 비슷하지만 동시에 작동하면서 송수신을 반복하도록 되어 있다는 점이 다르다. 이 코드는 non-blocking 설정을 수행하지
/// 않으므로 구현이 완성되지 않았지만, 이 부분은 뒤에서 설명할 버전에서 마무리 할 것이다.
///
/// - blocking이란 송수신 준비가 되지 않은 상태에서 송수신 함수를 호출하면 해당 함수 호출을 정지하고 송수신 준비가 되었을
/// 때 재개하는 작동을 말한다. 송수신 준비가 되지 않은 경우에 송수신 함수가 호출되면 OS는 그 함수들을 호출한 OS 프로세스를
/// 대기 상태로 만들고, 다른 OS 프로세스를 실행한다.
/// - non-blocking이면 송수신할 수 없는 경우 즉시 함수에서 반환되므로 송수신 함수를 호출해도 OS 프로세스는 대기 상태가 되지 않는다.
#[test]
pub fn func_181p() {
    use nix::sys::epoll::{
        epoll_create1, epoll_ctl, epoll_wait, EpollCreateFlags, EpollEvent, EpollFlags, EpollOp
    };
    use std::collections::HashMap;
    use std::io::{BufRead, BufReader, BufWriter, Write};
    use std::net::TcpListener;
    use std::os::unix::io::{AsRawFd, RawFd};

    // epoll 플래그 단축 계열
    let epoll_in = EpollFlags::EPOLLIN;
    let epoll_add = EpollOp::EpollCtlAdd;
    let epoll_del = EpollOp::EpollCtlDel;

    // TCP 10000번 포트 리슨
    let listener = TcpListener::bind("127.0.0.1:10000").unwrap();

    // epoll용 객체 생성. epoll에서는 감시할 socket(파일 descriptor)을 epoll용 객체에 등록한 뒤
    // 감시 대상 event가 발생할 때까지 대기하고 이벤트 발생 후 해당 이벤트에 대응하는 처리를 수행한다.
    // epoll 객체 생성은 epoll_create1 함수로 하고, 삭제는 close함수로 한다.
    let epfd = epoll_create1(EpollCreateFlags::empty()).unwrap();

    // 생성한 epoll 객체에 listen용 소켓을 감시 대상으로 등록함.
    // connection request 도착 감시는 event 종류를 EPOLLIN으로 설정해서 수행한다.
    let listen_fd = listener.as_raw_fd(); // 여기서 fd는 file descriptor
    let mut ev = EpollEvent::new(epoll_in, listen_fd as u64);
    // epoll_ctrl 함수는 감시 대상 추가, 삭제, 수정을 하는 함수다.
    epoll_ctl(epfd, epoll_add, listen_fd, &mut ev).unwrap();

    let mut fd2buf = HashMap::new();
    let mut events = vec![EpollEvent::empty(); 1024];

    // epoll_wait 함수로 event 발생을 감시. 이 함수에서는 두 번째 인수에 전달된 슬라이스에 event가 발생한 파일 descriptor가
    // 쓰여지고, 발생한 event 수를 Option type으로 반환한다. 세 번째 인수는 timeout 시간이며 밀리초 단위로 지정 가능.
    // 단 세 번째 인수에 -1을 전달하면 timeout되지 않는다.
    while let Ok(nfds) = epoll_wait(epfd, &mut events, -1) {
        for n in 0..nfds { // event가 발생한 file descriptor에 대해 순서대로 처리를 수행한다.
            let event_data = events[n].data();
            // 여기서 처리를 listen socket의 event와 client socket의 event로 분리한다.
            if event_data == listen_fd as u64 { // listen socket의 event일 경우
                // listen용 socket 처리. 먼저 file descriptor를 취득하고 읽기 쓰기용 객체를 생성한 뒤 epoll_ctl함수로
                // epoll에 읽기 event를 감시 대상으로 등록한다.
                if let Ok((stream, _)) = listener.accept() {
                    // 읽기, 쓰기 객체 생성
                    let fd = stream.as_raw_fd(); // raw fd로 key를 만들기 위해 fd를 borrow
                    let stream0 = stream.try_clone().unwrap(); // 읽기, 쓰기 객체를 분리하기 위한 clone()
                    let reader = BufReader::new(stream0); // 읽기 객체 생성
                    let writer = BufWriter::new(stream); // 쓰기 객체 생성

                    // fd와 reader, writer의 관계를 만듬
                    fd2buf.insert(fd, (reader, writer));

                    println!("accept: fd = {}", fd);

                    // fd를 감시 대상에 등록하기 위해 epollevent 객체 생성
                    let mut ev = EpollEvent::new(epoll_in, fd as u64);
                    // fd를 감시 대상에 등록
                    epoll_ctl(epfd, epoll_add, fd, &mut ev).unwrap();
                }
            } else { // client socket의 event일 경우
                // client용 소켓 처리. client에서 데이터 도착한다면 먼저 1행을 읽는다. 이때 connection이 close 상태면
                // read_line()의 값은 0이 되므로 connection close 처리를 수행한다. 이와 같이 epoll의 감시 대상에서
                // event를 제외하려면 epoll_ctl 함수에 EpollCtlDel을 지정한다.
                let fd = event_data as RawFd;
                let (reader, writer) = fd2buf.get_mut(&fd).unwrap();

                // 1행 읽기
                let mut buf = String::new();
                let n = reader.read_line(&mut buf).unwrap();

                // connection을 close한 경우 epoll 감시 대상에서 제외한다.
                if n == 0 {
                    let mut ev = EpollEvent::new(epoll_in, fd as u64);
                    epoll_ctl(epfd, epoll_del, fd, &mut ev).unwrap();
                    fd2buf.remove(&fd); // connection이 close 상태일 경우 buf에 데이터가 없기 때문에, fd2buf에서 fd를 지워버림
                    println!("closed: fd = {}", fd);
                    continue
                }

                print!("read: fd = {}, buf = {}", fd, buf);

                // n이 0이 아닐 경우 읽은 데이터를 그대로 쓴다.
                writer.write(buf.as_bytes()).unwrap();
                writer.flush().unwrap();
            }
        }
    }
}
// epoll에서는 감시할 file descriptor를 등록하고, 그 file descriptor에 대해 읽기나 쓰기 등을 할 수 있는 상태가 되면
// epoll 호출을 반환한다. API는 다소 다르지만 select, poll, kqueue에서도 거의 비슷하게 수행한다.
// 이렇게 epoll이나 select 등 여러 IO에 대해 동시에 처리를 수행하는 방법을 IO 다중화(I/O multiplexing)라 부른다.
// IO 다중화를 기술하는 방법론의 하나로 이 코드에서 기술한 것처럼 event에 대해 처리를 기술하는 방법이 있다. 이런 프로그래밍 모델,
// design pattern을 이벤트 주도(event-driven)라 부르며, event-driven programming 역시 비동기 프로그래밍으로 간주한다.
//
// 유명한 event-driven library로는 libevent와 libev가 있다. 이들 라이브러리는 C언어에서 이용할 수 있는 library이며
// epoll이나 kqueue를 추상화한 것이므로 OS에 의존하지 않고 소프트웨어를 구현할 수 있다.
// 이들 라이브러리는 file descriptor에 대해 콜백 함수를 등록함으로써 concurrent programming을 구현한다.
// 그리고 POSIX에서도 AIO(Asynchronous IO)라 불리는 API가 존재한다. POSIX AIO에서는 2종류의 비동기 프로그래밍 방법을
// 선택할 수 있다. 한 가지는 대상이 되는 file descriptor에 대해 callback 함수를 설정하고 event 발생 시 스레드가 생성되어
// 그 함수가 실행되는 방법이다. 다른 한 가지는 signal로 알리는 방법이다.

/// 5.2 코루틴과 스케줄링
/// 이 절에서는 코루틴을 스케줄링하는 방법을 알아보자. 코루틴을 이용함으로써 비동기 프로그래밍을 보다 추상적으로 기술함을 목표로 한다.
///
/// 5.2.1 couroutine
/// 코루틴은 다양한 의미로 사용되지만 여기서는 중단과 재개가 가능함 함수를 총칭하는 것으로 하자.
/// 코루틴을 이용하면 함수를 임의의 시점에 중단하고, 중단한 위치에서 함수를 재개할 수 있다. 코루틴이라는 용어는
/// 1963년 Conway의 논문에 등장했으며 COBOL과 ALGOL 프로그래밍 언어에 적용되었다.
///
/// 현재 코루틴은 대칭 코루틴(symmetric coroutine과 비대칭 코루틴(asymmetric coroutine)으로 분류된다.
/// - 대칭 코루틴? 함수는 routine과 sub routine이라는 주종관계가 일반적인데 서로 동등한 대칭 관계의 루틴을 말함.
/// 다음 코드는 대칭 코루틴을 의사 코드로 기술한 예다.
/// courtine A {
///     // 무언가 처리
///     yield to B 2
///     // 무언가 처리
///     yield to B 4
/// }
/// coroutine B {
///     // 무언가 처리
///     yield to A 3
///     // 무언가 처리
/// }
///
/// yield to A 1
/// 1) A 호출
/// 2) B 호출. 처리는 여기서 중단
/// 3) A의 도중부터 재개. 처리는 여기서 중단
/// 4) B의 도중부터 재개
///
/// 대칭 코루틴(symmetric couroutine)에서는 재개하는 함수명을 명시적으로 지정해서 함수 중단과 재개를 수행한다.
/// 가장 마지막 행에서 코루틴 A가 실행되어 무언가 처리를 수행하고, yield to B로 코루틴 B의 처리를 시작한다.
/// 코루틴 B가 실행되면 이번에는 yield to A로 코루틴 A로 처리가 옮겨진다. 이 때 코루틴 A 안의 yield에 의해 중단된
/// 위치부터 처리가 재개된다. 그 후 다시 코루틴 A의 두 번째 yield to B까지 실행되고 코루틴 B의 yield부터 처리가 재개된다.
/// 일반적인 함수 호출은 호출원과 호출되는 측이라는 주종 관계가 있지만 대칭 코루틴에서는 서로 동등한 대칭 관계가 된다.
///
/// 다음 코드는 비대칭 코루틴의 예를 Python으로 나타낸 것이다. Pytgon에서는 비대칭 코루틴을 generator라고 부르며,
/// 뒤에서 나올 async/await로 scheduling 가능하도록 수정된 특수한 코루틴을 코루틴이라 부른다.
/// def hello():
///     print('Hello,', end='')
///     yield # 여기서 중단, 재개 1
///     print('World!')
///     yield # 여기까지 실행 2
/// h = hello()  # 1까지 실행
/// h.__next__() # 1부터 재개하여 2까지 실행
/// h.__next__() # 2부터 재개
///
/// 위 코드는 Hello, World!를 출력할 뿐이지만 yield로 함수의 중단과 재개가 수행된다. yield를 호출하면 함수를 지속하면서
/// 호출할 객체가 반환되고 해당 객체에 대해 __next__함수를 호출함으로써 지속할 위치부터 재개할 수 있다.
///
/// Rust에는 코루틴은 없지만 코루틴과 같은 작동을 하는 함수를 상태를 기다리는 함수로 구현할 수 있다. 다음 코드는 Python의
/// 코루틴 버전 Hello, World!를 Rust로 구현한 것이다. Rust에는 Future trait이라 불리는 비동기 trait이 있으므로
/// https://rust-lang.github.io/async-book/02_execution/02_future.html
/// 여기에서 future를 사용해 보자.
#[test]
pub fn func_186p() {
    use futures::future::{BoxFuture, FutureExt};
    use futures::task::{waker_ref, ArcWake};
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::{Arc, Mutex};
    use std::task::{Context, Poll};

    struct Hello { // 함수의 상태와 변수를 저장하는 Hello type 정의.
        state: StateHello, // Hello, World!에는 변수가 없으므로 함수의 실행 위치 상태만 필드로 가진다.
    }

    // 함수의 실행 상태를 나타내는 StateHello type.
    enum StateHello {
        HELLO, // 초기 상태는 Hello 상태고
        WORLD, // Python version의 첫 번째 yield를 나타내는 상태가 WORLD 상태
        END,   // 두 번째 yield를 나타내는 상태가 END 상태가 된다.
    }

    impl Hello {
        fn new() -> Self {
            Hello {
                state: StateHello::HELLO, // 초기 상태
            }
        }
    }

    impl Future for Hello {
        type Output = ();

        // poll 함수가 실제 함수 호출(Python에서 h = hello()). 인수의 Pin type은 Box등과 같은 type(https://rust-lang.github.io/async-book/04_pinning/01_chapter.html)
        // Pin type은 내부적인 메모리 복사로의 move를 할 수 없어서 주소 변경을 할 수 없는 type이지만 이것은 Rust 특유의 성질에 속한다.(unpinn을 구현해야함)
        // _cx는 Waker 및 future의 내부구조부터 파악하고 뜯어 보길 바란다.
        fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
            match (*self).state {
                StateHello::HELLO => {
                    println!("Hello, ");
                    // WORLD 상태로 전이
                    (*self).state = StateHello::WORLD;
                    Poll::Pending // 다시 호출 가능
                }
                StateHello::WORLD => {
                    println!("World!");
                    // END 상태로 전이
                    (*self).state = StateHello::END;
                    Poll::Pending // 다시 호출 가능
                }
                StateHello::END => {
                    Poll::Ready(()) // 종료
                }
            }
        }
    }
    // 이 구현에서 알 수 있듯이 poll 함수에서는 함수의 상태에 따라 필요한 코드를 실행하고 내부적으로 상태 전이를 수행한다.
    // 함수가 재실행 가능한 경우 poll 함수는 Poll::Pending을 반환하고, 모두 종료한 경우 Poll::Ready에 반환값을 감싸서 반환한다.

    // 실행 단위. Task type은 async/await에서 프로세스의 실행 단위이고, ArcWake는 프로세스를 scheduling 하기 위한 trait.
    struct Task {
        hello: Mutex<BoxFuture<'static, ()>>,
    }

    impl Task {
        fn new() -> Self {
            let hello = Hello::new();
            Task {
                hello: Mutex::new(hello.boxed()),
            }
        }
    }

    // 아무것도 하지 않음
    impl ArcWake for Task {
        fn wake_by_ref(_arc_self: &Arc<Self>) {}
    }

    // 초기화
    let task = Arc::new(Task::new());
    let waker = waker_ref(&task);
    let mut ctx = Context::from_waker(&waker); // poll 함수를 실행하려면 Context type값이 필요하므로 여기에서는
    // 아무것도 하지 않는 Task type을 정의하고 거기에 ArcWake trait을 구현했다. Context type의 값은 ArcWake 참조로부터
    // 생성할 수 있다.
    let mut hello = task.hello.lock().unwrap();

    // 정지와 재개 반복. poll 함수를 3번 호출하면 최종적으로 Hello type의 poll 함수가 실행되어 Hello, World!가 표시된다.
    // 이것은 Python 버전 코드와 그 작동이 완전히 같다.
    hello.as_mut().poll(&mut ctx);
    hello.as_mut().poll(&mut ctx);
    hello.as_mut().poll(&mut ctx);
}
// 이렇게 코루틴이 프로그래밍 언어 사양이 아니어도 동등하게 작동하는 함수를 구현할 수 있다. 코루틴을 이용하면 비동기 프로그래밍을
// 보다 고도로 추상화해 간략하게 기술할 수 있다. 이 절 이후에는 이러한 코루틴 구조를 알아보자.

/// 5.2.2 scheduling
/// 비대칭 코루틴을 이용하면 중단된 함수를 프로그래머 측에서 자유롭게 재개할 수 있으며, 이 중단과 재개를 스케줄링해서 실행할
/// 수도 있다. 이렇게 하면 정밀도가 높은 제어는 할 수 없지만 프로그래머는 코루틴 관리에서 해방되어 보다 추상도가 높은 동시 계산을
/// 기술할 수 있다. 이 절에서는 코루틴을 스케줄링해서 실행하는 방법을 알아보자.
/// 구현에 앞서 선행되어야 할 사전 지식이 있다. 먼저 구현할 역할을 알아보자.
/// 역할은 크게 Executor, Task, Waker 세가지로 나뉜다.
///
///                                wake
/// Executor <------- 실행 Queue <------- Waker[Task 정보, ...]
///      \                                      /
///  poll \                                   /
///         ↘                               ↙
///       Task[Future[Future, Future, ...], ...]
///
/// - Task가 스케줄링의 대상이 되는 계산의 실행 단위인 '프로세스'에 해당한다.
/// - Executor는 실행 가능한 Task를 적당한 순서로 실행(Task 안의 Future를 poll)
/// - Waker는 Task를 스케줄링할 때 이용된다(Task에 대한 정보를 가진 Waker가 필요에 따라 실행 Queue에 Task를 넣음).
/// 위 그림 및 작동방식은 전형적인 예이며, 다른 실행방법도 가능하다.
/// 이 장에서는 Waker와 Task를 동일 type으로 구현한다.
#[test]
pub fn func_189p() {
    use futures::future::{BoxFuture, FutureExt};
    use futures::task::{waker_ref, ArcWake};
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::mpsc::{sync_channel, Receiver, SyncSender}; // 통신 채널을 위한 함수와 타입. 채널을
    // 경유하면 스레드 사이에서 데이터 송수신을 수행할 수 있다. Rust에서는 많은 채널 구현에서 송신단과 수신단을 구별하며,
    // Receiver와 SyncSender type이 수신과 송신용 endpoint의 type이 된다. mpsc는 말 그대로 송신은 여러 스레드에서.
    // 수신은 단일 스레드에서만 가능한 채널이다.
    use std::sync::{Arc, Mutex};
    use std::task::{Context, Poll};

    // 간략화를 위해 Task 자체를 Waker로 구현
    struct Task {
        // 실행하는 코루틴
        future: Mutex<BoxFuture<'static, ()>>, // 실행할 코루틴(Future). Future의 실행을 완료할 때까지
                                               // Executor가 실행을 수행한다.
        // Executor에 스케줄링하기 위한 채널
        sender: SyncSender<Arc<Task>>, // Executor로 Task를 전달하고 스케줄링을 수행하기 위한 채널
    }

    impl ArcWake for Task {
        fn wake_by_ref(arc_self: &Arc<Self>) { // 자신의 Arc 참조를 Executor로 송신하고 스케줄링한다.
            // 자신을 스케줄링
            let self0 = arc_self.clone(); // 송신은 여러 스레드에서 할 것이기 때문에 참조 카운트 업
            arc_self.sender.send(self0).unwrap();
        }
    }
    // 이렇게 Task는 실행할 코루틴을 저장하고 자신을 스케줄링 가능하도록 ArcWake trait을 실행한다. 스케줄링은
    // 단순히 Task로의 Arc 참조를 채널로 송신(실행 Queue에 넣음)한다.

    // Task의 실행을 수행하는 Executor를 구현해보자. 여기서 구현한 Executor는 단일 채널에서 실행 가능한 Task를 받아
    // Task 안의 Future를 poll하는 단순한 것이다.
    struct Executor { // Executor type은 단순히 Task를 송수신하는 채널(실행 Queue)의 endpoint를 저장한다.
        // 실행 Queue
        sender: SyncSender<Arc<Task>>,
        receiver: Receiver<Arc<Task>>,
    }

    impl Executor {
        fn new() -> Self {
            // 채널 생성. Queue의 사이즈는 최대 1024
            let (sender, receiver) = sync_channel(1024);
            Executor {
                sender: sender.clone(), // mp 다중 송신. 참조 증가
                receiver, // sc 단일 수신
            }
        }

        // 새롭게 Task를 생성하고 실행 Queue에 넣기위한 객체를 반환함. spawn 함수에 해당하는 작동을 수행하기 위한 객체.
        fn get_spawner(&self) -> Spawner {
            Spawner {
                sender: self.sender.clone(), // 참조 증가.
            }
        }

        fn run(&self) { // 채널에서 Task를 수신해서 순서대로 실행한다. 이번 구현에서는 Task와 Waker가 같으므로
                        // Task에서 Waker를 생성하고 Waker에서 Context를 생성한 뒤 context를 인수로 poll() 호출
            while let Ok(task) = self.receiver.recv() {
                // context 생성
                let mut future = task.future.lock().unwrap();
                let waker = waker_ref(&task); // 수신한 task(future)로 waker_ref를 만듬
                let mut ctx = Context::from_waker(&waker); // waker_ref로부터 context를 만듬
                // poll을 호출해서 실행
                let _ = future.as_mut().poll(&mut ctx);
            }
        }
    }
    // context는 실행 상태를 저장하는 객체이며 Future 실행 시 이를 전달해야 한다.
    // Rust의 context는 내부에 Waker 및 _marker(lifetime을 명시해 수명을 불변으로 강제하여 분산 변경에 대한
    // future를 보장함 (phantomdata))를 가지고 있다. 이번 구현에서는 Waker와 Task가 같으므로 context에서 Waker를
    // 꺼낼 때 Task가 꺼내진다.

    // 다음 코드는 Task를 생성하는 Spawner type의 정의와 구현이다. Spawner는 Future를 받아 Task로 감사서
    // 실행 Queue에 넣기 위한(channel로 송신) type이다.
    struct Spawner { // 단순히 실행 Queue에 추가하기 위해 channel의 송수신 endpoint를 저장할 뿐이다.
        sender: SyncSender<Arc<Task>>,
    }

    impl Spawner {
        // Task를 생성해서 실행 Queue에 추가한다. 이 함수는 Future를 받아 Box화해서 Task에 감싸서 실행 Queue에 넣는다.
        fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
            let future = future.boxed();  // Future를 Box화
            let task = Arc::new(Task {  // Task 생성
                future: Mutex::new(future),
                sender: self.sender.clone(),
            });

            // 실행 Queue에 인큐
            self.sender.send(task).unwrap();
        }
    }


    // 실행을 위한 구조체, impl block
    struct Hello { // 함수의 상태와 변수를 저장하는 Hello type 정의.
        state: StateHello, // Hello, World!에는 변수가 없으므로 함수의 실행 위치 상태만 필드로 가진다.
    }

    // 함수의 실행 상태를 나타내는 StateHello type.
    enum StateHello {
        HELLO, // 초기 상태는 Hello 상태고
        WORLD, // Python version의 첫 번째 yield를 나타내는 상태가 WORLD 상태
        END,   // 두 번째 yield를 나타내는 상태가 END 상태가 된다.
    }

    impl Hello {
        fn new() -> Self {
            Hello {
                state: StateHello::HELLO, // 초기 상태
            }
        }
    }

    impl Future for Hello {
        type Output = ();

        // poll 함수가 실제 함수 호출(Python에서 h = hello()). 인수의 Pin type은 Box등과 같은 type(https://rust-lang.github.io/async-book/04_pinning/01_chapter.html)
        // Pin type은 내부적인 메모리 복사로의 move를 할 수 없어서 주소 변경을 할 수 없는 type이지만 이것은 Rust 특유의 성질에 속한다.(unpinn을 구현해야함)
        // _cx는 Waker 및 future의 내부구조부터 파악하고 뜯어 보길 바란다.
        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
            match (*self).state {
                StateHello::HELLO => {
                    print!("Hello, ");
                    // WORLD 상태로 전이
                    (*self).state = StateHello::WORLD;
                    // 추가
                    cx.waker().wake_by_ref(); // 자신을 실행 Queue에 넣음
                    return Poll::Pending // 다시 호출 가능
                }
                StateHello::WORLD => {
                    println!("World!");
                    // END 상태로 전이
                    (*self).state = StateHello::END;
                    // 추가
                    cx.waker().wake_by_ref(); // 자신을 실행 Queue에 넣음
                    return Poll::Pending // 다시 호출 가능
                }
                StateHello::END => {
                    return Poll::Ready(()) // 종료
                }
            }
        }
    }
    // 이 구현에서 알 수 있듯이 poll 함수에서는 함수의 상태에 따라 필요한 코드를 실행하고 내부적으로 상태 전이를 수행한다.
    // 함수가 재실행 가능한 경우 poll 함수는 Poll::Pending을 반환하고, 모두 종료한 경우 Poll::Ready에 반환값을 감싸서 반환한다.

    // 실행
    // let executor = Executor::new();
    // executor.get_spawner().spawn(Hello::new());
    // executor.run();

    let executor = Executor::new();
    // async로 Future trait을 구현한 type의 값으로 변환
    executor.get_spawner().spawn(async {
        let h = Hello::new();
        h.await; // poll을 호출해서 실행
    });
    executor.run();
    // async로 둘러싸인 처리 부분이 Rust 컴파일러에 의해 Future trait을 구현한 type의 값으로 변환되어 await으로
    // Future trait의 poll 함수를 호출한다. (async block은 impl Future<Output =()>를 반환하는 함수로 바뀜)
    // 즉, async { code }라고 기술하는 경우 Future trait을 구현한 type이 컴파일러에 의해 새롭게 정의되어
    // async { code } 부분에는 해당 type의 new 함수에 해당하는 호출이 이뤄진다.
    // 그리고 그 type이 poll 함수에는 async code 부분이 구현되어 있다.
    //
    // h.await의 의미는 다음과 같은 생략 type이라 보면 된다.
    // match h.poll(cx) {
    //     Poll::Pending => return Poll::Pending,
    //     Poll::Result(x) => x,
    // }
    // 이렇게 함으로써 async, 즉 Future trait의 poll 함수가 중첩해서 호출되는 경우에도 함수의 중단과 값 반환을 적절하게
    // 다룰 수 있다. 즉, poll 함수 호출로 Pending이 반환되는 경우에는 Executor까지 Pending임이 소급되어 전달된다.
}
// 이처럼 Executor의 생성과 spawn에서의 Task 생성을 수행한 뒤 run 함수를 호출함으로써 hello의 코루틴이 마지막까지
// 자동 실행된다. 스케줄링 실행을 수행하면 프로그래머가 코루틴 호출을 고려할 필요가 없으며, 자동으로 코루틴을 실행할
// 수 있게 된다.

/// 5.3 async/await
///
/// https://rust-lang.github.io/async-book/01_getting_started/02_why_async.html
///
/// 5.3.1 Future와 async/await
/// Future는 미래의 언젠가의 시점에서 값이 결정되는(또는 일정한 처리가 종료되는) 것을 나타내는 type으로 lang에 따라
/// promise 또는 eventual이라고 부르기도 한다. Future나 Promise라는 용어가 등장한 것은 1977년경이며, Future는
/// 1985년 MultiLisp 언어에 내장되었고, Promise는 1988년에 언어에 의존하지 않는 기술 방식으로 제안되었다.
/// 사실 지금까지 사용했던 Future trait은 미래 언젠가의 시점에서 값이 결정되는 것을 나타내기 위한 interface를 규정한
/// trait이다. 일반적으로 Future는 coroutine을 이용해 구현되며 이로 인해
/// '중단, 재개 가능한 함수'에서 '미래에 결정되는 값을 표현한 것'으로 의미 전환이 이뤄진다.
/// Future type을 이용한 기술 방법에는 명시적으로 기술하는 방법과 암묵적으로 기술하는 방법이 있다.
/// 암묵적으로 기술하는 경우 Future type은 일반적인 type과 완전히 동일하게 기술되지만
/// 명시적으로 기술할 때는 Future type에 대한 조작은 프로그래머가 기술해야 한다.
/// async/await은 명시적인 Future type에 대한 기술이라고 생각하면 된다.
/// await은 Future type의 값이 결정될 때까지 처리를 정지하고 다른 함수에 CPU 리소스를 양보하기 위해 이용하고,
/// async는 Future type을 포함한 처리를 기술하기 위해 사용한다.
/// NOTE_ 명시적 또는 암묵적으로 기술한다는 것은 참조를 생각해보면 이해하기 쉽다. 예를 들어 &u32 type의 변수 a의 값을
///       참조하기 위해 Rust에서는 *a라고 명시적으로 참조제외를 해야 하지만 a라고 쓰기만 해도 컴파일러가 자동적으로
///       참조 제외를 수행하는 언어 설계도 생각할 수 있다(e.g. Deref Coercion).
/// 예를 들어 앞의 Future trait을 이용한 Hello, World!는 async/await을 이용해 다음과 같이 쓸 수 있다
/// (func_189p의 실행부를 수정해보기)
fn func_194p() {
    // let executor = Executor::new();
    // // async로 Future trait을 구현한 type의 값으로 변환
    // executor.get_spawner().spawn(async {
    //     let h = Hello::new();
    //     h.await; // poll을 호출해서 실행
    // });
    // executor.run();
}
// async로 둘러싸인 처리 부분이 Rust 컴파일러에 의해 Future trait을 구현한 type의 값으로 변환되어 await으로
// Future trait의 poll 함수를 호출한다. (async block은 impl Future<Output =()>를 반환하는 함수로 바뀜)
// 즉, async { code }라고 기술하는 경우 Future trait을 구현한 type이 컴파일러에 의해 새롭게 정의되어
// async { code } 부분에는 해당 type의 new 함수에 해당하는 호출이 이뤄진다.
// 그리고 그 type이 poll 함수에는 async code 부분이 구현되어 있다.
//
// h.await의 의미는 다음과 같은 생략 type이라 보면 된다.
// match h.poll(cx) {
//     Poll::Pending => return Poll::Pending,
//     Poll::Result(x) => x,
// }
//
// 이렇게 함으로써 async, 즉 Future trait의 poll 함수가 중첩해서 호출되는 경우에도 함수의 중단과 값 반환을 적절하게
// 다룰 수 있다. 즉, poll 함수 호출로 Pending이 반환되는 경우에는 Executor까지 Pending임이 소급되어 전달된다.
//
// 비동기 프로그래밍은 콜백을 이용해서도 기술된다. 하지만 콜백을 이용하는 방법은 가독성이 낮아진다. 특히 콜백을 연속해서
// 호출하면 매우 읽기 어려운 코드가 되어 콜백 지옥이라 불리기도 한다. 다음 코드는 콜백 지옥의 예다. 여기서 poll 함수는
// 콜백 함수를 받아 값이 결정되었을 때 해당 콜백 함수에 결과를 전달해서 호출한다고 가정한다.
// x.poll(|a| {
//     y.poll(|b| {
//         z.poll(|c| {
//             a + b + c
//         })
//     })
// })
// 이처럼 콜백 기반의 비동기 처리 코드는 가독성이 낮다. 한편 async/await을 사용하면 이 코드는
// x.await + y.await + z.await
// 과 같이 기존의 동기 프로그래밍과 완전히 동일하게 기술할 수 있다.

/// 5.3.2 IO 다중화와 async/await
/// 이 절에서는 epoll을 이용한 비동기 IO와 async/await을 조합하는 방법을 설명한다. 다음 그림은 이 절에서 구현할
/// 컴포넌트의 관계를 나타낸 것이다.
///                                       ┌---IO Selector------┐
///                                       |      [epoll]       |
///                                 wake  |         ↑          |
///  Executor <----- [실행 Queue] <--------┼---[Task 정보, ...] |
///      |                                |         ↑         |
/// poll |                                └---------┼---------┘
///      |           [IO Queue]---------------------┘
///      ↓               ↑
/// Task/Waker[Future[Future, Future, ...], ...]
///
/// 그림 5-3 IO 다중화와 async/await
///
/// Task type, Executor type, Spawner type은 189p의 scheduling에서 했던 구현을 사용한다. 여기서는 이들 type에
/// 더해 IO 다중화를 수행하기 위한 IO Selector type을 구현한다. IO Selector는 Task 정보를 받아 epoll을 이용해
/// 감시를 수행하고 event가 발생하면 wake 함수를 호출해 실행 queue에 Task를 등록한다. 따라서 Future의 코드 안에서
/// 비동기 IO를 수행할 때는 IO Selector로 감시 대상 파일 descriptor 및 Waker를 등록해야 한다.
/// 다음 코드는 기본적으로는 epoll, TCP/IP, async/await을 이용하기 위해 필요한 것들을 조합한 것이다.
#[test]
pub fn func_197() {
    use futures::{
        future::{BoxFuture, FutureExt},
        task::{waker_ref, ArcWake},
    };
    use nix::{
        errno::Errno,
        sys::{
            epoll::{
                epoll_create1, epoll_ctl, epoll_wait,
                EpollCreateFlags, EpollEvent, EpollFlags, EpollOp,
            },
            eventfd::{eventfd, EfdFlags}, // eventfd용 import. eventfd? 리눅스 고유의 이벤트 알림용 인터페이스.
            // eventfd에서는 커널 안에 8bytes의 정수값을 저장하며 그 값이 0보다 큰 경우 읽기 event가 발생함. 값에 대한
            // 읽기 쓰기는 read와 write 시스템 콜로 수행할 수 있다. 이번 구현에서는 IO Selector에 대한 알림에
            // 이 eventfd를 이용해 보자.
        },
        unistd::write,
    };
    use std::{
        collections::{HashMap, VecDeque},
        future::Future,
        io::{BufRead, BufReader, BufWriter, Write},
        net::{SocketAddr, TcpListener, TcpStream},
        os::unix::io::{AsRawFd, RawFd},
        pin::Pin,
        sync::{
            mpsc::{sync_channel, Receiver, SyncSender},
            Arc, Mutex,
        },
        task::{Context, Poll, Waker},
    };

    fn write_eventfd(fd: RawFd, n: usize) {
        // usize를 *const u8로 변환
        let ptr = &n as *const usize as *const u8;
        let val = unsafe {
            std::slice::from_raw_parts(
                ptr, std::mem::size_of_val(&n))
        };
        // write 시스템 콜 호출
        write(fd, &val).unwrap();
    }
    // 이번 구현에서는 이 함수를 이용해 eventfd에 1을 입력함으로써 IO Selector에 알리고,
    // IOSelector는 읽기 후에 0을 입력함으로써 event 알림을 해제한다.

    // Implementating IOSelector type
    // 먼저 IOOps와 IOSelector type을 정의해보자
    enum IOOps {
        ADD(EpollFlags, RawFd, Waker), // epoll에 추가
        REMOVE(RawFd),                 // epoll에서 삭제
    }

    struct IOSelector {
        wakers: Mutex<HashMAp<RawFd, Waker>>, // fd에서 waker
        queue: Mutex<VecDeque<IOOps>>,        // IO Queue: 그림 5-3의 IO Queue의 변수
        epfd: RawFd,  // epoll의 fd
        event: RawFd, // eventfd의 fd
    }
    // IOOps type은 IOSelector에 Task와 파일 descriptor의 등록과 삭제를 수행하는 조작을 정의한 type이다.
    // epoll의 감시대상으로 추가할 때는 ADD에 Flag, file descriptor(RawFd), Waker를 감싸서 IO Queue에 넣고,
    // 삭제할 때는 file descriptor(RawFd)를 REMOVE에 감싸서 Queue에 넣는다.
    // IO 다중화를 수행하기 위해서는 file descriptor에 event가 발생했을 때 이에 대응하는 Waker를 호출해야 하기 때문에
    // file descriptor에서 Waker로의 맵을 저장해야 한다. IOSelector type은 그것을 수행하기 위한 정보를 저장하는
    // type이 된다. Queue 변수가 [그림 5-3]의 IO Queue가 된다. 이 변수는 LinkedList가 아니라 VecDeque type으로
    // 정의했는데 이는 계산량을 줄이기 위해서다. LinkedList type에서는 추가와 삭제를 할 때마다 메모리 확보와 해제를
    // 수행하지만 VecDeque type은 내부적인 데이터 구조는 Vector List로 되어 있기 때문에 메모리 확보와 해제를 수행하는
    // 횟수가 적어진다. 따라서 stack이나 queue로 이용한다면 VecDeque를 사용하는 편이 효율이 좋다. 단, LinkedList
    // type과 같이 임의 위치로의 요소 추가 등은 할 수 없다는 제한이 있다.

    // Implemetating IOSelector type
    impl IOSelector {
        fn new() -> Arc<Self> { // 1
            let s = IOSelector {
                wakers: Mutex::new(HashMap::new()),
                queue: Mutex::new(VecDeque::new()),
                epfd: epoll_create1(EpollCreateFlags::empty()).unwrap(),
                // eventfd 생성
                event: eventfd(0, EfdFlags::empty()).unwrap(),
            };
            let result = Arc::new(s); // result에 Arc로 감싼 s(IOSelector)를 할당하고
            let s = result.clone(); // s에 result의 clone(참조)을 붙임 기존의 s는 없어짐
                                                  // 모두 Arc로 감싸져 있음
            // epoll용 스레드 생성. IOSelector에서는 별도 스레드에서 epoll에 의한 event 관리를 수행하기 위해
            // epoll용 스레드를 생성하고 select 함수를 호출
            std::thread::spawn(move || s.select());

            result
        }

        // epoll로 감시하기 위한 함수. file descriptor의 epoll로의 추가와 Waker에 대한 대응을 수행한다.
        fn add_event(
            &self,
            flag: EpollFlags, // epoll flag
            fd: RawFd, // 감시 대상 file descriptor
            waker: Waker,
            wakers: &mut HashMap<RawFd, Waker>,
        ) {
            // 각 정의의 숏컷
            let epoll_add = EpollOp::EpollCtlAdd;
            let epoll_mod = EpollOp::EpollCtlMod;
            let epoll_one = EpollFlags::EPOLLONESHOT;

            // EPOLLONESHOT을 지정하여 일단 event가 발생하면
            // 그 fd로의 event는 재설정하기 전까지 알림이 발생하지 않게 한다(oneshot. epoll로의 연관성이 삭제되는 것은 아님)
            let mut ev = EpollEvent::new(flag | epoll_one, fd as u64);

            // 감시 대상에 추가
            if let Err(err) = epoll_ctl(self.epfd, epoll_add, fd, &mut ev) {
                match err {
                    nix::Error::Sys(Errno::EEXIST) => {
                        // 이미 추가되어 있는 경우에 재설정. epoll_ctl을 호출해서 지정된 file descriptor를
                        // 감시 대상으로 추가한다. 이미 추가되어 있는 경우에는 EpollCtlMod를 지정해 재설정한다.
                        // 이것은 EPOLLONESHOT으로 비활성화된 event를 설정하기 위해 필요하다. 보다 효율적인 구현을
                        // 하기 위해서는 이미 epoll에 추가했는지 기록해두고 시스템 콜 호출 횟수를 줄여야 하지만
                        // EPOLLONESHOT의 이해를 위해 이렇게 구현했음.
                        epoll_ctl(self.epfd, epoll_mod, fd, &mut ev).unwrap();
                    }
                    _ => {
                        panic!("epoll_ctl: {}", err);
                    }
                }
            }

            assert!(!wakers.contains_key(&fd));
            wakers.insert(fd, waker); // file descriptor와 Waker를 k, v쌍으로 wakers에 넣음
        }

        // epoll의 감시에서 삭제하기 위한 함수. 지정한 파일 디스크럽터를 epoll의 감시 대상에서 삭제한다. 여기서는
        // 단순히 epoll_ctl 함수에 EpollCtlDel을 지정해 감시 대상에서 제외하고 file descriptor와 Waker의 관련성도
        // 삭제할 뿐이다.
        fn rm_event(&self, fd: RawFd, wakers: &mut HashMap<RawFd, Waker>) {
            let epoll_del = EpollOp::EpollCtlDel;
            let mut ev = EpollEvent::new(EpollFlags::empty(), fd as u64);
            epoll_ctl(self.epfd, epoll_del, fd, &mut ev).ok();
            wakers.remove(&fd);
        }

        fn select(&self) { // 전용 스레드로 file descriptor의 감시를 수행하기 위한 함수
            // 각 정의의 숏컷
            let epoll_in = EpollFlags::EPOLLIN;
            let epoll_add = EpollOp::EpollCtlAdd;

            // eventfd를 epoll의 감시 대상에 추가.
            let mut ev = EpollEvent::new(epoll_in, self.event as u64);
            epoll_ctl(Self.epfd, epoll_add, self.event, &mut ev).unwrap();

            let mut events = vec![EpollEvent::empty(); 1024];
            //event 발생 감시
            while let Ok(nfds) = epoll_wait(self.epfd, // 위에서 eventfd를 감시 대상에 추가하고 이벤트 발생 감시
                                            &mut events, -1) {
                let mut t = self.wakers.lock().unwrap();
                for n in 0..nfds {
                    if events[n].data() == self.event as u64 {
                        // eventfd의 경우 file descriptor와 Waker를 등록 및 삭제 요구 처리
                        let mut q = self.queue.lock().unwrap();
                        while let Some(op) = q.pop_front() {
                            match op {
                                // 추가
                                IOOps::ADD(flag, fd, waker) =>
                                    self.add_event(flag, fd, waker, &mut t),
                                IOOps::REMOVE(fd) => self.rm_event(fd, &mut t),
                            }
                        }
                    } else {
                        // 생성한 event가 file descriptor인 경우에는 Waker의 wake_by_ref를 호출해 실행 큐에 추가
                        let data = events[n].data() as i32;
                        let waker = t.remove(&data).unwrap();
                        waker.wake_by_ref();
                    }
                }
            }
        }

        // file descriptor 등록용 함수. file descriptor와 Waker를 IOSelector에 등록한다. 이것은 Future가
        // IO Queue에 요청을 넣기 위해 이용됨.
        fn register(&self, flags: EpollFlags, fd: RawFd, waker: Waker) {
            let mut q = self.queue.lock().unwrap();
            q.push_back(IOOps::ADD(flags, fd, waker));
            write_eventfd(self.event, 1);
        }

        // file descriptor 삭제용 함수. file descriptor와 Waker의 연관성을 삭제함.
        fn unregister(&self, fd: RawFd) {
            let mut q = self.queue.lock().unwrap();
            q.push_back(IOOps::REMOVE(fd));
            write_eventfd(self.event, 1);
        }
    }
    // 이렇게 IOSelctor type은 file descriptor와 Waker를 연관짓는다. IOSelector로의 요청은 queue 변수에
    // 요청을 넣고 eventfd에 알린다. channel이 아닌 eventfd에서 수행하는 이유는 IOSelector는 epoll을 이용한
    // file descriptor 감시도 수행해야 하기 때문이다.
}
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
/// 다음 코드는 epoll(리눅스 IO event 감시 시스템 콜)을 이용한 병렬 서버 구현 예따. 작동상으로는 앞의 코드와 거의
/// 비슷하지만 동시에 작동하면서 송수신을 반복하도록 되어 있다는 점이 다르다. 이 코드는 non-blocking 설정을 수행하지
/// 않으므로 구현이 완성되지 않았지만, 이 부분은 뒤에서 설명할 버전에서 마무리 할 것이다.
///
/// - blocking이란 송수신 준비가 되지 않은 상태에서 송수신 함수를 호출하면 해당 함수 호출을 정지하고 송수신 준비가 되었을
/// 때 재개하는 작동을 말한다. 송수신 준비가 되지 않은 경우에 송수신 함수가 호출되면 OS는 그 함수들을 호출한 OS 프로세스를
/// 대기 상태로 만들고, 다른 OS 프로세스를 실행한다.
/// - non-blocking이면 송수신할 수 없는 경우 즉시 함수에서 반환되므로 송수신 함수를 호출해도 OS 프로세스는 대기 상태가 되지 않는다.
#[test]
fn func_181p() {
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

    // epoll용 객체 생성. epoll에서는 감시할 socket(파일 descriptor)을 epoll용 객체에 등록한 뒤 감시 대상 event가
    // 발생할 때까지 대기하고 이벤트 발생 후 해당 이벤트에 대응하는 처리를 수행한다.
    let epfd = epoll_create1(EpollCreateFlags::empty()).unwrap(); // 1

    // listen용 socket을 감시 대상에 추가 2
    let listen_fd = listener.as_raw_fd();
    let mut ev = EpollEvent::new(epoll_in, listen_fd as u64);
    epoll_ctl(epfd, epoll_Add, listen_fd, &mut ev).unwrap();

    let mut fd2buf = HashMap::new();
    let mut events = vec![EpollEvent::empty(); 1024];

    // epoll로 이벤트 발생 감시
    while let Ok(nfds) = epoll_wait(epfd, &mut events, -1) { // 3
        for n in 0..nfds { // 4
            if events[n].data() == listen_fd as u64 {
                // listen socket에 event 5
                if let Ok((stream, _)) = listener.accept() {
                    // 읽기, 쓰기 객체 생성
                    let fd = stream.as_raw_fd();
                    let stream0 = strea.try_clone().unwrap();
                    let reader = BufReader::new(stream0);
                    let writer = BufWriter::new(stream);

                    // fd와 reader, writer의 관계를 만듬
                    fd2buf.insert(fd, (reader, writer));

                    println!("accept: fd = {}", fd);

                    // fd를 감시 대상에 등록
                    let mut ev = EpollEvent::new(epoll_in, fd as u64);
                    epoll_ctl(epfd, epoll_add, fd, &mut ev).unwrap();
                }
            } else {
                // client에서 데이터 도착 6
                let fd = events[n].data() as RawFd;
                let (reader, writer) = fd2buf.get_mut(&fd).unwrap();

                // 1행 읽기
                let mut buf = String::new();
                let n = reader.read_line(&mut buf).unwrap();

                // connection을 close한 경우 epoll 감시 대상에서 제외한다.
                if n == 0 {
                    let mut ev = EpollEvent::new(epoll_in, fd as u64);
                    epoll_ctl(epfd, epoll_del, fd, &mut ev).unwrap();
                    fd2buf.remove(&fd);
                    println!("closed: fd = {}", fd);
                    continue
                }

                print!("read: fd = {}, buf = {}", fd, buf);

                // 읽은 데이터를 그대로 쓴다.
                writer.write(buf.as_bytes()).unwrap();
                writer.flush().unwrap();
            }
        }
    }
}

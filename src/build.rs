use std::process::Command;

const ASM_FILE: &str = "asm/context.S";
const O_FILE: &str = "asm/context.o";
const LIB_FILE: &str = "asm/libcontext.a";

fn main() {
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
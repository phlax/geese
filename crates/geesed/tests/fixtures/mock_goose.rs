// Mock goose binary for testing. Logs its arguments and GOOSE_PATH_ROOT to stderr,
// then parks the thread forever (simulating a long-running acp server).
fn main() {
    let args: Vec<String> = std::env::args().collect();
    eprintln!(
        "mock-goose args={args:?} GOOSE_PATH_ROOT={:?}",
        std::env::var("GOOSE_PATH_ROOT").ok(),
    );
    std::thread::park();
}

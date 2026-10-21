fn main() {
    for _ in 0..600 {
        std::thread::spawn(|| tikra::convert(r"\tikz \draw (0,0) -- (1,1);").unwrap()).join().unwrap();
    }
}

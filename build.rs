fn main() {
    prost_build::compile_protos(&["client/game.proto"], &["client/"]).unwrap();
}

use dlopen_note::dlopen_note;

dlopen_note! {
    soname: ["libfoo.so"],
    frobnicate: "nope",
}

fn main() {}

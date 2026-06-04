use dlopen_note::dlopen_note;

dlopen_note! {
    soname: ["libfoo.so"],
    description: "line one\nline two",
}

fn main() {}

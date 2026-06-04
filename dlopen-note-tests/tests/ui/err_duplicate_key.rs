use dlopen_note::dlopen_note;

dlopen_note! {
    soname: ["libfoo.so"],
    feature: "a",
    feature: "b",
}

fn main() {}

use dlopen_note::dlopen_note;

dlopen_note! {
    soname: ["libfoo.so"],
    priority: "high",
}

fn main() {}

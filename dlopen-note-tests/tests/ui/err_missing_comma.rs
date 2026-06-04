use dlopen_note::dlopen_note;

dlopen_note! {
    soname: ["libfoo.so"]
    feature: "vulkan"
}

fn main() {}

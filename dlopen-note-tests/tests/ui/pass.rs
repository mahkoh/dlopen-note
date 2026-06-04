use dlopen_note::dlopen_note;

dlopen_note! {
    soname: ["libvulkan.so.1", "libfoo.so"],
    feature: "vulkan",
    description: "needs \"quotes\" and a \\ backslash",
    priority: "recommended",
}

dlopen_note! {
    soname: ["libbar.so"],
}

dlopen_note! {
    soname: ["libbaz.so"]
}

fn main() {}

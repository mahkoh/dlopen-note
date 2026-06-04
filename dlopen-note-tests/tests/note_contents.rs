use {
    dlopen_note::dlopen_note,
    object::{Object, ObjectSection},
};

dlopen_note! {
    soname: ["libvulkan.so.1", "libfoo.so"],
    feature: "vulkan",
    description: "needs \"quotes\" and a \\ backslash",
    priority: "recommended",
}

dlopen_note! {
    soname: ["libbar.so"],
}

const NOTE_TYPE: u32 = 0x407c0c0a;

struct Note {
    name: Vec<u8>,
    ty: u32,
    desc: Vec<u8>,
}

fn parse_notes(data: &[u8]) -> Vec<Note> {
    let align = |n: usize| (n + 3) & !3;
    let read_u32 = |o: usize| u32::from_ne_bytes(data[o..o + 4].try_into().unwrap());

    let mut notes = vec![];
    let mut off = 0;
    while off + 12 <= data.len() {
        let namesz = read_u32(off) as usize;
        let descsz = read_u32(off + 4) as usize;
        let ty = read_u32(off + 8);
        off += 12;
        let name = data[off..off + namesz].to_vec();
        off += align(namesz);
        let desc = data[off..off + descsz].to_vec();
        off += align(descsz);
        notes.push(Note { name, ty, desc });
    }
    assert_eq!(off, data.len(), "trailing bytes: notes not tightly packed");
    notes
}

fn dlopen_notes() -> Vec<Note> {
    let exe = std::env::current_exe().unwrap();
    let bytes = std::fs::read(exe).unwrap();
    let obj = object::File::parse(&*bytes).unwrap();
    let section = obj
        .section_by_name(".note.dlopen")
        .expect(".note.dlopen section is present in the binary");
    parse_notes(section.data().unwrap())
}

#[test]
fn header_matches_spec() {
    let notes = dlopen_notes();
    assert!(!notes.is_empty(), "no notes were emitted");
    for note in &notes {
        assert_eq!(note.name, b"FDO\0");
        assert_eq!(note.ty, NOTE_TYPE);
        assert_eq!(
            note.desc.last().copied(),
            Some(0),
            "desc must be NUL-terminated"
        );
    }
}

#[test]
fn payload_is_the_expected_json() {
    let payloads: Vec<String> = dlopen_notes()
        .into_iter()
        .map(|n| {
            assert_eq!(n.desc.last().copied(), Some(0));
            String::from_utf8(n.desc[..n.desc.len() - 1].to_vec()).unwrap()
        })
        .collect();

    let expected_full = r#"[{"soname":["libvulkan.so.1","libfoo.so"],"feature":"vulkan","description":"needs \"quotes\" and a \\ backslash","priority":"recommended"}]"#;
    let expected_minimal = r#"[{"soname":["libbar.so"]}]"#;

    assert_eq!(payloads.len(), 2);
    assert!(payloads.iter().any(|p| p == expected_full));
    assert!(payloads.iter().any(|p| p == expected_minimal));
}

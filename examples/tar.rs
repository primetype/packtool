use packtool::{Packed, View};

fn main() {
    assert_eq!(Header::SIZE, 512,);

    let bytes = std::fs::read("example.tar").expect("need example tar file");
    let mut sub = bytes.as_slice();
    while sub.len() > Header::SIZE {
        let header_view = View::<'_, Header>::try_from_slice(&sub[..Header::SIZE])
            .expect("should start with a header");
        let header = header_view.unpack();

        let file = std::str::from_utf8(&header.filename.0).expect("valid filename");

        let file_size = header.file_size.to_size();
        let file_padding = 512 - file_size % 512;

        println!("compressed file: {} ({} bytes)", file, file_size);

        sub = &sub[Header::SIZE + file_size + file_padding..];

        if sub.len() == 1024 {
            break;
        }
    }
}

#[derive(Packed)]
pub struct FileName([u8; 100]);

#[derive(Packed)]
pub struct FileMode([u8; 8]);

#[derive(Packed)]
pub struct Owner(u64);

#[derive(Packed)]
pub struct Group(u64);

#[derive(Packed)]
pub struct FileSize([u8; 12]);

#[derive(Packed)]
pub struct LastUpdate([u8; 12]);

#[derive(Packed)]
pub struct Checksum([u8; 8]);

#[derive(Packed)]
#[repr(u8)]
pub enum TypeFlag {
    NormalFile = 0x30,
    HardLink = 0x31,
    SymbolicLink = 0x32,
    CharacterSpecial = 0x33,
    BlockSpecial = 0x34,
    Directory = 0x35,
    Fifo = 0x36,
    ContiguousFile = 0x37,
    GlobalExtended = 0x67,
    ExtendedHeader = 0x78,
    // TODO: support for enum with field value size == enum size
    // #[packed(range = "0x41..=0x5a")]
    // Vendor(u8)
}

#[derive(Packed)]
#[packed(value = b"ustar\x00")]
pub struct UStar;

#[derive(Packed)]
#[packed(value = b"00")]
pub struct Version;

#[derive(Packed)]
pub struct OwnerUserName([u8; 32]);

#[derive(Packed)]
pub struct OwnerGroupName([u8; 32]);

#[derive(Packed)]
pub struct DeviceMajorNumber([u8; 8]);

#[derive(Packed)]
pub struct DeviceMinorNumber([u8; 8]);

#[derive(Packed)]
pub struct FileNamePrefix([u8; 155]);

#[derive(Packed)]
#[packed(value = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00")]
pub struct HeaderPadding;

#[derive(Packed)]
pub struct Header {
    filename: FileName,
    file_mode: FileMode,
    owner: Owner,
    group: Group,
    file_size: FileSize,
    last_update: LastUpdate,
    checksum: Checksum,
    type_flag: TypeFlag,
    linked_file: FileName,
    ustar: UStar,
    version: Version,
    user_name: OwnerUserName,
    group_name: OwnerGroupName,
    device_major_number: DeviceMajorNumber,
    device_minor_number: DeviceMinorNumber,
    filename_prefix: FileNamePrefix,
    _padding: HeaderPadding,
}

impl FileSize {
    fn to_size(&self) -> usize {
        let mut size = 0;
        for o in self.0.iter().copied() {
            match o {
                b'0'..=b'8' => {
                    size = (size * 8) + (o - b'0') as usize;
                }
                0x00 | 0x20 => {}
                _ => {
                    panic!("unknown octal value: {:?}", o)
                }
            }
        }
        size
    }
}

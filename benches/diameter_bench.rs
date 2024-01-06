#![feature(test)]

extern crate test;
use std::io::Cursor;

use diameter::diameter::DiameterHeader;
use diameter::diameter::DiameterMessage;
use test::black_box;
use test::Bencher;

#[bench]
fn bench_decode_header(b: &mut Bencher) {
    let data = test_data();
    // b.iter(|| black_box(DiameterHeader::decode_from(&data).unwrap()))
    b.iter(|| {
        let mut cursor = Cursor::new(&data);
        black_box(DiameterHeader::decode_from(&mut cursor).unwrap())
    });
}

#[bench]
fn bench_decode_message(b: &mut Bencher) {
    let data = test_data_2();
    b.iter(|| {
        let mut cursor = Cursor::new(&data);
        black_box(DiameterMessage::decode_from(&mut cursor).unwrap())
    });
}

fn test_data() -> &'static [u8] {
    return &[
        0x01, 0x00, 0x00, 0x14, // version, length
        0x80, 0x00, 0x01, 0x10, // flags, code
        0x00, 0x00, 0x00, 0x04, // application_id
        0x00, 0x00, 0x00, 0x03, // hop_by_hop_id
        0x00, 0x00, 0x00, 0x04, // end_to_end_id
    ];
}

fn test_data_2() -> &'static [u8] {
    return &[
        0x01, 0x00, 0x00, 0x34, // version, length
        0x80, 0x00, 0x01, 0x10, // flags, code
        0x00, 0x00, 0x00, 0x04, // application_id
        0x00, 0x00, 0x00, 0x03, // hop_by_hop_id
        0x00, 0x00, 0x00, 0x04, // end_to_end_id
        0x00, 0x00, 0x02, 0x3B, // avp code
        0x40, 0x00, 0x00, 0x0C, // flags, length
        0x00, 0x00, 0x04, 0xB0, // value
        0x00, 0x00, 0x00, 0x1E, // avp code
        0x00, 0x00, 0x00, 0x12, // flags, length
        0x66, 0x6F, 0x6F, 0x62, // value
        0x61, 0x72, 0x31, 0x32, // value
        0x33, 0x34, 0x00, 0x00,
    ];
}

fn main() {}

#![feature(test)]

extern crate test;
use std::io::Cursor;

use diameter::diameter::DiameterHeader;
use test::black_box;
use test::Bencher;

#[bench]
fn bench_decode(b: &mut Bencher) {
    let data = test_data();
    // b.iter(|| black_box(DiameterHeader::decode_from(&data).unwrap()))
    b.iter(|| {
        let cursor = Cursor::new(&data);
        black_box(DiameterHeader::decode_from(cursor).unwrap())
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

fn main() {}

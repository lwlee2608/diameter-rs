#![feature(test)]

extern crate test;
use diameter::avp;
use diameter::avp::enumerated::EnumeratedAvp;
use diameter::avp::group::GroupAvp;
use diameter::avp::identity::IdentityAvp;
use diameter::avp::unsigned32::Unsigned32Avp;
use diameter::avp::utf8string::UTF8StringAvp;
use diameter::avp::Avp;
use diameter::diameter::ApplicationId;
use diameter::diameter::CommandCode;
use diameter::diameter::DiameterHeader;
use diameter::diameter::DiameterMessage;
use diameter::diameter::PROXYABLE_FLAG;
use diameter::diameter::REQUEST_FLAG;
use std::io::Cursor;
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
fn bench_encode_header(b: &mut Bencher) {
    let data = test_data();
    let mut cursor = Cursor::new(&data);
    let header = DiameterHeader::decode_from(&mut cursor).unwrap();

    let mut encoded = Vec::new();
    b.iter(|| {
        encoded.clear();
        black_box(header.encode_to(&mut encoded).unwrap());
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

#[bench]
fn bench_encode_message(b: &mut Bencher) {
    let data = test_data_2();
    let mut cursor = Cursor::new(&data);
    let message = DiameterMessage::decode_from(&mut cursor).unwrap();

    let mut encoded = Vec::new();
    b.iter(|| {
        encoded.clear();
        black_box(message.encode_to(&mut encoded).unwrap());
    });
}

#[bench]
fn bench_decode_cca(b: &mut Bencher) {
    let message = cca_message();
    let mut data = Vec::new();
    message.encode_to(&mut data).unwrap();

    b.iter(|| {
        let mut cursor = Cursor::new(&data);
        black_box(DiameterMessage::decode_from(&mut cursor).unwrap())
    });
}

#[bench]
fn bench_encode_cca(b: &mut Bencher) {
    let message = cca_message();
    let mut encoded = Vec::new();
    b.iter(|| {
        encoded.clear();
        black_box(message.encode_to(&mut encoded).unwrap());
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

fn cca_message() -> DiameterMessage {
    let mut message = DiameterMessage::new(
        CommandCode::CreditControl,
        ApplicationId::CreditControl,
        REQUEST_FLAG | PROXYABLE_FLAG,
        1123158610,
        3102381851,
    );

    message.add_avp(avp!(264, None, IdentityAvp::new("host.example.com"), true));
    message.add_avp(avp!(296, None, IdentityAvp::new("realm.example.com"), true));
    message.add_avp(avp!(263, None, UTF8StringAvp::new("ses;12345888"), true));
    message.add_avp(avp!(268, None, Unsigned32Avp::new(2001), true));
    message.add_avp(avp!(416, None, EnumeratedAvp::new(1), true));
    message.add_avp(avp!(415, None, Unsigned32Avp::new(1000), true));
    message.add_avp(avp!(
        873,
        Some(10415),
        GroupAvp::new(vec![avp!(
            874,
            Some(10415),
            GroupAvp::new(vec![avp!(30, None, UTF8StringAvp::new("10999"), true)]),
            true,
        )]),
        true,
    ));
    message
}

fn main() {}

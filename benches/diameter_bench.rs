#![feature(test)]

extern crate test;
use diameter::avp::flags::M;
use diameter::avp::Enumerated;
use diameter::avp::Grouped;
use diameter::avp::Identity;
use diameter::avp::UTF8String;
use diameter::avp::Unsigned32;
use diameter::dictionary;
use diameter::dictionary::Dictionary;
use diameter::flags;
use diameter::ApplicationId;
use diameter::CommandCode;
use diameter::DiameterHeader;
use diameter::DiameterMessage;
use std::io::Cursor;
use std::sync::Arc;
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
    let dict = Dictionary::new(&[&dictionary::DEFAULT_DICT_XML]);
    let dict = Arc::new(dict);

    let data = test_data_2();
    b.iter(|| {
        let mut cursor = Cursor::new(&data);
        black_box(DiameterMessage::decode_from(&mut cursor, Arc::clone(&dict)).unwrap())
    });
}

#[bench]
fn bench_encode_message(b: &mut Bencher) {
    let dict = Dictionary::new(&[&dictionary::DEFAULT_DICT_XML]);
    let dict = Arc::new(dict);

    let data = test_data_2();
    let mut cursor = Cursor::new(&data);
    let message = DiameterMessage::decode_from(&mut cursor, dict).unwrap();

    let mut encoded = Vec::new();
    b.iter(|| {
        encoded.clear();
        black_box(message.encode_to(&mut encoded).unwrap());
    });
}

#[bench]
fn bench_decode_cca(b: &mut Bencher) {
    let dict = Dictionary::new(&[&dictionary::DEFAULT_DICT_XML]);
    let dict = Arc::new(dict);

    let message = cca_message(Arc::clone(&dict));
    let mut data = Vec::new();
    message.encode_to(&mut data).unwrap();

    b.iter(|| {
        let mut cursor = Cursor::new(&data);
        black_box(DiameterMessage::decode_from(&mut cursor, Arc::clone(&dict)).unwrap())
    });
}

#[bench]
fn bench_encode_cca(b: &mut Bencher) {
    let dict = Dictionary::new(&[&dictionary::DEFAULT_DICT_XML]);
    let dict = Arc::new(dict);

    let message = cca_message(dict);
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
        0x00, 0x00, 0x01, 0x9F, // avp code
        0x40, 0x00, 0x00, 0x0C, // flags, length
        0x00, 0x00, 0x04, 0xB0, // value
        0x00, 0x00, 0x00, 0x1E, // avp code
        0x00, 0x00, 0x00, 0x12, // flags, length
        0x66, 0x6F, 0x6F, 0x62, // value
        0x61, 0x72, 0x31, 0x32, // value
        0x33, 0x34, 0x00, 0x00,
    ];
}

fn cca_message(dict: Arc<Dictionary>) -> DiameterMessage {
    let mut message = DiameterMessage::new(
        CommandCode::CreditControl,
        ApplicationId::CreditControl,
        flags::REQUEST | flags::PROXYABLE,
        1123158610,
        3102381851,
        Arc::clone(&dict),
    );

    message.add_avp(264, None, M, Identity::new("host.example.com").into());
    message.add_avp(296, None, M, Identity::new("realm.example.com").into());
    message.add_avp(263, None, M, UTF8String::new("ses;12345888").into());
    message.add_avp(268, None, M, Unsigned32::new(2001).into());
    message.add_avp(416, None, M, Enumerated::new(1).into());
    message.add_avp(415, None, M, Unsigned32::new(1000).into());

    let mut ps_information = Grouped::new(vec![], Arc::clone(&dict));
    ps_information.add_avp(30, None, M, UTF8String::new("10999").into());
    let mut service_information = Grouped::new(vec![], Arc::clone(&dict));
    service_information.add_avp(874, Some(10415), M, ps_information.into());

    message.add_avp(873, Some(10415), M, service_information.into());
    message
}

fn main() {}

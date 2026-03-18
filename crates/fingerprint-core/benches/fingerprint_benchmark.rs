/// Performance Benchmarks for Fingerprinting Framework
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use fingerprint_core::{grease, ja3_database::JA3Database, packet_capture::*, pcap_generator::*};

fn first_packet_from_pcap(path: &std::path::Path) -> Vec<u8> {
    let mut first_packet = None;
    PacketParser::stream_pcap_file(path, |packet| {
        first_packet = Some(packet.data.to_vec());
        Ok(())
    })
    .expect("Failed to stream PCAP");
    first_packet.expect("PCAP contained no packets")
}

// ========== Packet Parsing Benchmarks ==========

fn bench_packet_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("packet_parsing");

    // Generate sample packet
    let mut gen = PcapGenerator::new();
    gen.add_chrome_syn();
    let pcap_path = std::env::temp_dir().join("bench_packet.pcap");
    gen.write_to_file(&pcap_path)
        .expect("Failed to write test PCAP");

    let packet_data = first_packet_from_pcap(&pcap_path);

    group.throughput(Throughput::Bytes(packet_data.len() as u64));

    // Benchmark Ethernet parsing
    group.bench_function("parse_ethernet", |b| {
        b.iter(|| PacketParser::parse_ethernet(black_box(packet_data.as_slice())));
    });

    // Complete parsing chain benchmark
    group.bench_function("parse_complete_packet", |b| {
        b.iter(|| {
            if let Some((_, rest)) = PacketParser::parse_ethernet(black_box(packet_data.as_slice()))
            {
                if let Some((_, rest)) = PacketParser::parse_ipv4(rest) {
                    PacketParser::parse_tcp(rest)
                } else {
                    None
                }
            } else {
                None
            }
        });
    });

    group.finish();
}

// ========== PCAP Generation Benchmarks ==========

fn bench_pcap_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("pcap_generation");

    group.bench_function("generate_chrome_syn", |b| {
        b.iter(|| {
            let mut gen = PcapGenerator::new();
            gen.add_chrome_syn();
            black_box(gen);
        });
    });

    group.bench_function("generate_firefox_syn", |b| {
        b.iter(|| {
            let mut gen = PcapGenerator::new();
            gen.add_firefox_syn();
            black_box(gen);
        });
    });

    group.bench_function("write_pcap_file", |b| {
        let mut gen = PcapGenerator::new();
        gen.add_chrome_syn();

        b.iter(|| {
            gen.write_to_file(std::env::temp_dir().join("bench_write.pcap"))
                .expect("Failed to write PCAP");
        });
    });

    group.finish();
}

// ========== Complete Fingerprinting Benchmarks ==========

fn bench_complete_fingerprinting(c: &mut Criterion) {
    let mut group = c.benchmark_group("complete_fingerprinting");

    // Generate sample PCAP
    let mut gen = PcapGenerator::new();
    gen.add_chrome_syn();
    let pcap_path = std::env::temp_dir().join("bench_complete.pcap");
    gen.write_to_file(&pcap_path).expect("Failed to write PCAP");

    let packet_data = first_packet_from_pcap(&pcap_path);

    group.bench_function("complete_pipeline", |b| {
        b.iter(|| {
            // Parse layers
            if let Some((_, rest)) = PacketParser::parse_ethernet(packet_data.as_slice()) {
                if let Some((ipv4, rest)) = PacketParser::parse_ipv4(rest) {
                    let _ttl = ipv4.ttl;
                    if let Some((tcp, _)) = PacketParser::parse_tcp(rest) {
                        let _window = tcp.window_size;
                    }
                }
            }

            black_box(());
        });
    });

    group.finish();
}

// ========== Scalability Benchmarks ==========

fn bench_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability");

    for packet_count in [10, 100, 1000].iter() {
        let mut gen = PcapGenerator::new();
        for _ in 0..*packet_count {
            gen.add_chrome_syn();
        }

        let pcap_path = std::env::temp_dir().join(format!("bench_scale_{}.pcap", packet_count));
        gen.write_to_file(&pcap_path).expect("Failed to write PCAP");

        group.throughput(Throughput::Elements(*packet_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(packet_count),
            packet_count,
            |b, _| {
                b.iter(|| {
                    let mut parsed_count = 0;

                    PacketParser::stream_pcap_file(&pcap_path, |packet| {
                        let packet_data = packet.data;
                        if let Some((_, rest)) = PacketParser::parse_ethernet(packet_data) {
                            if let Some((_, rest)) = PacketParser::parse_ipv4(rest) {
                                if PacketParser::parse_tcp(rest).is_some() {
                                    parsed_count += 1;
                                }
                            }
                        }
                        Ok(())
                    })
                    .expect("Failed to stream PCAP");

                    black_box(parsed_count);
                });
            },
        );
    }

    group.finish();
}

// ========== GREASE Normalization Benchmarks ==========

fn bench_grease_normalization(c: &mut Criterion) {
    let mut group = c.benchmark_group("grease_normalization");

    // Sample JA3 strings with and without GREASE
    let ja3_with_grease = black_box(
        "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,\
         0-23-65281-10-11-35-16-5-13-18-51-45-43-27-21-1a1a,29-23-24,0",
    );

    let ja3_without_grease = black_box(
        "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,\
         0-23-65281-10-11-35-16-5-13-18-51-45-43-27-21,29-23-24,0",
    );

    // Benchmark GREASE value detection
    group.bench_function("is_grease_value", |b| {
        let grease_val = black_box(0x1a1a_u16);
        b.iter(|| grease::is_grease_value(grease_val));
    });

    // Benchmark JA3 normalization
    group.bench_function("normalize_ja3_string", |b| {
        b.iter(|| grease::normalize_ja3_string(ja3_with_grease));
    });

    // Benchmark GREASE-aware equality check
    group.bench_function("ja3_equal_ignore_grease", |b| {
        b.iter(|| grease::ja3_equal_ignore_grease(ja3_with_grease, ja3_without_grease));
    });

    // Benchmark JA3 similarity calculation
    group.bench_function("ja3_similarity", |b| {
        b.iter(|| grease::ja3_similarity(ja3_with_grease, ja3_without_grease));
    });

    // Benchmark multiple normalization calls
    let ja3_variants: Vec<&str> = vec![
        ja3_with_grease,
        "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,\
         0-23-65281-10-11-35-16-5-13-18-51-45-43-27-21-0a0a,29-23-24,0",
        "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,\
         0-23-65281-10-11-35-16-5-13-18-51-45-43-27-21-2a2a,29-23-24,0",
    ];

    group.bench_function("normalize_batch_10", |b| {
        b.iter(|| {
            for _ in 0..10 {
                for ja3 in &ja3_variants {
                    black_box(grease::normalize_ja3_string(ja3));
                }
            }
        });
    });

    group.finish();
}

// ========== JA3 Database Benchmarks ==========

fn bench_ja3_database(c: &mut Criterion) {
    let mut group = c.benchmark_group("ja3_database");

    let db = JA3Database::new();

    // Benchmark exact match (hash lookup)
    group.bench_function("exact_match_chrome", |b| {
        b.iter(|| db.match_ja3(black_box("b19a89106f50d406d38e8bd92241af60")));
    });

    group.bench_function("exact_match_firefox", |b| {
        b.iter(|| db.match_ja3(black_box("d76a5a80b4bb0c75ac45782b0b53da91")));
    });

    // Benchmark fuzzy match (JA3 string)
    let chrome_ja3_string = black_box(
        "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,\
         0-23-65281-10-11-35-16-5-13-18-51-45-43-27-21,29-23-24,0",
    );

    group.bench_function("fuzzy_match_chrome_ja3", |b| {
        b.iter(|| db.match_ja3(chrome_ja3_string));
    });

    // Benchmark fuzzy match with GREASE variation
    let chrome_with_grease = black_box(
        "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,\
         0-23-65281-10-11-35-16-5-13-18-51-45-43-27-21-1a1a,29-23-24,0",
    );

    group.bench_function("fuzzy_match_chrome_with_grease", |b| {
        b.iter(|| db.match_ja3(chrome_with_grease));
    });

    // Benchmark batch matching (100 matches)
    group.throughput(Throughput::Elements(100));
    group.bench_function("batch_match_100", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(db.match_ja3(chrome_ja3_string));
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_packet_parsing,
    bench_pcap_generation,
    bench_complete_fingerprinting,
    bench_scalability,
    bench_grease_normalization,
    bench_ja3_database,
);
criterion_main!(benches);

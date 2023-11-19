use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mpcnet::sss::{split_secret, combine_shares, refresh_shares, generate_refresh_key};

fn bench_split_secret(c: &mut Criterion) {
    c.bench_function("split_secret", |b| {
        let secret = b"this is a very secret message";
        let threshold = 5;
        let shares = 10;
        b.iter(|| split_secret(black_box(secret), black_box(threshold), black_box(shares)))
    });
}

fn bench_combine_shares(c: &mut Criterion) {
    c.bench_function("combine_shares", |b| {
        let secret = b"this is a very secret message";
        let threshold = 5;
        let shares = 10;
        let shares_map = split_secret(secret, threshold, shares).unwrap();
        b.iter(|| combine_shares(black_box(&shares_map)))
    });
}

fn bench_refresh_shares(c: &mut Criterion) {
    c.bench_function("refresh_shares", |b| {
        let secret = b"benchmark secret";
        let threshold = 5;
        let shares = 10;
        let mut shares_map = split_secret(secret, threshold, shares).unwrap();

        b.iter(|| {
            let _ = refresh_shares(black_box(&mut shares_map), black_box(threshold));
        })
    });
}

fn bench_generate_refresh_key(c: &mut Criterion) {
    c.bench_function("generate_refresh_key", |b| {
        let threshold = 5;
        let secret_length = 10;

        b.iter(|| {
            let _ = generate_refresh_key(black_box(threshold), black_box(secret_length));
        })
    });
}

criterion_group!(benches, bench_split_secret, bench_combine_shares, bench_refresh_shares, bench_generate_refresh_key);
criterion_main!(benches);

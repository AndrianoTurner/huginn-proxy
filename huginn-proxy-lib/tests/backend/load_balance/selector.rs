use std::str::FromStr;

use huginn_proxy_lib::config::dynamic::backend::BackendUrl;
use huginn_proxy_lib::{BackendSelector, HealthRegistry};

fn backend_url(addr: &str) -> BackendUrl {
    BackendUrl::from_str(addr).expect("hardcoded backend URL must be valid")
}

#[test]
fn select_round_robin_across_healthy_candidates() {
    let selector = BackendSelector::new();
    let registry = HealthRegistry::new();

    let a = backend_url("http://backend-a:9000");
    let b = backend_url("http://backend-b:9000");
    let a_url = a.clone().as_url();
    let b_url = b.clone().as_url();

    let candidates = [a_url, b_url];

    let first = selector
        .select("/api", &candidates, &registry)
        .unwrap_or_else(|| panic!("expected first candidate"));

    let second = selector
        .select("/api", &candidates, &registry)
        .unwrap_or_else(|| panic!("expected second candidate"));

    let third = selector
        .select("/api", &candidates, &registry)
        .unwrap_or_else(|| panic!("expected wraparound candidate"));

    assert_eq!(&first, a.as_url());
    assert_eq!(&second, b.as_url());
    assert_eq!(&third, a.as_url());
}

#[test]
fn select_skips_unhealthy_candidates() {
    let selector = BackendSelector::new();
    let registry = HealthRegistry::new();

    let a = backend_url("http://backend-a:9000");
    let b = backend_url("http://backend-b:9000");
    let candidates = [a.clone(), b.clone()].map(|v| v.as_url().clone());

    let a_health = registry.get_or_create(a.as_url());
    let _b_health = registry.get_or_create(b.as_url());

    a_health.set(false);

    let selected = selector
        .select("/api", &candidates, &registry)
        .unwrap_or_else(|| panic!("expected healthy fallback candidate"));

    assert_eq!(&selected, b.as_url());
}

#[test]
fn select_returns_none_when_all_candidates_unhealthy() {
    let selector = BackendSelector::new();
    let registry = HealthRegistry::new();

    let a = backend_url("http://backend-a:9000");
    let b = backend_url("http://backend-b:9000");
    let candidates = [a.clone(), b.clone()];

    let a_health = registry.get_or_create(a.as_url());
    let b_health = registry.get_or_create(b.as_url());

    a_health.set(false);
    b_health.set(false);

    assert!(selector.select("/api", &candidates, &registry).is_none());
}

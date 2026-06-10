use std::{str::FromStr, sync::Arc};

use huginn_proxy_lib::{config::dynamic::backend::BackendUrl, HealthRegistry};

fn make_backend_url(addr: &str) -> BackendUrl {
    BackendUrl::from_str(addr).expect("hardcoded backend URL must be valid")
}

#[test]
fn empty_registry_treats_unknown_as_healthy() {
    let r = HealthRegistry::new();
    let backend = make_backend_url("http://anywhere:1234");

    assert!(r.is_healthy(backend.as_url()));
    assert!(r.is_empty());
    assert_eq!(r.len(), 0);
}

#[test]
fn get_or_create_inserts_once() {
    let r = HealthRegistry::new();
    let backend = make_backend_url("http://backend:9000");

    let h1 = r.get_or_create(backend.as_url());
    let h2 = r.get_or_create(backend.as_url());

    assert!(Arc::ptr_eq(&h1, &h2));
    assert_eq!(r.len(), 1);
}

#[test]
fn newly_created_backend_is_healthy() {
    let r = HealthRegistry::new();
    let backend = make_backend_url("http://backend:9000");

    r.get_or_create(backend.as_url());

    assert!(r.is_healthy(backend.as_url()));
}

#[test]
fn unhealthy_backend_is_reported_unhealthy() {
    let r = HealthRegistry::new();
    let backend = make_backend_url("http://backend:9000");

    let h = r.get_or_create(backend.as_url());
    h.set(false);

    assert!(!r.is_healthy(backend.as_url()));
}

#[test]
fn unknown_address_is_healthy_even_when_others_are_not() {
    let r = HealthRegistry::new();
    let known = make_backend_url("http://known:9000");
    let other = make_backend_url("http://other:9000");

    let h = r.get_or_create(known.as_url());
    h.set(false);

    assert!(!r.is_healthy(known.as_url()));
    assert!(r.is_healthy(other.as_url()));
}

#[test]
fn remove_drops_entry() {
    let r = HealthRegistry::new();
    let backend = make_backend_url("http://backend:9000");

    let h = r.get_or_create(backend.as_url());
    h.set(false);

    assert!(!r.is_healthy(backend.as_url()));

    r.remove(backend.as_url());

    assert!(r.is_healthy(backend.as_url()));
    assert_eq!(r.len(), 0);
}

#[test]
fn remove_idempotent() {
    let r = HealthRegistry::new();
    let backend = make_backend_url("http://never-registered:1");

    r.remove(backend.as_url());

    assert_eq!(r.len(), 0);
}

#[test]
fn addresses_returns_all_keys() {
    let r = HealthRegistry::new();

    let a = make_backend_url("http://a:1");
    let b = make_backend_url("http://b:2");
    let c = make_backend_url("http://c:3");

    r.get_or_create(a.as_url());
    r.get_or_create(b.as_url());
    r.get_or_create(c.as_url());

    let mut addrs = r.addresses();
    addrs.sort();

    assert_eq!(
        addrs.iter().map(|v| v.to_string()).collect::<Vec<String>>(),
        vec!["http://a:1/".to_string(), "http://b:2/".to_string(), "http://c:3/".to_string(),]
    );
}

#[test]
fn clone_shares_state() {
    let r1 = HealthRegistry::new();
    let r2 = r1.clone();

    let backend = make_backend_url("http://shared:1");

    let h = r1.get_or_create(backend.as_url());
    h.set(false);

    assert!(!r2.is_healthy(backend.as_url()));
    assert_eq!(r2.len(), 1);
}

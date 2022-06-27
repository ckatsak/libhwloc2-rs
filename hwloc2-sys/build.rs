fn main() {
    pkg_config::Config::new()
        .atleast_version("2.7.1")
        .probe("hwloc")
        .expect("failed to find libhwloc >= 2.7.1 via pkg-config");
}

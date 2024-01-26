use extism_pdk::*;

#[plugin_fn]
pub fn greet(name: String) -> FnResult<String> {
    Ok(format!("Hello, {}!", name))
}

#[host_fn]
extern "ExtismHost" {
    fn format_rust_source_code(source: String) -> String;
}

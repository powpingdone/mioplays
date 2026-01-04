use std::{collections::HashMap, env, path::Path};

fn main() {
    slint_build::compile_with_config("ui/main.slint", {
        slint_build::CompilerConfiguration::new().with_library_paths(HashMap::from([(
            "material".into(),
            Path::new(&env::var_os("CARGO_MANIFEST_DIR").unwrap())
                .join("ui/material-1.0/material.slint"),
        )]))
    })
    .unwrap();
}

use anyhow::Result;
use std::path::PathBuf;

pub struct Activator {
    env_dir: PathBuf,
}

impl Activator {
    pub fn new(env_dir: PathBuf) -> Self {
        Self { env_dir }
    }

    pub fn generate_activation_script(&self) -> Result<String> {
        let env_path = self.env_dir.canonicalize()?;

        let script = format!(r#"# Lux environment activation script
# Usage: eval "$(lux activate)"

export LUX_ENV="{env_dir}"
export PATH="{env_dir}/bin:$PATH"
export LD_LIBRARY_PATH="{env_dir}/lib:$LD_LIBRARY_PATH"
export DYLD_LIBRARY_PATH="{env_dir}/lib:$DYLD_LIBRARY_PATH"
export PKG_CONFIG_PATH="{env_dir}/lib/pkgconfig:$PKG_CONFIG_PATH"
export CMAKE_PREFIX_PATH="{env_dir}:$CMAKE_PREFIX_PATH"

# Provide deactivation function
lux_deactivate() {{
    if [ -n "$_LUX_OLD_PATH" ]; then
        export PATH="$_LUX_OLD_PATH"
        unset _LUX_OLD_PATH
    fi
    if [ -n "$_LUX_OLD_LD_LIBRARY_PATH" ]; then
        export LD_LIBRARY_PATH="$_LUX_OLD_LD_LIBRARY_PATH"
        unset _LUX_OLD_LD_LIBRARY_PATH
    fi
    if [ -n "$_LUX_OLD_DYLD_LIBRARY_PATH" ]; then
        export DYLD_LIBRARY_PATH="$_LUX_OLD_DYLD_LIBRARY_PATH"
        unset _LUX_OLD_DYLD_LIBRARY_PATH
    fi
    if [ -n "$_LUX_OLD_PKG_CONFIG_PATH" ]; then
        export PKG_CONFIG_PATH="$_LUX_OLD_PKG_CONFIG_PATH"
        unset _LUX_OLD_PKG_CONFIG_PATH
    fi
    if [ -n "$_LUX_OLD_CMAKE_PREFIX_PATH" ]; then
        export CMAKE_PREFIX_PATH="$_LUX_OLD_CMAKE_PREFIX_PATH"
        unset _LUX_OLD_CMAKE_PREFIX_PATH
    fi
    unset LUX_ENV
    unset -f lux_deactivate
}}

# Save old environment
export _LUX_OLD_PATH="$PATH"
export _LUX_OLD_LD_LIBRARY_PATH="$LD_LIBRARY_PATH"
export _LUX_OLD_DYLD_LIBRARY_PATH="$DYLD_LIBRARY_PATH"
export _LUX_OLD_PKG_CONFIG_PATH="$PKG_CONFIG_PATH"
export _LUX_OLD_CMAKE_PREFIX_PATH="$CMAKE_PREFIX_PATH"
"#,
            env_dir = env_path.display()
        );

        Ok(script)
    }

    pub fn print_instructions(&self) {
        println!("\n🚀 To activate this environment, run:");
        println!("   eval \"$(lux activate)\"");
        println!("\n📍 Environment location: {}", self.env_dir.display());
        println!("💡 To deactivate later, run:");
        println!("   lux_deactivate");
    }

    pub fn show_status(&self) -> Result<()> {
        if !self.env_dir.exists() {
            println!("❌ No environment found at {}", self.env_dir.display());
            println!("💡 Run `lux install` to create an environment");
            return Ok(());
        }

        println!("📍 Environment: {}", self.env_dir.display());

        // Check what's installed
        let bin_dir = self.env_dir.join("bin");
        let lib_dir = self.env_dir.join("lib");

        if bin_dir.exists() {
            let bin_count = std::fs::read_dir(&bin_dir)?.count();
            println!("   Binaries: {} files in bin/", bin_count);
        }

        if lib_dir.exists() {
            let lib_count = std::fs::read_dir(&lib_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let path = e.path();
                    path.extension()
                        .and_then(|s| s.to_str())
                        .map(|ext| ext == "so" || ext == "dylib" || ext == "a")
                        .unwrap_or(false)
                })
                .count();
            println!("   Libraries: {} shared libraries in lib/", lib_count);
        }

        println!("\n💡 Activate with: eval \"$(lux activate)\"");
        Ok(())
    }
}

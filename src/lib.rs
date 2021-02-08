use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn source_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("boringssl")
}

pub struct Build {
    out_dir: Option<PathBuf>,
    msvc: bool,
}

pub struct Artifacts {
    root_dir: PathBuf,
    libs: Vec<String>,
}

impl Build {
    pub fn new() -> Build {
        Build {
            out_dir: env::var_os("OUT_DIR").map(|s| PathBuf::from(s).join("boringssl-build")),
            msvc: env::var("TARGET").map_or(false, |t| t.contains("msvc")),
        }
    }

    pub fn out_dir<P: AsRef<Path>>(&mut self, path: P) -> &mut Build {
        self.out_dir = Some(path.as_ref().to_path_buf());
        self
    }

    fn configure_asm_support(&self, cfg: &mut cmake::Config) {
        if !self.msvc {
            return;
        }

        let output = Command::new("cmake")
            .arg("--version")
            .output()
            .expect("Can't find cmake.");
        let output = String::from_utf8_lossy(&output.stdout);
        let uptodate = output
            .split_whitespace()
            .skip(2)
            .next()
            .map_or(false, |version| {
                let vers: Vec<u32> = match version.split(".").map(|n| n.parse()).collect() {
                    Ok(v) => v,
                    Err(_) => return false,
                };
                // Visual Studio build with assembly optimizations is broken for older version of cmake
                vers.len() == 3 && vers[0] >= 3 && vers[1] >= 13
            });
        if uptodate {
            let check_exist = |cmd, arg| {
                Command::new(cmd)
                    .arg(arg)
                    .status()
                    .map_or(false, |s| s.success())
            };
            if check_exist("yasm", "--version") || check_exist("nasm", "-v") {
                return;
            }
        }

        cfg.define("OPENSSL_NO_ASM", "ON");
    }

    pub fn build(&mut self) -> Artifacts {
        let out_dir = self.out_dir.as_ref().expect("OUT_DIR not set");

        let build_dir = out_dir.join("build");
        if build_dir.exists() {
            fs::remove_dir_all(&build_dir).unwrap();
        }
        fs::create_dir_all(&build_dir).unwrap();

        let mut cfg = cmake::Config::new(source_dir());
        self.configure_asm_support(&mut cfg);
        cfg.out_dir(&out_dir);
        cfg.build_target("ssl").build();
        cfg.build_target("crypto").build();

        let header_dir = out_dir.join("include");
        if header_dir.exists() {
            fs::remove_dir_all(&header_dir).unwrap();
        }
        fs::create_dir_all(&header_dir).unwrap();

        let include_dir = source_dir().join("src").join("include");
        cp_r(&include_dir, &header_dir);

        let lib_dir = out_dir.join("lib");
        if lib_dir.exists() {
            fs::remove_dir_all(&lib_dir).unwrap();
        }
        let from_dir = if self.msvc {
            let profile = cfg.get_profile();
            build_dir.join(profile)
        } else {
            build_dir
        };
        fs::rename(&from_dir, &lib_dir).unwrap();

        Artifacts {
            root_dir: self.out_dir.clone().unwrap(),
            libs: vec!["ssl".to_string(), "crypto".to_string()],
        }
    }
}

fn cp_r(src: impl AsRef<Path>, dst: impl AsRef<Path>) {
    for f in fs::read_dir(src.as_ref()).unwrap() {
        let f = f.unwrap();
        let path = f.path();
        let name = path.file_name().unwrap();

        // Skip git metadata as it's been known to cause issues (#26) and
        // otherwise shouldn't be required
        if name.to_str() == Some(".git") {
            continue;
        }

        let dst = dst.as_ref().join(name);
        if f.file_type().unwrap().is_dir() {
            fs::create_dir_all(&dst).unwrap();
            cp_r(&path, &dst);
        } else {
            let _ = fs::remove_file(&dst);
            fs::copy(&path, &dst).unwrap();
        }
    }
}

impl Artifacts {
    pub fn include_dir(&self) -> PathBuf {
        self.root_dir.join("include")
    }

    pub fn lib_dir(&self) -> PathBuf {
        self.root_dir.join("lib")
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    pub fn libs(&self) -> &[String] {
        &self.libs
    }

    pub fn print_cargo_metadata(&self) {
        println!(
            "cargo:rustc-link-search=native={}",
            self.lib_dir().display()
        );
        for lib in self.libs.iter() {
            println!("cargo:rustc-link-lib=static={}", lib);
        }
        println!("cargo:include={}", self.include_dir().display());
        println!("cargo:lib={}", self.lib_dir().display());
    }
}

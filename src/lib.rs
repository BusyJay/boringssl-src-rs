use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub fn source_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("boringssl")
}

pub struct Build {
    out_dir: Option<PathBuf>,
}

pub struct Artifacts {
    root_dir: PathBuf,
    libs: Vec<String>,
}

impl Build {
    pub fn new() -> Build {
        Build {
            out_dir: env::var_os("OUT_DIR").map(|s| PathBuf::from(s).join("boringssl-build")),
        }
    }

    pub fn out_dir<P: AsRef<Path>>(&mut self, path: P) -> &mut Build {
        self.out_dir = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn build(&mut self) -> Artifacts {
        let out_dir = self.out_dir.as_ref().expect("OUT_DIR not set");

        let build_dir = out_dir.join("build");
        if build_dir.exists() {
            fs::remove_dir_all(&build_dir).unwrap();
        }
        fs::create_dir_all(&build_dir).unwrap();

        let mut cfg = cmake::Config::new(source_dir());
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
        fs::rename(&build_dir, &lib_dir).unwrap();

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

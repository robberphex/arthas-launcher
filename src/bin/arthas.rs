use std::{
    env,
    fs::{self, File},
    io::Cursor,
    path::PathBuf,
};
use temp_dir::TempDir;

use jni::{
    objects::{JObject, JValue},
    InitArgsBuilder, JNIVersion, JavaVM,
};
use semver::Version;
extern crate dirs;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    env::remove_var("JAVA_TOOL_OPTIONS");

    println!("arthas-launcher version: v{}", VERSION);

    let arthas_lib_dir = get_arthas_lib_dir();
    let x = String::from(arthas_lib_dir.to_str().unwrap());

    let remote_version = get_remote_version();
    update_if_necessary(
        String::from(arthas_lib_dir.to_str().unwrap()),
        remote_version.clone(),
    );
    println!("remote_version: {}", remote_version);
    let arthas_local_version = get_local_version(arthas_lib_dir);

    let arthas_home = format!("{}/{}/arthas", x, arthas_local_version);
    println!("arthas_home {:?}", arthas_home);

    println!("Calculating attach execution time...");

    let args: Vec<String> = env::args().skip(1).collect();

    let jvm_args = InitArgsBuilder::new()
        .version(JNIVersion::V8)
        .option("-Djava.class.path=/Users/robert/arthas/arthas-boot.jar")
        .build()
        .unwrap();

    let jvm = JavaVM::new(jvm_args).unwrap();

    let env = jvm.attach_current_thread().unwrap();

    let values_array = env
        .new_object_array(
            args.len().try_into().unwrap(),
            "java/lang/String",
            JObject::null(),
        )
        .unwrap();

    for (pos, arg) in args.iter().enumerate() {
        let arg_j = env.new_string(arg).unwrap();
        env.set_object_array_element(values_array, pos.try_into().unwrap(), arg_j)
            .unwrap();
    }

    let values_array = unsafe { JObject::from_raw(values_array) };

    env.call_static_method(
        "com/taobao/arthas/boot/Bootstrap",
        "main",
        "([Ljava/lang/String;)V",
        &[JValue::Object(values_array)],
    )
    .unwrap()
    .v()
    .unwrap();
}

fn get_arthas_lib_dir() -> PathBuf {
    match env::var("ARTHAS_LIB_DIR") {
        Ok(val) => {
            println!("[INFO] ARTHAS_LIB_DIR: {}", val);
            PathBuf::from(val)
        }
        Err(_) => {
            let mut lib_dir = dirs::home_dir().expect("msg");
            lib_dir.push(".arthas");
            lib_dir.push("lib");
            lib_dir
        }
    }
}

fn get_local_version(arthas_lib_dir: PathBuf) -> String {
    let dir_entries = fs::read_dir(&arthas_lib_dir).unwrap();

    let mut versions: Vec<Version> = Vec::new();
    //
    for ele in dir_entries {
        let dir_entry = ele.unwrap();
        let path = dir_entry.path();
        let x = path.strip_prefix(&arthas_lib_dir);
        let y = x.unwrap().to_str();
        let v = Version::parse(y.unwrap()).unwrap();
        versions.push(v);
    }
    versions.sort();
    let version = versions.last().cloned().unwrap();
    version.to_string()
}

fn get_remote_version() -> String {
    let body = reqwest::blocking::get("https://arthas.aliyun.com/api/latest_version")
        .unwrap()
        .text()
        .unwrap();

    String::from(body.trim())
}

fn update_if_necessary(arthas_lib_dir: String, update_version: String) {
    let mut target_dir = PathBuf::from(arthas_lib_dir);
    target_dir = target_dir.join(&update_version);
    target_dir = target_dir.join("arthas");
    if !target_dir.exists() {
        println!("updating version {} ...", update_version);
        let mut download_url = String::from(
            "https://arthas.aliyun.com/download/PLACEHOLDER_VERSION?mirror=PLACEHOLDER_REPO",
        );
        download_url = download_url.replace("PLACEHOLDER_REPO", &get_repo_url());
        download_url = download_url.replace("PLACEHOLDER_VERSION", &update_version);

        println!("Download arthas from: {}", download_url);
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path();
        let zip_path = tmp_dir.child("arthas.zip");
        let mut fd = File::create(&zip_path).unwrap();

        let client = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .unwrap();

        let response = client.get(download_url).send().unwrap();
        let mut content = Cursor::new(response.bytes().unwrap());
        std::io::copy(&mut content, &mut fd).unwrap();
        drop(fd);
        tmp_dir.leak();

        let r_fd = File::open(&zip_path).unwrap();

        fs::create_dir_all(&target_dir).unwrap();
        zip_extract::extract(r_fd, &target_dir, true).unwrap();
        println!("update completed. {:?}", target_dir);
    }
}

fn get_repo_url() -> String {
    String::from("aliyun")
}

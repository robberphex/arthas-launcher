use std::env;

use jni::{
    objects::{JObject, JValue},
    InitArgsBuilder, JNIVersion, JavaVM,
};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    env::remove_var("JAVA_TOOL_OPTIONS");

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
        env.set_object_array_element(values_array, pos.try_into().unwrap(), arg_j).unwrap();
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

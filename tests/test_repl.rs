use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use viper_lang::repl::ReplSession;

#[test]
fn repl_uses_input_path_for_import_resolution() {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_dir = std::env::temp_dir().join(format!("viper_repl_test_{}", timestamp));
    fs::create_dir_all(&temp_dir).unwrap();
    fs::write(temp_dir.join("helper.vp"), "value: i64 = 7\n").unwrap();

    let input_path = temp_dir.join("__repl__.vp");
    let mut session = ReplSession::with_input_path(input_path);
    let result = session.execute_chunk("import helper\n");

    let _ = fs::remove_file(temp_dir.join("helper.vp"));
    let _ = fs::remove_dir(&temp_dir);

    assert!(
        result.is_ok(),
        "REPL should resolve imports relative to its input path: {:?}",
        result
    );
}

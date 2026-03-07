//! JIT stub registration with macros to reduce repetition

/// Register a single JIT stub function
macro_rules! register_stub {
    ($ee:expr, $mod:expr, $func:literal, $stub:expr) => {
        if let Some(func) = $mod.get_function($func) {
            $ee.add_global_mapping(&func.as_global_value(), $stub as *const () as usize);
        }
    };
}

/// Register multiple JIT stub functions at once
macro_rules! register_stubs {
    ($ee:expr, $mod:expr, [$($func:literal => $stub:expr),* $(,)?]) => {
        $(
            register_stub!($ee, $mod, $func, $stub);
        )*
    };
}

mod collections;
mod concurrency;
mod core;
mod io;
mod math;
mod strings;

pub use collections::register_collection_stubs;
pub use concurrency::register_concurrency_stubs;
pub use core::register_core_stubs;
pub use io::register_io_stubs;
pub use math::register_math_stubs;
pub use strings::register_string_stubs;

pub fn register_stubs(execution_engine: &inkwell::execution_engine::ExecutionEngine, module: &inkwell::module::Module) {
    // Register core stubs (memory, GC, tagged int)
    core::register_core_stubs(execution_engine, module);

    // Register I/O stubs
    io::register_io_stubs(execution_engine, module);

    // Register collection stubs (lists, dicts, sets, tuples, arrays, bitvec)
    collections::register_collection_stubs(execution_engine, module);

    // Register string stubs
    strings::register_string_stubs(execution_engine, module);

    // Register math stubs (bigint, decimal, math)
    math::register_math_stubs(execution_engine, module);

    // Register concurrency stubs (channels, tasks, asyncio)
    concurrency::register_concurrency_stubs(execution_engine, module);

    // Register hash functions
    register_stubs!(execution_engine, module, [
        "vp_hash_i64" => super::hash::vp_hash_i64,
        "vp_hash_f64" => super::hash::vp_hash_f64,
        "vp_hash_bool" => super::hash::vp_hash_bool,
        "vp_hash_str" => super::hash::vp_hash_str,
        "vp_hash_none" => super::hash::vp_hash_none,
    ]);

    // Register JSON functions
    register_stubs!(execution_engine, module, [
        "vp_json_loads" => super::json::vp_json_loads,
        "vp_json_dumps" => super::json::vp_json_dumps,
        "vp_json_load_file" => super::json::vp_json_load_file,
        "vp_json_dump_file" => super::json::vp_json_dump_file,
        "vp_json_get_error" => super::json::vp_json_get_error,
    ]);

    // Register regex (re) module functions
    register_stubs!(execution_engine, module, [
        "vp_re_compile" => super::re::vp_re_compile,
        "vp_re_pattern_free" => super::re::vp_re_pattern_free,
        "vp_re_match" => super::re::vp_re_match,
        "vp_re_search" => super::re::vp_re_search,
        "vp_re_findall" => super::re::vp_re_findall,
        "vp_re_split" => super::re::vp_re_split,
        "vp_re_sub" => super::re::vp_re_sub,
        "vp_re_fullmatch" => super::re::vp_re_fullmatch,
        "vp_re_escape" => super::re::vp_re_escape,
        "vp_match_free" => super::re::vp_match_free,
        "vp_match_start" => super::re::vp_match_start,
        "vp_match_end" => super::re::vp_match_end,
        "vp_match_group" => super::re::vp_match_group,
        "vp_match_span" => super::re::vp_match_span,
        "vp_re_ignorecase" => super::re::vp_re_ignorecase,
        "vp_re_multiline" => super::re::vp_re_multiline,
        "vp_re_dotall" => super::re::vp_re_dotall,
        "vp_re_verbose" => super::re::vp_re_verbose,
    ]);

    // Register random module functions
    register_stubs!(execution_engine, module, [
        "vp_random_random" => super::random_mod::vp_random_random,
        "vp_random_randint" => super::random_mod::vp_random_randint,
        "vp_random_seed" => super::random_mod::vp_random_seed,
        "vp_random_seed_secure" => super::random_mod::vp_random_seed_secure,
        "vp_random_choice" => super::random_mod::vp_random_choice,
        "vp_random_shuffle" => super::random_mod::vp_random_shuffle,
        "vp_random_uniform" => super::random_mod::vp_random_uniform,
        "vp_random_gauss" => super::random_mod::vp_random_gauss,
        "vp_random_normal" => super::random_mod::vp_random_normal,
        "vp_random_exp" => super::random_mod::vp_random_exp,
        "vp_random_sample" => super::random_mod::vp_random_sample,
        "vp_random_bool" => super::random_mod::vp_random_bool,
        "vp_random_get_state" => super::random_mod::vp_random_get_state,
        "vp_random_set_state" => super::random_mod::vp_random_set_state,
        "vp_random_is_initialized" => super::random_mod::vp_random_is_initialized,
        "vp_random_getrandbits" => super::random_mod::vp_random_getrandbits,
        "vp_random_randbytes" => super::random_mod::vp_random_randbytes,
    ]);

    // Register socket module functions
    register_stubs!(execution_engine, module, [
        "vp_socket_create" => super::socket_mod::vp_socket_create,
        "vp_socket_connect" => super::socket_mod::vp_socket_connect,
        "vp_socket_send" => super::socket_mod::vp_socket_send,
        "vp_socket_recv" => super::socket_mod::vp_socket_recv,
        "vp_socket_close" => super::socket_mod::vp_socket_close,
        "vp_socket_bind" => super::socket_mod::vp_socket_bind,
        "vp_socket_listen" => super::socket_mod::vp_socket_listen,
        "vp_socket_accept" => super::socket_mod::vp_socket_accept,
        "vp_socket_setblocking" => super::socket_mod::vp_socket_setblocking,
        "vp_socket_getsockopt" => super::socket_mod::vp_socket_getsockopt,
        "vp_socket_setsockopt" => super::socket_mod::vp_socket_setsockopt,
        "vp_socket_fileno" => super::socket_mod::vp_socket_fileno,
        "vp_socket_af_inet" => super::socket_mod::vp_socket_af_inet,
        "vp_socket_af_inet6" => super::socket_mod::vp_socket_af_inet6,
        "vp_socket_sock_stream" => super::socket_mod::vp_socket_sock_stream,
        "vp_socket_sock_dgram" => super::socket_mod::vp_socket_sock_dgram,
        "vp_socket_sol_socket" => super::socket_mod::vp_socket_sol_socket,
        "vp_socket_so_reuseaddr" => super::socket_mod::vp_socket_so_reuseaddr,
        "vp_socket_tcp_nodelay" => super::socket_mod::vp_socket_tcp_nodelay,
        "vp_socket_shut_rd" => super::socket_mod::vp_socket_shut_rd,
        "vp_socket_shut_wr" => super::socket_mod::vp_socket_shut_wr,
        "vp_socket_shut_rdwr" => super::socket_mod::vp_socket_shut_rdwr,
    ]);

    // Register select module functions
    register_stubs!(execution_engine, module, [
        "vp_select_fdset_create" => super::select_mod::vp_select_fdset_create,
        "vp_select_fdset_free" => super::select_mod::vp_select_fdset_free,
        "vp_select_fdset_add" => super::select_mod::vp_select_fdset_add,
        "vp_select_fdset_remove" => super::select_mod::vp_select_fdset_remove,
        "vp_select_fdset_contains" => super::select_mod::vp_select_fdset_contains,
        "vp_select_fdset_clear" => super::select_mod::vp_select_fdset_clear,
        "vp_select_fdset_get_fds" => super::select_mod::vp_select_fdset_get_fds,
        "vp_select_select" => super::select_mod::vp_select_select,
        "vp_select_result_free" => super::select_mod::vp_select_result_free,
        "vp_select_can_read" => super::select_mod::vp_select_can_read,
        "vp_select_can_write" => super::select_mod::vp_select_can_write,
        "vp_select_get_error" => super::select_mod::vp_select_get_error,
        "vp_select_strerror" => super::select_mod::vp_select_strerror,
        "vp_poll_poll" => super::select_mod::vp_poll_poll,
        "vp_poll_result_free" => super::select_mod::vp_poll_result_free,
        "vp_epoll_create" => super::select_mod::vp_epoll_create,
        "vp_epoll_free" => super::select_mod::vp_epoll_free,
        "vp_epoll_ctl" => super::select_mod::vp_epoll_ctl,
        "vp_epoll_wait" => super::select_mod::vp_epoll_wait,
        "vp_epollin" => super::select_mod::vp_epollin,
        "vp_epollout" => super::select_mod::vp_epollout,
        "vp_epollerr" => super::select_mod::vp_epollerr,
        "vp_epollhup" => super::select_mod::vp_epollhup,
        "vp_epollet" => super::select_mod::vp_epollet,
        "vp_epoll_ctl_add" => super::select_mod::vp_epoll_ctl_add,
        "vp_epoll_ctl_mod" => super::select_mod::vp_epoll_ctl_mod,
        "vp_epoll_ctl_del" => super::select_mod::vp_epoll_ctl_del,
    ]);

    // Register HTTP module functions
    register_stubs!(execution_engine, module, [
        "vp_http_get" => super::http_mod::vp_http_get,
        "vp_http_post" => super::http_mod::vp_http_post,
        "vp_http_request" => super::http_mod::vp_http_request,
        "vp_http_response_status" => super::http_mod::vp_http_response_status,
        "vp_http_response_text" => super::http_mod::vp_http_response_text,
        "vp_http_response_json" => super::http_mod::vp_http_response_json,
        "vp_http_response_header" => super::http_mod::vp_http_response_header,
        "vp_http_response_free" => super::http_mod::vp_http_response_free,
        "vp_http_server_create" => super::http_mod::vp_http_server_create,
        "vp_http_server_free" => super::http_mod::vp_http_server_free,
        "vp_http_server_serve" => super::http_mod::vp_http_server_serve,
        "vp_http_server_stop" => super::http_mod::vp_http_server_stop,
        "vp_http_server_is_running" => super::http_mod::vp_http_server_is_running,
        "vp_http_urlencode" => super::http_mod::vp_http_urlencode,
        "vp_http_urldecode" => super::http_mod::vp_http_urldecode,
        "vp_http_ok" => super::http_mod::vp_http_ok,
        "vp_http_created" => super::http_mod::vp_http_created,
        "vp_http_no_content" => super::http_mod::vp_http_no_content,
        "vp_http_moved_permanently" => super::http_mod::vp_http_moved_permanently,
        "vp_http_found" => super::http_mod::vp_http_found,
        "vp_http_not_modified" => super::http_mod::vp_http_not_modified,
        "vp_http_bad_request" => super::http_mod::vp_http_bad_request,
        "vp_http_unauthorized" => super::http_mod::vp_http_unauthorized,
        "vp_http_forbidden" => super::http_mod::vp_http_forbidden,
        "vp_http_not_found" => super::http_mod::vp_http_not_found,
        "vp_http_method_not_allowed" => super::http_mod::vp_http_method_not_allowed,
        "vp_http_conflict" => super::http_mod::vp_http_conflict,
        "vp_http_internal_server_error" => super::http_mod::vp_http_internal_server_error,
        "vp_http_not_implemented" => super::http_mod::vp_http_not_implemented,
        "vp_http_bad_gateway" => super::http_mod::vp_http_bad_gateway,
        "vp_http_service_unavailable" => super::http_mod::vp_http_service_unavailable,
    ]);

    // Register hashlib module functions
    register_stubs!(execution_engine, module, [
        "vp_hash_sha256" => super::hashlib::vp_hash_sha256,
        "vp_hash_md5" => super::hashlib::vp_hash_md5,
        "vp_hash_sha512" => super::hashlib::vp_hash_sha512,
        "vp_hashlib_new" => super::hashlib::vp_hashlib_new,
        "vp_hashlib_free" => super::hashlib::vp_hashlib_free,
        "vp_hashlib_update" => super::hashlib::vp_hashlib_update,
        "vp_hashlib_digest" => super::hashlib::vp_hashlib_digest,
        "vp_hashlib_hexdigest" => super::hashlib::vp_hashlib_hexdigest,
        "vp_hashlib_block_size_md5" => super::hashlib::vp_hashlib_block_size_md5,
        "vp_hashlib_block_size_sha256" => super::hashlib::vp_hashlib_block_size_sha256,
        "vp_hashlib_block_size_sha512" => super::hashlib::vp_hashlib_block_size_sha512,
        "vp_hashlib_digest_size_md5" => super::hashlib::vp_hashlib_digest_size_md5,
        "vp_hashlib_digest_size_sha256" => super::hashlib::vp_hashlib_digest_size_sha256,
        "vp_hashlib_digest_size_sha512" => super::hashlib::vp_hashlib_digest_size_sha512,
    ]);

    // Register logging module functions
    register_stubs!(execution_engine, module, [
        "vp_logging_create_logger" => super::logging::vp_logging_create_logger,
        "vp_logging_logger_free" => super::logging::vp_logging_logger_free,
        "vp_logging_set_level" => super::logging::vp_logging_set_level,
        "vp_logging_get_level" => super::logging::vp_logging_get_level,
        "vp_logging_enabled_for" => super::logging::vp_logging_enabled_for,
        "vp_logging_debug" => super::logging::vp_logging_debug,
        "vp_logging_info" => super::logging::vp_logging_info,
        "vp_logging_warning" => super::logging::vp_logging_warning,
        "vp_logging_error" => super::logging::vp_logging_error,
        "vp_logging_critical" => super::logging::vp_logging_critical,
        "vp_logging_exception" => super::logging::vp_logging_exception,
        "vp_logging_get_logger" => super::logging::vp_logging_get_logger,
        "vp_logging_basic_config" => super::logging::vp_logging_basic_config,
        "vp_logging_cleanup" => super::logging::vp_logging_cleanup,
        "vp_logging_debug_level" => super::logging::vp_logging_debug_level,
        "vp_logging_info_level" => super::logging::vp_logging_info_level,
        "vp_logging_warning_level" => super::logging::vp_logging_warning_level,
        "vp_logging_error_level" => super::logging::vp_logging_error_level,
        "vp_logging_critical_level" => super::logging::vp_logging_critical_level,
        "vp_logging_notset_level" => super::logging::vp_logging_notset_level,
        "vp_logging_create_filter" => super::logging::vp_logging_create_filter,
        "vp_logging_filter_free" => super::logging::vp_logging_filter_free,
        "vp_logging_filter_call" => super::logging::vp_logging_filter_call,
    ]);

    // Register sys module functions
    register_stubs!(execution_engine, module, [
        "vp_sys_exit" => super::sys::vp_sys_exit,
        "vp_sys_getpid" => super::sys::vp_sys_getpid,
        "vp_sys_get_version" => super::sys::vp_sys_get_version,
        "vp_sys_get_platform" => super::sys::vp_sys_get_platform,
        "vp_sys_get_sysname" => super::sys::vp_sys_get_sysname,
        "vp_sys_get_machine" => super::sys::vp_sys_get_machine,
        "vp_sys_getenv" => super::sys::vp_sys_getenv,
        "vp_sys_setenv" => super::sys::vp_sys_setenv,
        "vp_sys_unsetenv" => super::sys::vp_sys_unsetenv,
        "vp_sys_init" => super::sys::vp_sys_init,
        "vp_sys_get_argv" => super::sys::vp_sys_get_argv,
    ]);

    // Register os module functions
    register_stubs!(execution_engine, module, [
        "vp_os_getcwd" => super::os::vp_os_getcwd,
        "vp_os_chdir" => super::os::vp_os_chdir,
        "vp_os_listdir" => super::os::vp_os_listdir,
        "vp_os_path_join" => super::os::vp_os_path_join,
        "vp_os_getenv" => super::os::vp_os_getenv,
        "vp_os_mkdir" => super::os::vp_os_mkdir,
        "vp_os_makedirs" => super::os::vp_os_makedirs,
        "vp_os_remove" => super::os::vp_os_remove,
        "vp_os_path_exists" => super::os::vp_os_path_exists,
        "vp_os_path_isfile" => super::os::vp_os_path_isfile,
        "vp_os_path_isdir" => super::os::vp_os_path_isdir,
        "vp_os_path_getsize" => super::os::vp_os_path_getsize,
        "vp_os_path_abspath" => super::os::vp_os_path_abspath,
        "vp_os_path_basename" => super::os::vp_os_path_basename,
        "vp_os_path_dirname" => super::os::vp_os_path_dirname,
        "vp_os_rename" => super::os::vp_os_rename,
        "vp_os_copy" => super::os::vp_os_copy,
        "vp_os_get_home" => super::os::vp_os_get_home,
        "vp_os_stat" => super::os::vp_os_stat,
    ]);

    // Register time module functions
    register_stubs!(execution_engine, module, [
        "vp_time_time" => super::time_mod::vp_time_time,
        "vp_time_monotonic" => super::time_mod::vp_time_monotonic,
        "vp_time_perf_counter" => super::time_mod::vp_time_perf_counter,
        "vp_time_sleep" => super::time_mod::vp_time_sleep,
        "vp_time_localtime" => super::time_mod::vp_time_localtime,
        "vp_time_gmtime" => super::time_mod::vp_time_gmtime,
        "vp_time_strftime" => super::time_mod::vp_time_strftime,
        "vp_time_timezone_offset" => super::time_mod::vp_time_timezone_offset,
        "vp_time_isdst" => super::time_mod::vp_time_isdst,
        "vp_time_days_in_month" => super::time_mod::vp_time_days_in_month,
        "vp_time_sleep_ms" => super::time_mod::vp_time_sleep_ms,
        "vp_time_sleep_us" => super::time_mod::vp_time_sleep_us,
    ]);

    // Register typing module functions
    register_stubs!(execution_engine, module, [
        "vp_typing_get_type_hints" => super::typing::vp_typing_get_type_hints,
        "vp_typing_get_origin" => super::typing::vp_typing_get_origin,
        "vp_typing_get_args" => super::typing::vp_typing_get_args,
        "vp_typing_is_generic_type" => super::typing::vp_typing_is_generic_type,
        "vp_typing_typevar_new" => super::typing::vp_typing_typevar_new,
    ]);

    // Register exception handling functions
    register_stubs!(execution_engine, module, [
        "viper_panic" => super::sys::viper_panic,
        "viper_raise_exception" => super::exceptions::viper_raise_exception,
        "viper_raise_with_code" => super::exceptions::viper_raise_with_code,
        "viper_raise_with_cause" => super::exceptions::viper_raise_with_cause,
        "viper_catch_exception" => super::exceptions::viper_catch_exception,
        "viper_get_exception_type" => super::exceptions::viper_get_exception_type,
        "viper_get_exception_message" => super::exceptions::viper_get_exception_message,
        "viper_get_exception_code" => super::exceptions::viper_get_exception_code,
        "viper_clear_exception" => super::exceptions::viper_clear_exception,
        "viper_set_exception" => super::exceptions::viper_set_exception,
        "viper_format_exception" => super::exceptions::viper_format_exception,
        "viper_print_traceback" => super::exceptions::viper_print_traceback,
        "viper_exception_matches" => super::exceptions::viper_exception_matches,
        "viper_free_string" => super::exceptions::viper_free_string,
        "viper_has_exception" => super::exceptions::viper_has_exception,
        "viper_reraise_exception" => super::exceptions::viper_reraise_exception,
        "viper_exception_to_string" => super::exceptions::viper_exception_to_string,
    ]);
}

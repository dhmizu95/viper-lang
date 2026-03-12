use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;

pub fn register_stubs(execution_engine: &ExecutionEngine, module: &Module) {
    // Register core stubs (memory, GC, tagged int)
    super::core::register_core_stubs(execution_engine, module);

    // Register I/O stubs
    super::io::register_io_stubs(execution_engine, module);

    // Register collection stubs (lists, dicts, sets, tuples, arrays, bitvec)
    super::collections::register_collection_stubs(execution_engine, module);

    // Register string stubs
    super::strings::register_string_stubs(execution_engine, module);

    // Register math stubs (bigint, decimal, math)
    super::math::register_math_stubs(execution_engine, module);

    // Register memoization stubs (lru_cache, cache)
    super::memoization::register_memoization_stubs(execution_engine, module);

    // Register concurrency stubs (channels, tasks, asyncio)
    super::concurrency::register_concurrency_stubs(execution_engine, module);

    // Register hash functions
    register_stubs!(execution_engine, module, [
        "vp_hash_i64" => super::super::hash::vp_hash_i64,
        "vp_hash_f64" => super::super::hash::vp_hash_f64,
        "vp_hash_bool" => super::super::hash::vp_hash_bool,
        "vp_hash_str" => super::super::hash::vp_hash_str,
        "vp_hash_none" => super::super::hash::vp_hash_none,
    ]);

    // Register JSON functions
    register_stubs!(execution_engine, module, [
        "vp_json_loads" => super::super::json::vp_json_loads,
        "vp_json_dumps" => super::super::json::vp_json_dumps,
        "vp_json_load_file" => super::super::json::vp_json_load_file,
        "vp_json_dump_file" => super::super::json::vp_json_dump_file,
        "vp_json_get_error" => super::super::json::vp_json_get_error,
    ]);

    // Register regex (re) module functions
    register_stubs!(execution_engine, module, [
        "vp_re_compile" => super::super::re::vp_re_compile,
        "vp_re_pattern_free" => super::super::re::vp_re_pattern_free,
        "vp_re_match" => super::super::re::vp_re_match,
        "vp_re_search" => super::super::re::vp_re_search,
        "vp_re_findall" => super::super::re::vp_re_findall,
        "vp_re_split" => super::super::re::vp_re_split,
        "vp_re_sub" => super::super::re::vp_re_sub,
        "vp_re_fullmatch" => super::super::re::vp_re_fullmatch,
        "vp_re_escape" => super::super::re::vp_re_escape,
        "vp_match_free" => super::super::re::vp_match_free,
        "vp_match_start" => super::super::re::vp_match_start,
        "vp_match_end" => super::super::re::vp_match_end,
        "vp_match_group" => super::super::re::vp_match_group,
        "vp_match_span" => super::super::re::vp_match_span,
        "vp_re_ignorecase" => super::super::re::vp_re_ignorecase,
        "vp_re_multiline" => super::super::re::vp_re_multiline,
        "vp_re_dotall" => super::super::re::vp_re_dotall,
        "vp_re_verbose" => super::super::re::vp_re_verbose,
    ]);

    // Register random module functions
    register_stubs!(execution_engine, module, [
        "vp_random_random" => super::super::random_mod::vp_random_random,
        "vp_random_randint" => super::super::random_mod::vp_random_randint,
        "vp_random_seed" => super::super::random_mod::vp_random_seed,
        "vp_random_seed_secure" => super::super::random_mod::vp_random_seed_secure,
        "vp_random_choice" => super::super::random_mod::vp_random_choice,
        "vp_random_shuffle" => super::super::random_mod::vp_random_shuffle,
        "vp_random_uniform" => super::super::random_mod::vp_random_uniform,
        "vp_random_gauss" => super::super::random_mod::vp_random_gauss,
        "vp_random_normal" => super::super::random_mod::vp_random_normal,
        "vp_random_exp" => super::super::random_mod::vp_random_exp,
        "vp_random_sample" => super::super::random_mod::vp_random_sample,
        "vp_random_bool" => super::super::random_mod::vp_random_bool,
        "vp_random_get_state" => super::super::random_mod::vp_random_get_state,
        "vp_random_set_state" => super::super::random_mod::vp_random_set_state,
        "vp_random_is_initialized" => super::super::random_mod::vp_random_is_initialized,
        "vp_random_getrandbits" => super::super::random_mod::vp_random_getrandbits,
        "vp_random_randbytes" => super::super::random_mod::vp_random_randbytes,
    ]);

    // Register socket module functions
    register_stubs!(execution_engine, module, [
        "vp_socket_create" => super::super::socket_mod::vp_socket_create,
        "vp_socket_connect" => super::super::socket_mod::vp_socket_connect,
        "vp_socket_send" => super::super::socket_mod::vp_socket_send,
        "vp_socket_recv" => super::super::socket_mod::vp_socket_recv,
        "vp_socket_close" => super::super::socket_mod::vp_socket_close,
        "vp_socket_bind" => super::super::socket_mod::vp_socket_bind,
        "vp_socket_listen" => super::super::socket_mod::vp_socket_listen,
        "vp_socket_accept" => super::super::socket_mod::vp_socket_accept,
        "vp_socket_setblocking" => super::super::socket_mod::vp_socket_setblocking,
        "vp_socket_getsockopt" => super::super::socket_mod::vp_socket_getsockopt,
        "vp_socket_setsockopt" => super::super::socket_mod::vp_socket_setsockopt,
        "vp_socket_fileno" => super::super::socket_mod::vp_socket_fileno,
        "vp_socket_af_inet" => super::super::socket_mod::vp_socket_af_inet,
        "vp_socket_af_inet6" => super::super::socket_mod::vp_socket_af_inet6,
        "vp_socket_sock_stream" => super::super::socket_mod::vp_socket_sock_stream,
        "vp_socket_sock_dgram" => super::super::socket_mod::vp_socket_sock_dgram,
        "vp_socket_sol_socket" => super::super::socket_mod::vp_socket_sol_socket,
        "vp_socket_so_reuseaddr" => super::super::socket_mod::vp_socket_so_reuseaddr,
        "vp_socket_tcp_nodelay" => super::super::socket_mod::vp_socket_tcp_nodelay,
        "vp_socket_shut_rd" => super::super::socket_mod::vp_socket_shut_rd,
        "vp_socket_shut_wr" => super::super::socket_mod::vp_socket_shut_wr,
        "vp_socket_shut_rdwr" => super::super::socket_mod::vp_socket_shut_rdwr,
    ]);

    // Register select module functions
    register_stubs!(execution_engine, module, [
        "vp_select_fdset_create" => super::super::select_mod::vp_select_fdset_create,
        "vp_select_fdset_free" => super::super::select_mod::vp_select_fdset_free,
        "vp_select_fdset_add" => super::super::select_mod::vp_select_fdset_add,
        "vp_select_fdset_remove" => super::super::select_mod::vp_select_fdset_remove,
        "vp_select_fdset_contains" => super::super::select_mod::vp_select_fdset_contains,
        "vp_select_fdset_clear" => super::super::select_mod::vp_select_fdset_clear,
        "vp_select_fdset_get_fds" => super::super::select_mod::vp_select_fdset_get_fds,
        "vp_select_select" => super::super::select_mod::vp_select_select,
        "vp_select_result_free" => super::super::select_mod::vp_select_result_free,
        "vp_select_can_read" => super::super::select_mod::vp_select_can_read,
        "vp_select_can_write" => super::super::select_mod::vp_select_can_write,
        "vp_select_get_error" => super::super::select_mod::vp_select_get_error,
        "vp_select_strerror" => super::super::select_mod::vp_select_strerror,
        "vp_poll_poll" => super::super::select_mod::vp_poll_poll,
        "vp_poll_result_free" => super::super::select_mod::vp_poll_result_free,
        "vp_epoll_create" => super::super::select_mod::vp_epoll_create,
        "vp_epoll_free" => super::super::select_mod::vp_epoll_free,
        "vp_epoll_ctl" => super::super::select_mod::vp_epoll_ctl,
        "vp_epoll_wait" => super::super::select_mod::vp_epoll_wait,
        "vp_epollin" => super::super::select_mod::vp_epollin,
        "vp_epollout" => super::super::select_mod::vp_epollout,
        "vp_epollerr" => super::super::select_mod::vp_epollerr,
        "vp_epollhup" => super::super::select_mod::vp_epollhup,
        "vp_epollet" => super::super::select_mod::vp_epollet,
        "vp_epoll_ctl_add" => super::super::select_mod::vp_epoll_ctl_add,
        "vp_epoll_ctl_mod" => super::super::select_mod::vp_epoll_ctl_mod,
        "vp_epoll_ctl_del" => super::super::select_mod::vp_epoll_ctl_del,
    ]);

    // Register HTTP module functions
    register_stubs!(execution_engine, module, [
        "vp_http_get" => super::super::http_mod::vp_http_get,
        "vp_http_post" => super::super::http_mod::vp_http_post,
        "vp_http_request" => super::super::http_mod::vp_http_request,
        "vp_http_response_status" => super::super::http_mod::vp_http_response_status,
        "vp_http_response_text" => super::super::http_mod::vp_http_response_text,
        "vp_http_response_json" => super::super::http_mod::vp_http_response_json,
        "vp_http_response_header" => super::super::http_mod::vp_http_response_header,
        "vp_http_response_free" => super::super::http_mod::vp_http_response_free,
        "vp_http_server_create" => super::super::http_mod::vp_http_server_create,
        "vp_http_server_free" => super::super::http_mod::vp_http_server_free,
        "vp_http_server_serve" => super::super::http_mod::vp_http_server_serve,
        "vp_http_server_stop" => super::super::http_mod::vp_http_server_stop,
        "vp_http_server_is_running" => super::super::http_mod::vp_http_server_is_running,
        "vp_http_urlencode" => super::super::http_mod::vp_http_urlencode,
        "vp_http_urldecode" => super::super::http_mod::vp_http_urldecode,
        "vp_http_ok" => super::super::http_mod::vp_http_ok,
        "vp_http_created" => super::super::http_mod::vp_http_created,
        "vp_http_no_content" => super::super::http_mod::vp_http_no_content,
        "vp_http_moved_permanently" => super::super::http_mod::vp_http_moved_permanently,
        "vp_http_found" => super::super::http_mod::vp_http_found,
        "vp_http_not_modified" => super::super::http_mod::vp_http_not_modified,
        "vp_http_bad_request" => super::super::http_mod::vp_http_bad_request,
        "vp_http_unauthorized" => super::super::http_mod::vp_http_unauthorized,
        "vp_http_forbidden" => super::super::http_mod::vp_http_forbidden,
        "vp_http_not_found" => super::super::http_mod::vp_http_not_found,
        "vp_http_method_not_allowed" => super::super::http_mod::vp_http_method_not_allowed,
        "vp_http_conflict" => super::super::http_mod::vp_http_conflict,
        "vp_http_internal_server_error" => super::super::http_mod::vp_http_internal_server_error,
        "vp_http_not_implemented" => super::super::http_mod::vp_http_not_implemented,
        "vp_http_bad_gateway" => super::super::http_mod::vp_http_bad_gateway,
        "vp_http_service_unavailable" => super::super::http_mod::vp_http_service_unavailable,
    ]);

    // Register hashlib module functions
    register_stubs!(execution_engine, module, [
        "vp_hash_sha256" => super::super::hashlib::vp_hash_sha256,
        "vp_hash_md5" => super::super::hashlib::vp_hash_md5,
        "vp_hash_sha512" => super::super::hashlib::vp_hash_sha512,
        "vp_hashlib_new" => super::super::hashlib::vp_hashlib_new,
        "vp_hashlib_free" => super::super::hashlib::vp_hashlib_free,
        "vp_hashlib_update" => super::super::hashlib::vp_hashlib_update,
        "vp_hashlib_digest" => super::super::hashlib::vp_hashlib_digest,
        "vp_hashlib_hexdigest" => super::super::hashlib::vp_hashlib_hexdigest,
        "vp_hashlib_block_size_md5" => super::super::hashlib::vp_hashlib_block_size_md5,
        "vp_hashlib_block_size_sha256" => super::super::hashlib::vp_hashlib_block_size_sha256,
        "vp_hashlib_block_size_sha512" => super::super::hashlib::vp_hashlib_block_size_sha512,
        "vp_hashlib_digest_size_md5" => super::super::hashlib::vp_hashlib_digest_size_md5,
        "vp_hashlib_digest_size_sha256" => super::super::hashlib::vp_hashlib_digest_size_sha256,
        "vp_hashlib_digest_size_sha512" => super::super::hashlib::vp_hashlib_digest_size_sha512,
    ]);

    // Register logging module functions
    register_stubs!(execution_engine, module, [
        "vp_logging_create_logger" => super::super::logging::vp_logging_create_logger,
        "vp_logging_logger_free" => super::super::logging::vp_logging_logger_free,
        "vp_logging_set_level" => super::super::logging::vp_logging_set_level,
        "vp_logging_get_level" => super::super::logging::vp_logging_get_level,
        "vp_logging_enabled_for" => super::super::logging::vp_logging_enabled_for,
        "vp_logging_debug" => super::super::logging::vp_logging_debug,
        "vp_logging_info" => super::super::logging::vp_logging_info,
        "vp_logging_warning" => super::super::logging::vp_logging_warning,
        "vp_logging_error" => super::super::logging::vp_logging_error,
        "vp_logging_critical" => super::super::logging::vp_logging_critical,
        "vp_logging_exception" => super::super::logging::vp_logging_exception,
        "vp_logging_get_logger" => super::super::logging::vp_logging_get_logger,
        "vp_logging_basic_config" => super::super::logging::vp_logging_basic_config,
        "vp_logging_cleanup" => super::super::logging::vp_logging_cleanup,
        "vp_logging_debug_level" => super::super::logging::vp_logging_debug_level,
        "vp_logging_info_level" => super::super::logging::vp_logging_info_level,
        "vp_logging_warning_level" => super::super::logging::vp_logging_warning_level,
        "vp_logging_error_level" => super::super::logging::vp_logging_error_level,
        "vp_logging_critical_level" => super::super::logging::vp_logging_critical_level,
        "vp_logging_notset_level" => super::super::logging::vp_logging_notset_level,
        "vp_logging_create_filter" => super::super::logging::vp_logging_create_filter,
        "vp_logging_filter_free" => super::super::logging::vp_logging_filter_free,
        "vp_logging_filter_call" => super::super::logging::vp_logging_filter_call,
    ]);

    // Register sys module functions
    register_stubs!(execution_engine, module, [
        "vp_sys_exit" => super::super::sys::vp_sys_exit,
        "vp_sys_getpid" => super::super::sys::vp_sys_getpid,
        "vp_sys_get_version" => super::super::sys::vp_sys_get_version,
        "vp_sys_get_platform" => super::super::sys::vp_sys_get_platform,
        "vp_sys_get_sysname" => super::super::sys::vp_sys_get_sysname,
        "vp_sys_get_machine" => super::super::sys::vp_sys_get_machine,
        "vp_sys_getenv" => super::super::sys::vp_sys_getenv,
        "vp_sys_setenv" => super::super::sys::vp_sys_setenv,
        "vp_sys_unsetenv" => super::super::sys::vp_sys_unsetenv,
        "vp_sys_init" => super::super::sys::vp_sys_init,
        "vp_sys_get_argv" => super::super::sys::vp_sys_get_argv,
    ]);

    // Register os module functions
    register_stubs!(execution_engine, module, [
        "vp_os_getcwd" => super::super::os::vp_os_getcwd,
        "vp_os_chdir" => super::super::os::vp_os_chdir,
        "vp_os_listdir" => super::super::os::vp_os_listdir,
        "vp_os_path_join" => super::super::os::vp_os_path_join,
        "vp_os_getenv" => super::super::os::vp_os_getenv,
        "vp_os_mkdir" => super::super::os::vp_os_mkdir,
        "vp_os_makedirs" => super::super::os::vp_os_makedirs,
        "vp_os_remove" => super::super::os::vp_os_remove,
        "vp_os_path_exists" => super::super::os::vp_os_path_exists,
        "vp_os_path_isfile" => super::super::os::vp_os_path_isfile,
        "vp_os_path_isdir" => super::super::os::vp_os_path_isdir,
        "vp_os_path_getsize" => super::super::os::vp_os_path_getsize,
        "vp_os_path_abspath" => super::super::os::vp_os_path_abspath,
        "vp_os_path_basename" => super::super::os::vp_os_path_basename,
        "vp_os_path_dirname" => super::super::os::vp_os_path_dirname,
        "vp_os_rmdir" => super::super::os::vp_os_rmdir,
        "vp_os_rename" => super::super::os::vp_os_rename,
        "vp_os_copy" => super::super::os::vp_os_copy,
        "vp_os_get_home" => super::super::os::vp_os_get_home,
        "vp_os_stat" => super::super::os::vp_os_stat,
    ]);

    // Register time module functions
    register_stubs!(execution_engine, module, [
        "vp_time_time" => super::super::time_mod::vp_time_time,
        "vp_time_monotonic" => super::super::time_mod::vp_time_monotonic,
        "vp_time_perf_counter" => super::super::time_mod::vp_time_perf_counter,
        "vp_time_sleep" => super::super::time_mod::vp_time_sleep,
        "vp_time_localtime" => super::super::time_mod::vp_time_localtime,
        "vp_time_gmtime" => super::super::time_mod::vp_time_gmtime,
        "vp_time_strftime" => super::super::time_mod::vp_time_strftime,
        "vp_time_timezone_offset" => super::super::time_mod::vp_time_timezone_offset,
        "vp_time_isdst" => super::super::time_mod::vp_time_isdst,
        "vp_time_days_in_month" => super::super::time_mod::vp_time_days_in_month,
        "vp_time_sleep_ms" => super::super::time_mod::vp_time_sleep_ms,
        "vp_time_sleep_us" => super::super::time_mod::vp_time_sleep_us,
    ]);

    // Register typing module functions
    register_stubs!(execution_engine, module, [
        "vp_typing_get_type_hints" => super::super::typing::vp_typing_get_type_hints,
        "vp_typing_get_origin" => super::super::typing::vp_typing_get_origin,
        "vp_typing_get_args" => super::super::typing::vp_typing_get_args,
        "vp_typing_is_generic_type" => super::super::typing::vp_typing_is_generic_type,
        "vp_typing_typevar_new" => super::super::typing::vp_typing_typevar_new,
    ]);

    // Register iterator functions
    register_stubs!(execution_engine, module, [
        "vp_iterator_next" => super::super::iterator::vp_iterator_next,
    ]);

    // Register exception handling functions
    register_stubs!(execution_engine, module, [
        "viper_panic" => super::super::sys::viper_panic,
        "viper_raise_exception" => super::super::exceptions::viper_raise_exception,
        "viper_raise_with_code" => super::super::exceptions::viper_raise_with_code,
        "viper_raise_with_cause" => super::super::exceptions::viper_raise_with_cause,
        "viper_catch_exception" => super::super::exceptions::viper_catch_exception,
        "viper_get_exception_type" => super::super::exceptions::viper_get_exception_type,
        "viper_get_exception_message" => super::super::exceptions::viper_get_exception_message,
        "viper_get_exception_code" => super::super::exceptions::viper_get_exception_code,
        "viper_clear_exception" => super::super::exceptions::viper_clear_exception,
        "viper_set_exception" => super::super::exceptions::viper_set_exception,
        "viper_format_exception" => super::super::exceptions::viper_format_exception,
        "viper_print_traceback" => super::super::exceptions::viper_print_traceback,
        "viper_exception_matches" => super::super::exceptions::viper_exception_matches,
        "viper_free_string" => super::super::exceptions::viper_free_string,
        "viper_has_exception" => super::super::exceptions::viper_has_exception,
        "viper_reraise_exception" => super::super::exceptions::viper_reraise_exception,
        "viper_exception_to_string" => super::super::exceptions::viper_exception_to_string,
    ]);
}

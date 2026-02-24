; ModuleID = 'test_chan_create'
source_filename = "test_chan_create"

@str = private unnamed_addr constant [6 x i8] c"Start\00"
@str.1 = private unnamed_addr constant [16 x i8] c"Channel created\00"
@str.2 = private unnamed_addr constant [4 x i8] c"End\00"

declare void @vp_print_i64(i64)

declare void @vp_print_f64(double)

declare void @vp_print_str(ptr)

declare void @vp_print_bool(i1)

declare void @vp_print_newline()

declare ptr @vp_str_concat(ptr, ptr)

declare ptr @vp_list_create()

declare void @vp_list_append(ptr, i64)

declare void @vp_list_free(ptr)

declare i64 @vp_list_get(ptr, i64)

declare i64 @vp_list_len(ptr)

declare void @vp_list_set(ptr, i64, i64)

declare void @vp_list_insert(ptr, i64, i64)

declare i64 @vp_list_remove(ptr, i64)

declare i64 @vp_list_pop(ptr)

declare void @vp_list_clear(ptr)

declare i1 @vp_list_contains(ptr, i64)

declare void @vp_retain(ptr)

declare void @vp_release(ptr, ptr)

declare double @vp_math_sqrt(double)

declare double @vp_math_abs(double)

declare double @vp_math_ln(double)

declare double @vp_math_floor(double)

declare ptr @vp_chan_create(i64)

declare void @vp_chan_destroy(ptr)

declare void @vp_chan_send(ptr, i64)

declare i64 @vp_chan_recv(ptr)

declare ptr @vp_waitgroup_create()

declare void @vp_waitgroup_destroy(ptr)

declare void @vp_waitgroup_add(ptr, i64)

declare void @vp_waitgroup_done(ptr)

declare void @vp_waitgroup_wait(ptr)

declare void @vp_init_threadpool(i64)

declare void @vp_shutdown_threadpool()

define void @main() {
entry:
  call void @vp_print_str(ptr @str)
  call void @vp_print_newline()
  %chan = call ptr @vp_chan_create(i64 10)
  %c = alloca ptr, align 8
  store ptr %chan, ptr %c, align 8
  call void @vp_retain(ptr %chan)
  call void @vp_print_str(ptr @str.1)
  call void @vp_print_newline()
  call void @vp_print_str(ptr @str.2)
  call void @vp_print_newline()
  ret void
}

define void @viper_init() {
entry:
  ret void
}

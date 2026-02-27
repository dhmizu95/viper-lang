import subprocess
res = subprocess.run(['./target/debug/viper', 'build', 'tests/viper_programs/test_bitwise.vp'], capture_output=True, text=True)
print(res.stderr)

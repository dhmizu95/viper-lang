import subprocess
import os

with open('tests/viper_programs/test_list_methods.vp', 'r') as f:
    lines = f.readlines()

out = []
for i, line in enumerate(lines):
    out.append(line)
    if "lst =" in line or line.strip()=="" or "def " in line or "print" in line or "test_list_methods" in line:
        continue
    
    with open('tests/viper_programs/test_list_tmp.vp', 'w') as f:
        f.write("".join(out) + "    print('ok')\n\ntest_list_methods()\n")
        
    res = subprocess.run(['./target/debug/viper', 'build', 'tests/viper_programs/test_list_tmp.vp'], capture_output=True, text=True)
    if res.returncode != 0:
        continue
    res = subprocess.run(['./test_list_tmp_vp_bin'], capture_output=True, text=True)
    if res.returncode != 0:
        print(f"Failed at line: {line.strip()}")
        break

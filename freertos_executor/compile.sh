. ../esp-idf/export.sh
idf.py fullclean && idf.py build
idf.py qemu gdb
cp build/gdbinit/gdbinit build/gdbinit/gdbinit1

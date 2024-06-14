| \#                    | Modules               | Versions           | Locations                                          | Bug Types         |
|-----------------------|-----------------------|--------------------|----------------------------------------------------|-------------------|
| 1                     | fs/buffer             | RT-Linux 6.7       | mark\_buffer\_dirty                                | logic error       |
| 2  | drivers/pci           | RT-Linux 6.7       | vga\_put                                           | logic error       |
| 3                     | kernel/sched          | RT-Linux 6.7       | select\_task\_rq\_fair                             | deadlock          |
| 4  | mm/filemap            | RT-Linux 6.7       | filemap\_fault / page\_add\_file\_rmap             | data race         |
| 5                     | drivers/net/          | RT-Linux 6.7       | e1000\_update\_stats                               | memory corruption |
| 6  | fs/inode              | RT-Linux 6.7       | inode\_update\_timestamps                          | data race         |
| 7                     | kernel/kprobes        | RT-Linux 6.7       | arch\_adjust\_kprobe\_addr                         | logic error       |
| 8  | fs/dcache             | RT-Linux 6.7       | d\_splice\_alias                                   | data race         |
| 9                     | fs/ext4               | RT-Linux 6.7       | \_\_ext4\_new\_inode /\_find\_next\_zero\_bit      | data race         |
| 10 | drivers/e1000         | RT-Linux 6.7       | e1000\_clean                                       | data race         |
| 11                    | lib/find\_bit         | RT-Linux 6.7       | \_find\_first\_bit                                 | data race         |
| 12 | kernel/sched          | RT-Linux 6.8       | \_\_wake\_up\_common                               | null-ptr defer    |
| 13                    | lib/kasprintf         | RT-Linux 6.8       | kvasprintf                                         | logic error       |
| 14 | kernel/events         | RT-Linux 6.8       | perf\_cgroup\_switch                               | logic error       |
| 15                    | fs/inode              | RT-Linux 6.8       | generic\_update\_time / inode\_needs\_update\_time | data race         |
| 16 | fs/ext4               | RT-Linux 6.8       | generic\_write\_end / mpage\_submit\_folio         | data race         |
| 17                    | fs/kernfs             | RT-Linux 6.8       | kernfs\_dop\_revalidate                            | memory corruption |
| 18 | fs/ext4               | RT-Linux 6.8       | ext4\_split\_extent\_at                            | memory corruption |
| 19                    | arch/x86/lib          | RT-Linux 6.8       | memcpy\_orig                                       | out-of-bounds     |
| 20 | kernel/events         | RT-Linux 6.8       | free\_event                                        | logic error       |
| 21                    | drivers/scsi          | RT-Linux 6.8       | \_\_bitmap\_weight /scsi\_device\_unbusy           | data race         |
| 22 | kernel/rcu            | RT-Linux 6.8       | \_\_call\_rcu\_common /mas\_walk                   | data race         |
| 23                    | mm/swap               | RT-Linux 6.8       | \_\_folio\_end\_writeback / lru\_add\_fn           | data race         |
| 24 | arch/x86/lib          | RT-Linux 6.8       | memmove                                            | memory corruption |
| 25                    | arch/x86/events/intel | RT-Linux 6.8       | intel\_pmu\_lbr\_counters\_reorder                 | logic error       |
| 26 | arch/x86/kernel       | RT-Linux 6.8       | deref\_stack\_reg                                  | logic error       |
| 27                    | arch/x86/kernel       | RT-Linux 6.8       | \_\_orc\_find                                      | memory leak       |
| 28 | net/9p                | RT-Linux 6.8       | p9pdu\_readf                                       | memory leak       |
| 29                    | arch/x86/lib          | OpenWrt 5.15       | memset\_erms                                       | logic error       |
| 30 | kernel/smp            | OpenWrt 5.15       | smp\_call\_function\_single                        | logic error       |
| 31                    | arch/arm64/kvm        | RaspberryPi OS 6.7 | kvm\_init\_stage2\_mmu                             | memory leak       |
| 32 | arch/arm64/kvm        | RaspberryPi OS 6.7 | kvm\_age\_gfn                                      | logic error       |
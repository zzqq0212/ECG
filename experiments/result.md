### Here we list the overall experiments results over 6 targets versions:

| Subject      |       openwrt5.15      |       openwrt6.1       |       RT-Linux6.7      |       RT-Linux6.8      |       RaspiOS6.7       |      RT-Linuxv6.8      |         Overall         |
|--------------|:----------------------:|:----------------------:|:----------------------:|:----------------------:|:----------------------:|:----------------------:|:-----------------------:|
| ECG          |        160210.6        |        173660.5        |        206004.3        |        215922.5        |        176111.4        |        168805.2        |        183,452.4        |
| ECG-         |  151153.5(+5.99)% |  166260.5(+4.55)% |  196004.3(+5.10)% |  203922.5(+5.88)% |  165550.9(+6.38)% | 151640.6(+11.32)% |   172422.1(+6.4)%  |
| Syzkaller    | 130876.4(+22.41)% | 144863.5(+19.88)% | 171117.3(+20.39)% | 172977.7(+24.83)% | 139509.9(+26.24)% | 134090.6(+25.89)% | 148,905.8(+23.20)% |
| Moonshine    | 135201.3(+18.50)% | 148661.9(+16.82)% | 174380.2(+18.14)% | 178459.3(+20.99)% | 145825.3(+20.77)% | 138891.7(+21.54)% | 153,569.9(+19.46)% |
| KernelGPT    | 143582.5(+11.58)% |  160346.4(+8.30)% | 185393.4(+11.12)% | 193854.7(+11.38)% | 158850.5(+10.87)% | 149928.1(+12.59)% | 165,325.9(+10.96)% |
| Rtkaller     | 139634.2(+14.74)% | 153689.7(+12.99)% | 179253.7(+14.92)% | 184209.4(+17.22)% | 151840.7(+15.98)% | 144622.3(+16.72)% |  158873.6(+15.47)% |
| ECG-directed |         47228.6        |         49845.2        |         46180.7        |         44865.5        |         47228.6        |         49856.5        |         47534.1         |
| DRLF         |  43283.3(+9.12)%  |                        |  41099.7(+12.36)% |  40445.2(+10.93)% |  43585.6(+8.36)%  |  43754.4(+13.95)% |  42802.4(+11.05)%  |

<div align="left">
  <img src="cov.png" height="500px" alt="coverage_pic" >
</div>

### The below table shows average bug counts of ECG and other fuzzers over 10 rounds of experiments.

| Subject      | openwrt5.15 | openwrt6.1 | RT-Linux6.7 | RT-Linux6.8 | RaspiOS6.7 | RaspiOS6.8 | Total |
|--------------|:-----------:|:----------:|:-----------:|:-----------:|:----------:|:----------:|:-----:|
| ECG          |     7.1     |     7.8    |     8.9     |     8.2     |     6.5    |     7.4    |  45.9 |
| ECG-         |     6.3     |     7.2    |     7.8     |     7.3     |     5.8    |     6.4    |  40.8 |
| KernelGPT    |     5.5     |     6.3    |     6.7     |     6.1     |     5.2    |     5.7    |  35.5 |
| Moonshine    |     4.5     |     4.9    |     4.4     |     5.1     |     4.2    |     4.7    |  27.8 |
| Rtkaller     |     4.7     |     5.3    |     5.2     |     5.5     |     4.6    |     5.1    |  30.4 |
| Syzkaller    |     4.1     |     4.6    |     4.3     |     4.8     |     3.9    |     3.7    |  25.4 |
| ECG-directed |     4.3     |     4.6    |     3.9     |     4.2     |     4.1    |     3.5    |  24.6 |
| DRLF         |     3.5     |     4.1    |     3.2     |     3.7     |     3.6    |     3.1    |  21.2 |


### New previous unknown bugs reported detailed description below, the `crash` folder contained all previously unknown bugs detected by ECG.

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
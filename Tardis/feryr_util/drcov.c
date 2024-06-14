 /*
 * Copyright (C) 2021, Ivanov Arkady <arkadiy.ivanov@ispras.ru>
 *
 * Drcov - a DynamoRIO-based tool that collects coverage information
 * from a binary. Primary goal this script is to have coverage log
 * files that work in Lighthouse.
 *
 * License: GNU GPL, version 2 or later.
 *   See the COPYING file in the top-level directory.
 */

#include <inttypes.h>
#include <assert.h>
#include <stdlib.h>
#include <inttypes.h>
#include <string.h>
#include <unistd.h>
#include <stdio.h>
#include <glib.h>
#include <fcntl.h>
#include <sys/types.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <errno.h>
#include <qemu-plugin.h>
#define SHM_LENGTH (1 << 20)

/* Translated blocks */
static GPtrArray *blocks;
static char* trace_bit, *trace_map;
static int fcov;
static uint64_t cur_location=0, pre_location=0, first_exec = 0;
static const char *cov_file = "", *funcaddr_suffix = "";
static const char *funcaddr_prefix = "bl #0x"; 
static char funcaddr[16];
static GMutex lock;

QEMU_PLUGIN_EXPORT int qemu_plugin_version = QEMU_PLUGIN_VERSION;

typedef struct {
    uint32_t start;
    uint16_t size;
    uint16_t mod_id;
    bool     exec;
} bb_entry_t;

static void shm_init(void) 
{
    
    g_mutex_lock(&lock);
 

    if ((fcov = shm_open(cov_file, O_CREAT|O_RDWR, 0777)) == -1) {
        perror("open coverage file failed");
    }

    if (ftruncate(fcov, (SHM_LENGTH+4)*sizeof(char)) < 0) {
        perror("initial ftruncate");
    }

    trace_map = (char*)mmap(0, (SHM_LENGTH+4)*sizeof(char), PROT_READ | PROT_WRITE, MAP_SHARED, fcov, 0);
    if (trace_map == MAP_FAILED) {
        perror("mmap coverage file failed");
    }

    trace_bit = (char*)malloc(sizeof(char)*(SHM_LENGTH+4));
    memset(trace_bit, 0, SHM_LENGTH+4*sizeof(char));
    
    g_mutex_unlock(&lock);
}

static void shm_reset(void) 
{
    
    g_mutex_lock(&lock);
    memcpy(trace_map, trace_bit, sizeof(char)*SHM_LENGTH+4);
    msync(trace_map, sizeof(trace_map), MS_SYNC); 
    // munmap(trace_map, sizeof(trace_map));   
    // close(fcov); 
    memset(trace_bit, 0, sizeof(char)*SHM_LENGTH+4);  
    
    g_mutex_unlock(&lock);
}

static void plugin_init(void)
{
    shm_init();
    blocks = g_ptr_array_sized_new(128);
}

static void plugin_exit(qemu_plugin_id_t id, void *p)
{
    g_mutex_lock(&lock);

    /* Clear */
    g_ptr_array_free(blocks, true);
    memcpy(trace_map, trace_bit, SHM_LENGTH+4);
    munmap(trace_map, sizeof(trace_map));
    close(fcov);

    g_mutex_unlock(&lock);
}

static void vcpu_tb_exec(unsigned int cpu_index, void *udata)
{
    bb_entry_t *bb = (bb_entry_t *) udata;

    g_mutex_lock(&lock);

    bb->exec = true;
    // is first block
    if ( !first_exec )  {
        pre_location = cur_location;
        first_exec = 1;
    }
    // coverage write to file 
    cur_location = bb->start;
    int edge_path_num = ((cur_location+pre_location)^cur_location) % SHM_LENGTH;
    if (edge_path_num >= SHM_LENGTH) {
        perror("overflow!!!");
    }
    trace_bit[edge_path_num] = (trace_bit[edge_path_num]+1)%CHAR_MAX;
    pre_location = cur_location >> 1;

    g_mutex_unlock(&lock);
}

static void vcpu_tb_trans(qemu_plugin_id_t id, struct qemu_plugin_tb *tb)
{

    uint64_t pc = qemu_plugin_tb_vaddr(tb);
    size_t n = qemu_plugin_tb_n_insns(tb);

    g_mutex_lock(&lock);

    bb_entry_t *bb = g_new0(bb_entry_t, 1);
    for (int i = 0; i < n; i++) {
        bb->size += qemu_plugin_insn_size(qemu_plugin_tb_get_insn(tb, i));
    }

    bb->start = pc;
    bb->mod_id = 0;
    bb->exec = false;
    g_ptr_array_add(blocks, bb);

    g_mutex_unlock(&lock);
    qemu_plugin_register_vcpu_tb_exec_cb(tb, vcpu_tb_exec,
                                         QEMU_PLUGIN_CB_NO_REGS,
                                         (void *)bb);
 
    size_t n_insns;
    size_t i;
    n_insns = qemu_plugin_tb_n_insns(tb); 

    // TODO: this is really time cosuming, need to optimize
    for (i = 0; i < n_insns; i++) {
        struct qemu_plugin_insn *insn = qemu_plugin_tb_get_insn(tb, i);
        char * disas_str = qemu_plugin_insn_disas(insn);
        if(!g_strcmp0(disas_str, funcaddr)) {  
            shm_reset(); 
            shm_init();  
            break;
        }
    }   

}

QEMU_PLUGIN_EXPORT
int qemu_plugin_install(qemu_plugin_id_t id, const qemu_info_t *info,
                        int argc, char **argv)
{
    for (int i = 0; i < argc; i++) {
        g_autofree char **tokens = g_strsplit(argv[i], "=", 4);
        if (g_strcmp0(tokens[0], "coverfile") == 0) {
            cov_file = g_strdup(tokens[1]);
        } else if (g_strcmp0(tokens[0], "funcaddr") == 0) {
            funcaddr_suffix = g_strdup(tokens[1]);
        } 
    }
    for (int i = 0; i < strlen(funcaddr_prefix); ++i) 
        funcaddr[i] = funcaddr_prefix[i];
    for (int i = 0; i < strlen(funcaddr_suffix); ++i) 
        funcaddr[i+strlen(funcaddr_prefix)] = funcaddr_suffix[i];

    plugin_init();
    qemu_plugin_register_vcpu_tb_trans_cb(id, vcpu_tb_trans);
    qemu_plugin_register_atexit_cb(id, plugin_exit, NULL);

    return 0;
}

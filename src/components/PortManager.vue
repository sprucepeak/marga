<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { RunningProcessPortInfo } from '../types';
import { invoke } from '@tauri-apps/api/core';
import {
  Network,
  RefreshCw,
  Search,
  XCircle,
  AlertCircle
} from 'lucide-vue-next';

const processPorts = ref<RunningProcessPortInfo[]>([]);
const loadingPorts = ref(false);
const portSearchQuery = ref('');
const killingPid = ref<number | null>(null);
const actionMsg = ref<string | null>(null);

let refreshInterval: number | null = null;

async function fetchProcessPorts() {
  loadingPorts.value = true;
  try {
    const list = await invoke<RunningProcessPortInfo[]>('list_process_ports');
    processPorts.value = list;
  } catch (err) {
    console.error('Failed to fetch process ports:', err);
  } finally {
    loadingPorts.value = false;
  }
}

async function handleKillProcess(proc: RunningProcessPortInfo) {
  if (!confirm(`确定要强制关闭进程 "${proc.name}" (PID: ${proc.pid}) 吗？`)) {
    return;
  }

  killingPid.value = proc.pid;
  try {
    await invoke('kill_process', { pid: proc.pid });
    actionMsg.value = `已成功关闭进程: ${proc.name} (PID: ${proc.pid})`;
    setTimeout(() => {
      actionMsg.value = null;
    }, 3000);
    await fetchProcessPorts();
  } catch (err: any) {
    alert(`关闭进程失败: ${String(err)}`);
  } finally {
    killingPid.value = null;
  }
}

const filteredProcessPorts = computed(() => {
  const query = portSearchQuery.value.trim().toLowerCase();
  if (!query) return processPorts.value;
  return processPorts.value.filter((p) => {
    if (p.name.toLowerCase().includes(query)) return true;
    if (p.pid.toString().includes(query)) return true;
    if (p.path.toLowerCase().includes(query)) return true;
    return p.ports.some(
      (port) =>
        port.port.toString().includes(query) ||
        port.protocol.toLowerCase().includes(query) ||
        port.local_addr.toLowerCase().includes(query)
    );
  });
});

onMounted(() => {
  fetchProcessPorts();
  refreshInterval = window.setInterval(() => {
    fetchProcessPorts();
  }, 10000);
});

onUnmounted(() => {
  if (refreshInterval !== null) {
    clearInterval(refreshInterval);
  }
});
</script>

<template>
  <div class="h-full flex flex-col p-6 space-y-4">
    <!-- Header Actions -->
    <div class="flex items-center justify-between gap-6 pb-3 border-b border-slate-800">
      <div class="min-w-0 flex-1">
        <h2 class="text-sm font-semibold text-white">端口管理与进程占用</h2>
        <p class="text-xs text-slate-400 mt-1 leading-relaxed">
          监控当前运行中的进程及其占用的网络端口与监听状态，可一键关闭异常或长连接进程
        </p>
      </div>

      <div class="flex items-center space-x-2.5 shrink-0">
        <!-- Search box -->
        <div class="relative w-48 sm:w-64">
          <Search class="w-3.5 h-3.5 text-slate-500 absolute left-2.5 top-2.5" />
          <input
            v-model="portSearchQuery"
            type="text"
            placeholder="搜索进程名 / PID / 端口..."
            class="w-full pl-8 pr-3 py-1.5 bg-[#080d1a] border border-slate-700/80 rounded-xl text-xs text-white focus:outline-none focus:border-cyan-500 transition-colors"
          />
        </div>

        <!-- Refresh button -->
        <button
          @click="fetchProcessPorts"
          :disabled="loadingPorts"
          class="flex items-center space-x-1.5 px-3 py-1.5 rounded-xl bg-slate-800/80 hover:bg-slate-700 text-slate-300 border border-slate-700 text-xs font-medium transition-colors cursor-pointer disabled:opacity-50"
          title="刷新列表"
        >
          <RefreshCw class="w-3.5 h-3.5" :class="{ 'animate-spin': loadingPorts }" />
          <span>刷新</span>
        </button>
      </div>
    </div>

    <!-- Action Toast / Notice -->
    <div
      v-if="actionMsg"
      class="px-4 py-2.5 rounded-xl bg-emerald-500/20 border border-emerald-500/40 text-emerald-300 text-xs flex items-center space-x-2 animate-fade-in"
    >
      <AlertCircle class="w-4 h-4 shrink-0" />
      <span>{{ actionMsg }}</span>
    </div>

    <!-- Process & Port Table Container -->
    <div class="flex-1 bg-[#0c1424]/60 border border-cyan-950/40 rounded-2xl overflow-hidden flex flex-col backdrop-blur-sm">
      <div class="grid grid-cols-12 gap-3 px-4 py-3 bg-slate-950/40 border-b border-slate-800 text-[11px] font-semibold text-slate-400 uppercase tracking-wider">
        <div class="col-span-4">进程信息 (名称 / PID / 路径)</div>
        <div class="col-span-6">监听 / 占用端口与协议</div>
        <div class="col-span-2 text-right">操作</div>
      </div>

      <!-- Empty State -->
      <div v-if="filteredProcessPorts.length === 0" class="flex-1 flex flex-col items-center justify-center text-slate-500 p-8 space-y-3">
        <div class="w-12 h-12 rounded-2xl bg-slate-800/50 flex items-center justify-center text-slate-400">
          <Network class="w-6 h-6" />
        </div>
        <p class="text-xs">{{ loadingPorts ? '正在扫描网络端点与端口占用...' : '未找到匹配的网络进程或端口' }}</p>
      </div>

      <!-- Rows -->
      <div v-else class="flex-1 overflow-y-auto divide-y divide-slate-800/50">
        <div
          v-for="proc in filteredProcessPorts"
          :key="proc.pid"
          class="grid grid-cols-12 gap-3 px-4 py-3 items-center hover:bg-slate-800/25 transition-colors group"
        >
          <!-- Process Info -->
          <div class="col-span-4 min-w-0 pr-2">
            <div class="flex items-center space-x-2">
              <span class="text-xs font-semibold text-slate-200 truncate group-hover:text-cyan-300 transition-colors">
                {{ proc.name }}
              </span>
              <span class="text-[10px] px-1.5 py-0.2 rounded bg-slate-800/90 text-slate-400 font-mono shrink-0">
                PID: {{ proc.pid }}
              </span>
            </div>
            <p
              v-if="proc.path"
              class="text-[10px] text-slate-500 truncate mt-0.5 font-mono select-all"
              :title="proc.path"
            >
              {{ proc.path }}
            </p>
          </div>

          <!-- Ports Badge List -->
          <div class="col-span-6 flex flex-wrap gap-1.5 max-h-24 overflow-y-auto py-0.5">
            <span
              v-for="(p, idx) in proc.ports"
              :key="idx"
              class="inline-flex items-center space-x-1 px-1.5 py-0.5 rounded bg-[#091020] border border-cyan-950/60 text-[10px] font-mono text-cyan-300"
              :title="`${p.protocol} ${p.local_addr}:${p.port} (${p.state})`"
            >
              <span class="text-slate-400 text-[9px] uppercase">{{ p.protocol }}</span>
              <span class="font-semibold text-white">:{{ p.port }}</span>
              <span
                v-if="p.state === 'LISTEN'"
                class="w-1.5 h-1.5 rounded-full bg-emerald-400 inline-block ml-0.5"
                title="监听中"
              ></span>
            </span>
          </div>

          <!-- Kill Process Action Button -->
          <div class="col-span-2 flex justify-end">
            <button
              @click="handleKillProcess(proc)"
              :disabled="killingPid === proc.pid"
              class="flex items-center space-x-1.5 px-2.5 py-1.5 rounded-lg bg-rose-500/10 hover:bg-rose-500/20 text-rose-400 hover:text-rose-300 border border-rose-500/20 text-xs font-medium transition-all cursor-pointer disabled:opacity-50"
              title="强制关闭此进程"
            >
              <XCircle class="w-3.5 h-3.5" :class="{ 'animate-spin': killingPid === proc.pid }" />
              <span>关闭进程</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

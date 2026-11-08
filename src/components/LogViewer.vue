<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue';
import { ConnectionLog } from '../types';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { Activity, Trash2, CheckCircle2, XCircle, Clock, Search, Pin, Globe } from 'lucide-vue-next';

const emit = defineEmits<{
  (e: 'addProcessRule', path: string): void;
  (e: 'addDomainRule', domain: string): void;
}>();

const logs = ref<ConnectionLog[]>([]);
const autoScroll = ref(true);
const searchQuery = ref('');
const logContainer = ref<HTMLDivElement | null>(null);

let unlisten: UnlistenFn | null = null;

const filteredLogs = computed(() => {
  const q = searchQuery.value.trim().toLowerCase();
  if (!q) return logs.value;
  return logs.value.filter(
    (l) =>
      l.process_name.toLowerCase().includes(q) ||
      l.target.toLowerCase().includes(q) ||
      String(l.pid).includes(q)
  );
});

function extractDomain(target: string): string | null {
  const host = target.split(':')[0].trim();
  // If host is an IP address, return null
  if (/^(\d{1,3}\.){3}\d{1,3}$/.test(host)) return null;
  return host || null;
}

onMounted(async () => {
  unlisten = await listen<ConnectionLog>('new-connection-log', (event) => {
    const newLog = event.payload;
    const existingIdx = logs.value.findIndex((l) => l.id === newLog.id);
    if (existingIdx !== -1) {
      logs.value[existingIdx] = newLog;
    } else {
      logs.value.push(newLog);
      if (logs.value.length > 500) {
        logs.value.shift();
      }
    }

    if (autoScroll.value && !searchQuery.value.trim()) {
      nextTick(() => {
        if (logContainer.value) {
          logContainer.value.scrollTop = logContainer.value.scrollHeight;
        }
      });
    }
  });
});

onUnmounted(() => {
  if (unlisten) {
    unlisten();
  }
});

function clearLogs() {
  logs.value = [];
}
</script>

<template>
  <div class="h-full flex flex-col p-6 space-y-4">
    <!-- Header -->
    <div class="flex items-center justify-between gap-4 pb-3 border-b border-cyan-950/40">
      <div class="flex items-center space-x-2.5 shrink-0">
        <Activity class="w-5 h-5 text-cyan-400" />
        <div>
          <h2 class="text-sm font-semibold text-white">实时连接日志</h2>
          <p class="text-xs text-slate-400">展示当前被重定向程序的出站 TCP 连接与 SNI 远程解析状态</p>
        </div>
      </div>

      <!-- Search filter -->
      <div class="relative flex-1 max-w-xs">
        <Search class="w-3.5 h-3.5 text-slate-500 absolute left-3 top-2.5" />
        <input
          v-model="searchQuery"
          type="text"
          placeholder="过滤进程、域名或 PID..."
          class="w-full bg-[#080d1a] border border-slate-700/80 rounded-xl pl-8 pr-3 py-1.5 text-xs text-slate-200 placeholder-slate-500 focus:outline-none focus:border-cyan-500 transition-colors"
        />
      </div>

      <div class="flex items-center space-x-4 shrink-0">
        <label class="flex items-center space-x-2 text-xs text-slate-400 cursor-pointer select-none">
          <button
            type="button"
            role="switch"
            :aria-checked="autoScroll"
            @click="autoScroll = !autoScroll"
            class="relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none"
            :class="autoScroll ? 'bg-cyan-500 shadow-sm shadow-cyan-500/30' : 'bg-slate-800'"
          >
            <span
              class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow-md ring-0 transition duration-200 ease-in-out"
              :class="autoScroll ? 'translate-x-4' : 'translate-x-0'"
            />
          </button>
          <span>自动滚动</span>
        </label>

        <button
          @click="clearLogs"
          class="flex items-center space-x-1 px-3 py-1.5 rounded-xl bg-slate-800/80 hover:bg-slate-700 text-xs text-slate-300 border border-slate-700 transition-colors cursor-pointer"
        >
          <Trash2 class="w-3.5 h-3.5" />
          <span>清空日志</span>
        </button>
      </div>
    </div>

    <!-- Logs List / Table -->
    <div 
      ref="logContainer"
      class="flex-1 overflow-y-auto rounded-2xl border border-cyan-950/40 bg-[#0c1424]/60 p-2 font-mono text-xs space-y-1 backdrop-blur-sm"
    >
      <div v-if="filteredLogs.length === 0" class="h-full flex flex-col items-center justify-center py-20 text-slate-500 space-y-2">
        <Activity class="w-8 h-8 text-slate-700" />
        <p class="text-xs">{{ searchQuery ? '未找到符合过滤条件的日志' : '暂无连接记录，开启代理后运行目标程序即可看到捕获的流量' }}</p>
      </div>

      <div
        v-for="log in filteredLogs"
        :key="log.id"
        class="px-3 py-2 rounded-xl bg-slate-950/60 border border-slate-800/60 flex items-center justify-between hover:bg-slate-800/30 transition-colors group"
      >
        <div class="flex items-center space-x-3 min-w-0 pr-4">
          <span class="text-slate-500 text-[11px] shrink-0 flex items-center space-x-1">
            <Clock class="w-3 h-3 text-slate-600" />
            <span>{{ log.timestamp }}</span>
          </span>

          <span class="text-slate-300 font-semibold truncate max-w-[180px]" :title="log.process_name">
            {{ log.process_name.split('\\').pop() }}
          </span>

          <span class="text-[10px] px-1.5 py-0.2 rounded bg-slate-800 text-slate-400 shrink-0">
            PID:{{ log.pid }}
          </span>

          <span class="text-slate-400 truncate max-w-sm" :title="log.target">
            {{ log.target }}
          </span>
        </div>

        <div class="flex items-center space-x-2.5 shrink-0">
          <!-- Quick Add Rule actions (Shown on hover) -->
          <div class="opacity-0 group-hover:opacity-100 flex items-center space-x-1.5 transition-opacity mr-1">
            <button
              v-if="log.process_name && !log.process_name.startsWith('PID:')"
              @click="emit('addProcessRule', log.process_name)"
              title="将此程序加入规则"
              class="px-2 py-0.5 rounded bg-cyan-950/80 hover:bg-cyan-900 border border-cyan-800/60 text-cyan-300 text-[10px] font-sans flex items-center space-x-1 cursor-pointer transition-colors"
            >
              <Pin class="w-2.5 h-2.5" />
              <span>钉程序</span>
            </button>

            <button
              v-if="extractDomain(log.target)"
              @click="emit('addDomainRule', extractDomain(log.target)!)"
              title="将此域名加入规则"
              class="px-2 py-0.5 rounded bg-indigo-950/80 hover:bg-indigo-900 border border-indigo-800/60 text-indigo-300 text-[10px] font-sans flex items-center space-x-1 cursor-pointer transition-colors"
            >
              <Globe class="w-2.5 h-2.5" />
              <span>加域名</span>
            </button>
          </div>

          <span class="text-[10px] px-1.5 py-0.5 rounded bg-slate-800 text-slate-300 border border-slate-700">
            {{ log.protocol }}
          </span>

          <!-- Status badge -->
          <div class="flex items-center space-x-1">
            <template v-if="log.status === 'success'">
              <CheckCircle2 class="w-3.5 h-3.5 text-emerald-400" />
              <span class="text-emerald-400 text-[11px]">代理转发</span>
            </template>
            <template v-else-if="log.status === 'connecting'">
              <span class="w-2 h-2 rounded-full bg-amber-400 animate-ping"></span>
              <span class="text-amber-400 text-[11px]">连接中</span>
            </template>
            <template v-else>
              <XCircle class="w-3.5 h-3.5 text-rose-400" />
              <span class="text-rose-400 text-[11px]" :title="log.error_msg || '失败'">失败</span>
            </template>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue';
import { ConnectionLog } from '../types';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { Activity, Trash2, CheckCircle2, XCircle, Clock } from 'lucide-vue-next';

const logs = ref<ConnectionLog[]>([]);
const autoScroll = ref(true);
const logContainer = ref<HTMLDivElement | null>(null);

let unlisten: UnlistenFn | null = null;

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

    if (autoScroll.value) {
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
    <div class="flex items-center justify-between pb-3 border-b border-cyan-950/40">
      <div class="flex items-center space-x-2.5">
        <Activity class="w-5 h-5 text-cyan-400" />
        <div>
          <h2 class="text-sm font-semibold text-white">实时连接日志</h2>
          <p class="text-xs text-slate-400">展示当前被重定向程序的出站 TCP 连接与 SNI 远程解析状态</p>
        </div>
      </div>

      <div class="flex items-center space-x-4">
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
      <div v-if="logs.length === 0" class="h-full flex flex-col items-center justify-center py-20 text-slate-500 space-y-2">
        <Activity class="w-8 h-8 text-slate-700" />
        <p class="text-xs">暂无连接记录，开启代理后运行目标程序即可看到捕获的流量</p>
      </div>

      <div
        v-for="log in logs"
        :key="log.id"
        class="px-3 py-2 rounded-xl bg-slate-950/60 border border-slate-800/60 flex items-center justify-between hover:bg-slate-800/30 transition-colors"
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

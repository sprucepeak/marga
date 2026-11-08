<script setup lang="ts">
import { ShieldAlert, ShieldCheck, Play, Square, Loader2 } from 'lucide-vue-next';

defineProps<{
  isRunning: boolean;
  isAdmin: boolean;
  isStarting: boolean;
}>();

const emit = defineEmits<{
  (e: 'toggleProxy'): void;
  (e: 'restartAdmin'): void;
}>();
</script>

<template>
  <header class="h-16 border-b border-cyan-950/40 bg-[#0c1322]/85 backdrop-blur-md px-6 flex items-center justify-between select-none shrink-0" data-tauri-drag-region>
    <!-- Logo & Title -->
    <div class="flex items-center space-x-3 pointer-events-none">
      <div class="w-9 h-9 rounded-xl bg-gradient-to-tr from-cyan-500 via-indigo-500 to-purple-600 p-[1.5px] shadow-lg shadow-cyan-500/20">
        <div class="w-full h-full bg-[#0c1322] rounded-[10px] flex items-center justify-center">
          <span class="text-transparent bg-clip-text bg-gradient-to-tr from-cyan-400 via-sky-300 to-indigo-300 font-black text-lg tracking-wider">M</span>
        </div>
      </div>
      <div>
        <div class="flex items-center space-x-2">
          <h1 class="text-base font-bold text-white tracking-wide">Marga</h1>
          <span class="text-[10px] uppercase font-semibold px-1.5 py-0.5 rounded bg-cyan-950/50 text-cyan-300/80 border border-cyan-800/40">v1.0.0</span>
        </div>
        <p class="text-xs text-slate-400">指定软件与域名，独享专属代理通道</p>
      </div>
    </div>

    <!-- Center Status & Admin Warnings -->
    <div class="flex items-center space-x-4">
      <div v-if="!isAdmin" class="flex items-center space-x-2 px-3 py-1.5 rounded-lg bg-amber-500/10 border border-amber-500/20 text-amber-400 text-xs">
        <ShieldAlert class="w-4 h-4 text-amber-400 shrink-0" />
        <span>普通权限模式（点击「启动代理」将自动请求提权）</span>
      </div>

      <div v-else class="flex items-center space-x-1.5 text-xs text-emerald-400 bg-emerald-500/10 px-2.5 py-1 rounded-md border border-emerald-500/20">
        <ShieldCheck class="w-3.5 h-3.5" />
        <span>管理员权限已就绪</span>
      </div>
    </div>

    <!-- Right Controls: Toggle Button -->
    <div class="flex items-center space-x-3">
      <div class="flex items-center space-x-2 mr-2">
        <span 
          class="w-2.5 h-2.5 rounded-full transition-colors duration-300"
          :class="isRunning ? 'bg-emerald-500 shadow-md shadow-emerald-500/50 animate-pulse' : 'bg-slate-600'"
        ></span>
        <span class="text-xs font-medium" :class="isRunning ? 'text-emerald-400' : 'text-slate-400'">
          {{ isRunning ? '代理服务运行中' : '代理服务已停止' }}
        </span>
      </div>

      <button
        @click="emit('toggleProxy')"
        :disabled="isStarting"
        class="flex items-center space-x-2 px-5 py-2 rounded-xl text-sm font-semibold transition-all duration-300 cursor-pointer shadow-lg disabled:opacity-50"
        :class="isRunning 
          ? 'bg-rose-500/10 text-rose-400 border border-rose-500/30 hover:bg-rose-500/20 shadow-rose-500/10' 
          : 'bg-gradient-to-r from-cyan-500 to-indigo-600 hover:from-cyan-400 hover:to-indigo-500 text-white shadow-cyan-500/25 hover:shadow-cyan-500/40'"
      >
        <Loader2 v-if="isStarting" class="w-4 h-4 animate-spin text-cyan-200" />
        <template v-else>
          <Square v-if="isRunning" class="w-4 h-4 fill-current" />
          <Play v-else class="w-4 h-4 fill-current" />
        </template>
        <span>{{ isRunning ? '停止代理' : '启动代理' }}</span>
      </button>
    </div>
  </header>
</template>

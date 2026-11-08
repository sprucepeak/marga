<script setup lang="ts">
import { ref } from 'vue';
import { DomainRule, DomainMatchType } from '../types';
import { Globe, Plus, Trash2, CheckCircle2, Circle } from 'lucide-vue-next';

const props = defineProps<{
  rules: DomainRule[];
}>();

const emit = defineEmits<{
  (e: 'update:rules', rules: DomainRule[]): void;
  (e: 'save'): void;
}>();

const showAddModal = ref(false);
const newPattern = ref('');
const newMatchType = ref<DomainMatchType>('suffix');

function addRule() {
  const pattern = newPattern.value.trim();
  if (!pattern) return;

  const newRule: DomainRule = {
    id: Date.now().toString(),
    pattern,
    match_type: newMatchType.value,
    enabled: true,
  };

  emit('update:rules', [...props.rules, newRule]);
  emit('save');

  newPattern.value = '';
  newMatchType.value = 'suffix';
  showAddModal.value = false;
}

function removeRule(id: string) {
  emit('update:rules', props.rules.filter((r) => r.id !== id));
  emit('save');
}

function toggleRule(rule: DomainRule) {
  rule.enabled = !rule.enabled;
  emit('update:rules', [...props.rules]);
  emit('save');
}

function getMatchTypeName(type: DomainMatchType): string {
  switch (type) {
    case 'suffix':
      return '主域名 / 后缀';
    case 'exact':
      return '完整匹配';
    case 'keyword':
      return '域名关键字';
    case 'prefix':
      return '域名前缀';
  }
}
</script>

<template>
  <div class="h-full flex flex-col p-6 space-y-4">
    <!-- Header Actions -->
    <div class="flex items-center justify-between gap-6 pb-3 border-b border-slate-800">
      <div class="min-w-0 flex-1">
        <h2 class="text-sm font-semibold text-white">全局域名分流规则</h2>
        <p class="text-xs text-slate-400 mt-1 leading-relaxed">所有应用程序访问匹配以下域名规则的请求，均将被透明重定向至指定代理</p>
      </div>

      <button
        @click="showAddModal = true"
        class="flex items-center space-x-1.5 px-3 py-1.5 rounded-xl bg-gradient-to-r from-cyan-500 to-indigo-600 hover:from-cyan-400 hover:to-indigo-500 text-xs font-semibold text-white shadow-md shadow-cyan-500/20 hover:shadow-cyan-500/30 transition-all cursor-pointer shrink-0"
      >
        <Plus class="w-3.5 h-3.5" />
        <span>添加域名规则</span>
      </button>
    </div>

    <!-- Domain Rules Table -->
    <div class="flex-1 bg-[#0c1424]/60 border border-cyan-950/40 rounded-2xl overflow-hidden flex flex-col backdrop-blur-sm">
      <div class="grid grid-cols-12 gap-4 px-4 py-3 bg-slate-950/40 border-b border-slate-800 text-[11px] font-semibold text-slate-400 uppercase tracking-wider">
        <div class="col-span-1">状态</div>
        <div class="col-span-5">域名 / 匹配表达式</div>
        <div class="col-span-4">匹配模式</div>
        <div class="col-span-2 text-right">操作</div>
      </div>

      <!-- Empty State -->
      <div v-if="rules.length === 0" class="flex-1 flex flex-col items-center justify-center text-slate-500 p-8 space-y-3">
        <div class="w-12 h-12 rounded-2xl bg-slate-800/50 flex items-center justify-center text-slate-400">
          <Globe class="w-6 h-6" />
        </div>
        <p class="text-xs">暂无配置域名规则，点击右上角添加（支持主域名、完整匹配、关键字等）</p>
      </div>

      <!-- List Rows -->
      <div v-else class="flex-1 overflow-y-auto divide-y divide-slate-800/60">
        <div
          v-for="rule in rules"
          :key="rule.id"
          class="grid grid-cols-12 gap-4 px-4 py-3.5 items-center hover:bg-slate-800/30 transition-colors group"
        >
          <!-- Status Toggle -->
          <div class="col-span-1">
            <button
              @click="toggleRule(rule)"
              class="cursor-pointer transition-transform active:scale-95"
              :title="rule.enabled ? '点击禁用' : '点击启用'"
            >
              <CheckCircle2 v-if="rule.enabled" class="w-4 h-4 text-emerald-400" />
              <Circle v-else class="w-4 h-4 text-slate-600" />
            </button>
          </div>

          <!-- Domain Pattern -->
          <div class="col-span-5 flex items-center space-x-2">
            <span class="text-xs font-semibold text-white tracking-wide">{{ rule.pattern }}</span>
          </div>

          <!-- Match Type Badge -->
          <div class="col-span-4">
            <span class="text-[11px] px-2 py-0.5 rounded-md bg-indigo-500/10 text-indigo-400 border border-indigo-500/20 font-medium">
              {{ getMatchTypeName(rule.match_type) }}
            </span>
          </div>

          <!-- Action -->
          <div class="col-span-2 flex justify-end">
            <button
              @click="removeRule(rule.id)"
              class="p-1.5 rounded-lg text-slate-400 hover:text-rose-400 hover:bg-rose-500/10 transition-colors cursor-pointer opacity-70 group-hover:opacity-100"
              title="删除规则"
            >
              <Trash2 class="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Add Domain Rule Modal -->
    <div
      v-if="showAddModal"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm p-4"
    >
      <div class="bg-[#0e1626] border border-cyan-950/60 rounded-2xl w-full max-w-md p-6 space-y-5 shadow-2xl">
        <h3 class="text-sm font-semibold text-white">添加指定代理域名</h3>

        <div class="space-y-4">
          <div>
            <label class="block text-xs font-medium text-slate-300 mb-1.5">域名表达式 / 关键字</label>
            <input
              v-model="newPattern"
              type="text"
              placeholder="例如：github.com 或 google"
              class="w-full px-3 py-2 bg-[#080d1a] border border-slate-700/80 rounded-xl text-xs text-white focus:outline-none focus:border-cyan-500"
              @keydown.enter="addRule"
            />
          </div>

          <div>
            <label class="block text-xs font-medium text-slate-300 mb-1.5">匹配模式</label>
            <select
              v-model="newMatchType"
              class="w-full px-3 py-2 bg-[#080d1a] border border-slate-700/80 rounded-xl text-xs text-white focus:outline-none focus:border-cyan-500"
            >
              <option value="suffix">主域名 / 后缀 (匹配 github.com 及 *.github.com)</option>
              <option value="exact">完整域名 (精确匹配 如 api.github.com)</option>
              <option value="keyword">域名关键字 (包含特定词 如 google)</option>
              <option value="prefix">域名前缀 (如 api.)</option>
            </select>
          </div>
        </div>

        <div class="flex justify-end space-x-2.5 pt-2">
          <button
            @click="showAddModal = false"
            class="px-4 py-2 rounded-xl text-xs font-medium text-slate-400 hover:bg-slate-800 transition-colors cursor-pointer"
          >
            取消
          </button>
          <button
            @click="addRule"
            :disabled="!newPattern.trim()"
            class="px-4 py-2 rounded-xl text-xs font-semibold bg-gradient-to-r from-cyan-500 to-indigo-600 hover:from-cyan-400 hover:to-indigo-500 text-white shadow-md shadow-cyan-500/20 transition-all cursor-pointer disabled:opacity-50"
          >
            确认添加
          </button>
        </div>
      </div>
    </div>
  </div>
</template>


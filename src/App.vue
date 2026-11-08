<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { AppConfig } from './types';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';
import Navbar from './components/Navbar.vue';
import ProcessList from './components/ProcessList.vue';
import ProxySettings from './components/ProxySettings.vue';
import LogViewer from './components/LogViewer.vue';
import DomainList from './components/DomainList.vue';
import { ListFilter, Globe, Server, Activity, Settings2, CheckCircle2, Download, Upload } from 'lucide-vue-next';

const currentTab = ref<'rules' | 'domains' | 'proxy' | 'logs' | 'settings'>('rules');

const isRunning = ref(false);
const isAdmin = ref(false);
const isStarting = ref(false);
const toast = ref<string | null>(null);

const config = ref<AppConfig>({
  proxy: {
    protocol: 'socks5',
    host: '127.0.0.1',
    port: 10808,
    username: null,
    password: null,
  },
  rules: [],
  domain_rules: [],
  settings: {
    bypass_lan: true,
    block_udp_443: true,
    auto_start: false,
    minimize_to_tray: true,
  },
});

function showToast(msg: string) {
  toast.value = msg;
  setTimeout(() => {
    toast.value = null;
  }, 3000);
}

async function loadData() {
  try {
    const loadedConfig = await invoke<AppConfig>('get_config');
    config.value = loadedConfig;

    try {
      const autostartEnabled = await isEnabled();
      config.value.settings.auto_start = autostartEnabled;
    } catch {
      // plugin not ready or fallback
    }

    const status = await invoke<{ is_running: boolean; is_admin: boolean }>('get_proxy_status');
    isRunning.value = status.is_running;
    isAdmin.value = status.is_admin;
  } catch (err) {
    console.error('Failed to load initial config/status:', err);
  }
}

function onAddProcessRuleFromLog(path: string) {
  const fileName = path.split(/[\\/]/).pop() || 'Unknown App';
  const name = fileName.replace(/\.exe$/i, '');
  const existing = config.value.rules.find((r) => r.path.toLowerCase() === path.toLowerCase());
  if (existing) {
    existing.enabled = true;
    showToast(`规则 "${name}" 已存在并启用`);
  } else {
    config.value.rules.push({
      id: Date.now().toString(),
      name,
      path,
      match_type: 'exact',
      include_children: true,
      enabled: true,
    });
    showToast(`已添加程序规则: ${name}`);
  }
  saveAllConfig();
}

function onAddDomainRuleFromLog(domain: string) {
  if (!config.value.domain_rules) {
    config.value.domain_rules = [];
  }
  const cleanDomain = domain.toLowerCase();
  const existing = config.value.domain_rules.find((r) => r.pattern.toLowerCase() === cleanDomain);
  if (existing) {
    existing.enabled = true;
    showToast(`域名 "${domain}" 规则已存在并启用`);
  } else {
    config.value.domain_rules.push({
      id: Date.now().toString(),
      pattern: cleanDomain,
      match_type: 'suffix',
      enabled: true,
    });
    showToast(`已添加域名规则: ${domain}`);
  }
  saveAllConfig();
}

function onExportConfig() {
  const jsonStr = JSON.stringify(config.value, null, 2);
  const blob = new Blob([jsonStr], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `marga-config-${new Date().toISOString().slice(0, 10)}.json`;
  a.click();
  URL.revokeObjectURL(url);
  showToast('配置已导出');
}

function onImportConfig() {
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = '.json,application/json';
  input.onchange = async (e: Event) => {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (!file) return;
    try {
      const text = await file.text();
      const imported = JSON.parse(text);
      if (imported.proxy && imported.rules && imported.settings) {
        config.value = imported;
        await saveAllConfig();
        showToast('配置导入成功！');
      } else {
        alert('导入失败：不是有效的 Marga 配置文件格式');
      }
    } catch (err: any) {
      alert('解析导入文件失败: ' + String(err));
    }
  };
  input.click();
}

async function onToggleAutoStart() {
  try {
    if (config.value.settings.auto_start) {
      await enable();
      showToast('已启用开机自启');
    } else {
      await disable();
      showToast('已关闭开机自启');
    }
    await saveAllConfig();
  } catch (err: any) {
    alert('设置开机自启失败: ' + String(err));
    // rollback toggle state on error
    try {
      config.value.settings.auto_start = await isEnabled();
    } catch {}
  }
}

async function saveAllConfig() {
  try {
    await invoke('save_config', { config: config.value });
    showToast('配置已保存');
  } catch (err: any) {
    alert('保存配置失败: ' + String(err));
  }
}

async function onToggleProxy() {
  if (isRunning.value) {
    try {
      await invoke('stop_proxy');
      isRunning.value = false;
      showToast('代理服务已停止');
    } catch (err: any) {
      alert('停止失败: ' + String(err));
    }
  } else {
    isStarting.value = true;
    try {
      await invoke('start_proxy');
      isRunning.value = true;
      showToast('代理服务已成功启动！');
    } catch (err: any) {
      alert('启动失败: ' + String(err));
    } finally {
      isStarting.value = false;
    }
  }
}

async function onRestartAdmin() {
  try {
    await invoke('trigger_restart_as_admin');
  } catch (err: any) {
    alert('以管理员重启失败: ' + String(err));
  }
}

onMounted(async () => {
  await loadData();
  listen('proxy-status-changed', async () => {
    try {
      const status = await invoke<{ is_running: boolean; is_admin: boolean }>('get_proxy_status');
      isRunning.value = status.is_running;
    } catch {}
  });
});
</script>

<template>
  <div class="h-screen w-screen flex flex-col bg-[#080d1a] select-none overflow-hidden">
    <!-- Navbar -->
    <Navbar 
      :is-running="isRunning" 
      :is-admin="isAdmin" 
      :is-starting="isStarting"
      @toggle-proxy="onToggleProxy"
      @restart-admin="onRestartAdmin"
    />

    <!-- Main Content Area -->
    <div class="flex-1 flex overflow-hidden">
      <!-- Sidebar / Navigation Tabs -->
      <aside class="w-48 bg-[#0b1220]/50 border-r border-cyan-950/40 flex flex-col p-3 space-y-1 shrink-0 backdrop-blur-sm">
        <button
          @click="currentTab = 'rules'"
          class="flex items-center space-x-2.5 px-3 py-2.5 rounded-xl text-xs font-medium transition-all cursor-pointer"
          :class="currentTab === 'rules' 
            ? 'bg-cyan-500/10 text-cyan-300 border border-cyan-500/30 font-semibold shadow-sm shadow-cyan-500/10' 
            : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/40'"
        >
          <ListFilter class="w-4 h-4" />
          <span>程序规则 ({{ config.rules.length }})</span>
        </button>

        <button
          @click="currentTab = 'domains'"
          class="flex items-center space-x-2.5 px-3 py-2.5 rounded-xl text-xs font-medium transition-all cursor-pointer"
          :class="currentTab === 'domains' 
            ? 'bg-cyan-500/10 text-cyan-300 border border-cyan-500/30 font-semibold shadow-sm shadow-cyan-500/10' 
            : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/40'"
        >
          <Globe class="w-4 h-4" />
          <span>域名规则 ({{ (config.domain_rules || []).length }})</span>
        </button>

        <button
          @click="currentTab = 'proxy'"
          class="flex items-center space-x-2.5 px-3 py-2.5 rounded-xl text-xs font-medium transition-all cursor-pointer"
          :class="currentTab === 'proxy' 
            ? 'bg-cyan-500/10 text-cyan-300 border border-cyan-500/30 font-semibold shadow-sm shadow-cyan-500/10' 
            : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/40'"
        >
          <Server class="w-4 h-4" />
          <span>代理设置</span>
        </button>

        <button
          @click="currentTab = 'logs'"
          class="flex items-center space-x-2.5 px-3 py-2.5 rounded-xl text-xs font-medium transition-all cursor-pointer"
          :class="currentTab === 'logs' 
            ? 'bg-cyan-500/10 text-cyan-300 border border-cyan-500/30 font-semibold shadow-sm shadow-cyan-500/10' 
            : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/40'"
        >
          <Activity class="w-4 h-4" />
          <span>实时日志</span>
        </button>

        <button
          @click="currentTab = 'settings'"
          class="flex items-center space-x-2.5 px-3 py-2.5 rounded-xl text-xs font-medium transition-all cursor-pointer"
          :class="currentTab === 'settings' 
            ? 'bg-cyan-500/10 text-cyan-300 border border-cyan-500/30 font-semibold shadow-sm shadow-cyan-500/10' 
            : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/40'"
        >
          <Settings2 class="w-4 h-4" />
          <span>高级设置</span>
        </button>
      </aside>

      <!-- Content Views -->
      <main class="flex-1 overflow-hidden relative">
        <ProcessList 
          v-if="currentTab === 'rules'"
          :rules="config.rules"
          @update:rules="config.rules = $event"
          @save="saveAllConfig"
        />

        <DomainList 
          v-else-if="currentTab === 'domains'"
          :rules="config.domain_rules || []"
          @update:rules="config.domain_rules = $event"
          @save="saveAllConfig"
        />

        <ProxySettings 
          v-else-if="currentTab === 'proxy'"
          v-model="config.proxy"
          @save="saveAllConfig"
        />

        <LogViewer 
          v-else-if="currentTab === 'logs'"
          @add-process-rule="onAddProcessRuleFromLog"
          @add-domain-rule="onAddDomainRuleFromLog"
        />

        <!-- Advanced Settings Tab -->
        <div v-else-if="currentTab === 'settings'" class="max-w-2xl mx-auto py-8 px-4 space-y-6">
          <div class="bg-[#0c1424]/70 border border-cyan-950/50 rounded-2xl p-6 backdrop-blur space-y-5">
            <h3 class="text-sm font-semibold text-white pb-3 border-b border-cyan-950/60">高级流量策略设置</h3>

            <div class="space-y-4">
              <!-- Bypass LAN -->
              <div class="flex items-center justify-between py-1">
                <div class="pr-6">
                  <h4 class="text-xs font-semibold text-slate-200">局域网与本地回环默认直连 (F-23)</h4>
                  <p class="text-[11px] text-slate-400 mt-0.5">
                    自动绕过 127.0.0.1、10.0.0.0/8、172.16.0.0/12、192.168.0.0/16 地址段
                  </p>
                </div>
                <button
                  type="button"
                  role="switch"
                  :aria-checked="config.settings.bypass_lan"
                  @click="config.settings.bypass_lan = !config.settings.bypass_lan; saveAllConfig()"
                  class="relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none"
                  :class="config.settings.bypass_lan ? 'bg-cyan-500 shadow-sm shadow-cyan-500/30' : 'bg-slate-800'"
                >
                  <span
                    class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow-md ring-0 transition duration-200 ease-in-out"
                    :class="config.settings.bypass_lan ? 'translate-x-4' : 'translate-x-0'"
                  />
                </button>
              </div>

              <!-- Block UDP 443 -->
              <div class="flex items-center justify-between pt-3.5 border-t border-slate-800/60 py-1">
                <div class="pr-6">
                  <h4 class="text-xs font-semibold text-slate-200">阻断命中程序的 UDP 443 (QUIC 对策, F-27)</h4>
                  <p class="text-[11px] text-slate-400 mt-0.5">
                    丢弃目标程序的 QUIC 包，促使 Chromium/Electron 系应用自动回退至 TCP 代理
                  </p>
                </div>
                <button
                  type="button"
                  role="switch"
                  :aria-checked="config.settings.block_udp_443"
                  @click="config.settings.block_udp_443 = !config.settings.block_udp_443; saveAllConfig()"
                  class="relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none"
                  :class="config.settings.block_udp_443 ? 'bg-cyan-500 shadow-sm shadow-cyan-500/30' : 'bg-slate-800'"
                >
                  <span
                    class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow-md ring-0 transition duration-200 ease-in-out"
                    :class="config.settings.block_udp_443 ? 'translate-x-4' : 'translate-x-0'"
                  />
                </button>
              </div>

              <!-- Block UDP 53 (DNS Leak Protection) -->
              <div class="flex items-center justify-between pt-3.5 border-t border-slate-800/60 py-1">
                <div class="pr-6">
                  <h4 class="text-xs font-semibold text-slate-200">阻断命中程序的本地 UDP 53 DNS (防 DNS 泄露)</h4>
                  <p class="text-[11px] text-slate-400 mt-0.5">
                    拦截目标程序的传统明文 DNS 解析，强制其依赖 TLS SNI / 远端代理代解析，防止运营商 DNS 劫持与泄露
                  </p>
                </div>
                <button
                  type="button"
                  role="switch"
                  :aria-checked="!!config.settings.block_udp_53"
                  @click="config.settings.block_udp_53 = !config.settings.block_udp_53; saveAllConfig()"
                  class="relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none"
                  :class="config.settings.block_udp_53 ? 'bg-cyan-500 shadow-sm shadow-cyan-500/30' : 'bg-slate-800'"
                >
                  <span
                    class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow-md ring-0 transition duration-200 ease-in-out"
                    :class="config.settings.block_udp_53 ? 'translate-x-4' : 'translate-x-0'"
                  />
                </button>
              </div>

              <!-- Minimize to Tray -->
              <div class="flex items-center justify-between pt-3.5 border-t border-slate-800/60 py-1">
                <div class="pr-6">
                  <h4 class="text-xs font-semibold text-slate-200">最小化时隐藏至系统托盘</h4>
                  <p class="text-[11px] text-slate-400 mt-0.5">
                    点击窗口最小化按钮时隐藏在托盘后台运行，点击关闭按钮 (X) 则退出程序
                  </p>
                </div>
                <button
                  type="button"
                  role="switch"
                  :aria-checked="config.settings.minimize_to_tray"
                  @click="config.settings.minimize_to_tray = !config.settings.minimize_to_tray; saveAllConfig()"
                  class="relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none"
                  :class="config.settings.minimize_to_tray ? 'bg-cyan-500 shadow-sm shadow-cyan-500/30' : 'bg-slate-800'"
                >
                  <span
                    class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow-md ring-0 transition duration-200 ease-in-out"
                    :class="config.settings.minimize_to_tray ? 'translate-x-4' : 'translate-x-0'"
                  />
                </button>
              </div>

              <!-- Auto Start on Boot -->
              <div class="flex items-center justify-between pt-3.5 border-t border-slate-800/60 py-1">
                <div class="pr-6">
                  <h4 class="text-xs font-semibold text-slate-200">开机自动启动 (Auto-Start)</h4>
                  <p class="text-[11px] text-slate-400 mt-0.5">
                    Windows 系统登录时自动随系统启动并常驻后台运行
                  </p>
                </div>
                <button
                  type="button"
                  role="switch"
                  :aria-checked="config.settings.auto_start"
                  @click="onToggleAutoStart"
                  class="relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none"
                  :class="config.settings.auto_start ? 'bg-cyan-500 shadow-sm shadow-cyan-500/30' : 'bg-slate-800'"
                >
                  <span
                    class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow-md ring-0 transition duration-200 ease-in-out"
                    :class="config.settings.auto_start ? 'translate-x-4' : 'translate-x-0'"
                  />
                </button>
              </div>
            </div>
          </div>

          <!-- Backup & Restore Rules Box -->
          <div class="bg-[#0c1424]/70 border border-cyan-950/50 rounded-2xl p-6 backdrop-blur space-y-4">
            <h3 class="text-sm font-semibold text-white pb-3 border-b border-cyan-950/60">配置备份与迁移</h3>
            <p class="text-xs text-slate-400">将已添加的程序路径规则、域名规则及代理节点导出备份为 JSON 文件，或从备份恢复。</p>
            <div class="flex items-center space-x-3 pt-1">
              <button
                @click="onExportConfig"
                class="flex items-center space-x-1.5 px-3.5 py-2 rounded-xl bg-slate-800/80 hover:bg-slate-700 text-xs font-medium text-slate-200 border border-slate-700 transition-colors cursor-pointer"
              >
                <Download class="w-3.5 h-3.5 text-cyan-400" />
                <span>导出当前配置 (JSON)</span>
              </button>
              <button
                @click="onImportConfig"
                class="flex items-center space-x-1.5 px-3.5 py-2 rounded-xl bg-slate-800/80 hover:bg-slate-700 text-xs font-medium text-slate-200 border border-slate-700 transition-colors cursor-pointer"
              >
                <Upload class="w-3.5 h-3.5 text-indigo-400" />
                <span>从备份文件导入</span>
              </button>
            </div>
          </div>
        </div>
      </main>
    </div>

    <!-- Toast Notification -->
    <transition
      enter-active-class="transform ease-out duration-200 transition"
      enter-from-class="translate-y-2 opacity-0"
      enter-to-class="translate-y-0 opacity-100"
      leave-active-class="transition ease-in duration-150"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div 
        v-if="toast" 
        class="fixed bottom-6 right-6 z-50 flex items-center space-x-2 px-4 py-2.5 rounded-xl bg-indigo-600 text-white text-xs font-medium shadow-xl shadow-indigo-600/30"
      >
        <CheckCircle2 class="w-4 h-4 shrink-0" />
        <span>{{ toast }}</span>
      </div>
    </transition>
  </div>
</template>

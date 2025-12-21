<script>
  import { invoke } from '@tauri-apps/api/core';
  import { onMount, onDestroy } from 'svelte';
  import * as Card from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { Switch } from "$lib/components/ui/switch";
  import * as Alert from "$lib/components/ui/alert";
  import { Power, Activity, Network, Globe, AlertCircle, CheckCircle2 } from "lucide-svelte";
  import { Confetti } from 'svelte-confetti';

  let serverRunning = false;
  let loading = false;
  let message = '';
  let messageType = '';
  let port = 8080;
  let statusCheckInterval;
  let showConfetti = false;

  onMount(async () => {
    await checkStatus();
    await loadSettings();
    
    // Poll server status every 2 seconds
    statusCheckInterval = setInterval(checkStatus, 2000);
  });

  onDestroy(() => {
    if (statusCheckInterval) {
      clearInterval(statusCheckInterval);
    }
  });

  async function loadSettings() {
    try {
      const settings = await invoke('get_settings');
      port = settings.port;
    } catch (error) {
      console.error('Failed to load settings:', error);
    }
  }

  async function checkStatus() {
    try {
      serverRunning = await invoke('get_server_status');
    } catch (error) {
      console.error('Failed to check server status:', error);
    }
  }

  async function toggleServer() {
    loading = true;
    message = '';
    
    try {
      const settings = await invoke('get_settings');
      const newStatus = await invoke('toggle_server', { settings });
      serverRunning = newStatus;
      
      if (newStatus) {
        showMessage(`Server active on port ${settings.port}`, 'success');
        // Trigger confetti celebration
        showConfetti = true;
        setTimeout(() => { showConfetti = false; }, 3000);
      } else {
        showMessage('Server shutdown complete', 'success');
      }
    } catch (error) {
      showMessage('Failed to toggle server: ' + error, 'error');
      await checkStatus();
    } finally {
      loading = false;
    }
  }

  function showMessage(msg, type) {
    message = msg;
    messageType = type;
    setTimeout(() => {
      message = '';
    }, 3000);
  }

  function getServerUrl() {
    return `http://localhost:${port}`;
  }
</script>

<div class="space-y-6">
  <!-- Confetti container -->
  {#if showConfetti}
    <div class="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 pointer-events-none z-50">
      <Confetti 
        amount={200}
        x={[-1, 1]}
        y={[-0.5, 0.5]}
        duration={2500}
        size={12}
        colorArray={['#3b82f6', '#8b5cf6', '#ec4899', '#10b981', '#f59e0b']}
        cone
      />
    </div>
  {/if}

  {#if message}
    <div class="animate-in fade-in zoom-in duration-300">
      <Alert.Root variant={messageType === 'error' ? 'destructive' : 'default'} class="glass border-white/10 text-white">
        {#if messageType === 'error'}
          <AlertCircle class="h-4 w-4 text-red-400" />
        {:else}
          <CheckCircle2 class="h-4 w-4 text-green-400" />
        {/if}
        <Alert.Title>{messageType === 'error' ? 'Error' : 'Success'}</Alert.Title>
        <Alert.Description>{message}</Alert.Description>
      </Alert.Root>
    </div>
  {/if}

  <Card.Root class="glass border-white/10 overflow-hidden shadow-2xl transition-all duration-500">
    <div class="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 opacity-50"></div>
    
    <Card.Content class="p-5 space-y-5">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class={`p-2.5 rounded-xl transition-all duration-500 ${serverRunning ? 'bg-blue-500/20 text-blue-400' : 'bg-slate-500/20 text-slate-400'}`}>
            <Power class={`w-5 h-5 ${serverRunning ? 'animate-pulse' : ''}`} />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h3 class="text-lg font-bold text-white tracking-tight">Command Server</h3>
              <Badge variant={serverRunning ? "default" : "secondary"} class={`text-[10px] h-5 px-1.5 font-bold ${serverRunning ? "bg-blue-600 text-white" : "bg-slate-800 text-slate-300 border-white/10"}`}>
                {serverRunning ? 'ONLINE' : 'OFFLINE'}
              </Badge>
            </div>
            <p class="text-slate-400 text-xs mt-0.5 font-medium">
              {serverRunning ? 'Engine active & listening' : 'System standby'}
            </p>
          </div>
        </div>

        <div class="flex items-center gap-3 bg-white/5 pl-4 pr-2 py-1.5 rounded-xl border border-white/10">
          <span class="text-[11px] font-black text-slate-300 uppercase tracking-wider">{serverRunning ? 'Shutdown' : 'Boot'}</span>
          <Switch checked={serverRunning} onCheckedChange={toggleServer} disabled={loading} class="data-[state=checked]:bg-blue-500 scale-90" />
        </div>
      </div>

      {#if serverRunning}
        <div class="grid grid-cols-1 md:grid-cols-2 gap-3 animate-in fade-in slide-in-from-top-2 duration-500">
          <div class="p-3 rounded-xl bg-blue-500/10 border border-blue-500/20 flex items-center gap-3">
            <Globe class="w-4 h-4 text-blue-400" />
            <div>
              <p class="text-[10px] text-blue-300 font-black uppercase tracking-wider">Local IP</p>
              <p class="text-white text-xs font-mono font-bold">{getServerUrl()}</p>
            </div>
          </div>
          <div class="p-3 rounded-xl bg-purple-500/10 border border-purple-500/20 flex items-center gap-3">
            <Network class="w-4 h-4 text-purple-400" />
            <div>
              <p class="text-[10px] text-purple-300 font-black uppercase tracking-wider">Access</p>
              <p class="text-white text-xs font-bold">Public 0.0.0.0</p>
            </div>
          </div>
        </div>
      {/if}

      <div class="flex items-center justify-between pt-1 border-t border-white/10">
        <div class="flex gap-4">
          <span class="text-[10px] text-slate-400 font-bold uppercase tracking-tight">Auto-restart: <span class="text-blue-400">ON</span></span>
          <span class="text-[10px] text-slate-400 font-bold uppercase tracking-tight">Uptime: <span class={serverRunning ? "text-green-400" : "text-slate-500"}>{serverRunning ? 'Active' : 'N/A'}</span></span>
        </div>
        {#if !serverRunning}
          <div class="flex items-center gap-1.5 text-[10px] text-slate-400 font-medium italic">
            <AlertCircle class="w-3 h-3" />
            PC accessible on port {port}
          </div>
        {/if}
      </div>
    </Card.Content>
  </Card.Root>
</div>

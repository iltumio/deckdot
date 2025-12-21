<script>
  import { invoke } from '@tauri-apps/api/core';
  import { onMount, onDestroy } from 'svelte';
  import * as Card from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import * as Alert from "$lib/components/ui/alert";
  import { Power, Network, Globe, AlertCircle, CheckCircle2, Copy, Check, Share2, ExternalLink } from "lucide-svelte";

  let serverRunning = false;
  let loading = false;
  let message = '';
  let messageType = '';
  let port = 8080;
  let authCode = '';
  let localIps = [];
  let statusCheckInterval;
  let copiedIp = null;
  let copiedShareLink = false;

  onMount(async () => {
    await refresh();
    
    // Poll server status and settings every 2 seconds
    statusCheckInterval = setInterval(refresh, 2000);
  });

  async function refresh() {
    await checkStatus();
    await loadSettings();
    await loadLocalIps();
  }

  onDestroy(() => {
    if (statusCheckInterval) {
      clearInterval(statusCheckInterval);
    }
  });

  async function loadSettings() {
    try {
      const settings = await invoke('get_settings');
      port = settings.port;
      authCode = settings.auth_code;
    } catch (error) {
      console.error('Failed to load settings:', error);
    }
  }

  async function loadLocalIps() {
    try {
      localIps = await invoke('get_local_ips');
    } catch (error) {
      console.error('Failed to get local IPs:', error);
      localIps = ['localhost'];
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

  function getServerUrl(ip) {
    return `http://${ip || 'localhost'}:${port}`;
  }

  function getShareUrl(ip) {
    return `http://${ip || 'localhost'}:${port}?code=${encodeURIComponent(authCode)}`;
  }

  async function copyToClipboard(url) {
    try {
      await navigator.clipboard.writeText(url);
      copiedIp = url;
      setTimeout(() => { copiedIp = null; }, 2000);
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  }

  async function copyShareLink() {
    const ip = localIps.length > 0 ? localIps[0] : 'localhost';
    const shareUrl = getShareUrl(ip);
    try {
      await navigator.clipboard.writeText(shareUrl);
      copiedShareLink = true;
      showMessage('Share link copied to clipboard!', 'success');
      setTimeout(() => { copiedShareLink = false; }, 2000);
    } catch (error) {
      showMessage('Failed to copy share link', 'error');
    }
  }
</script>

<div class="space-y-6">
  {#if message}
    <div class="fixed top-4 right-4 z-50 max-w-sm animate-in fade-in slide-in-from-top-4 duration-300">
      <Alert.Root variant={messageType === 'error' ? 'destructive' : 'default'} class="glass border-white/10 text-white shadow-2xl">
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

        <Button 
          onclick={toggleServer}
          disabled={loading}
          variant={serverRunning ? "destructive" : "default"}
          class={`px-6 h-10 rounded-xl font-bold flex items-center gap-2 ${serverRunning ? 'bg-red-600 hover:bg-red-700' : 'bg-blue-600 hover:bg-blue-700'}`}
        >
          <Power class="w-4 h-4" />
          {#if loading}
            {serverRunning ? 'Stopping...' : 'Starting...'}
          {:else}
            {serverRunning ? 'Stop Server' : 'Start Server'}
          {/if}
        </Button>
      </div>

      {#if serverRunning}
        <div class="space-y-3 animate-in fade-in slide-in-from-top-2 duration-500">
          <!-- Share Button - Main Action -->
          <Button
            onclick={copyShareLink}
            class="w-full h-14 rounded-xl font-bold text-base bg-gradient-to-r from-purple-600 to-blue-600 hover:from-purple-700 hover:to-blue-700 shadow-lg shadow-purple-600/20 flex items-center justify-center gap-3"
          >
            {#if copiedShareLink}
              <Check class="w-5 h-5" />
              <span>Link Copied!</span>
            {:else}
              <Share2 class="w-5 h-5" />
              <span>Share Remote Access Link</span>
            {/if}
          </Button>
          
          <p class="text-center text-xs text-slate-500 font-medium">
            Copy link with embedded access code — just open on your phone
          </p>

          <div class="pt-2 border-t border-white/10">
            <h4 class="text-[10px] font-black text-slate-400 uppercase tracking-widest mb-3">Access URLs</h4>
            <div class="grid grid-cols-1 gap-2">
              {#each localIps as ip}
                {@const url = getServerUrl(ip)}
                <button
                  onclick={() => copyToClipboard(url)}
                  class="p-3 rounded-xl bg-blue-500/10 border border-blue-500/20 flex items-center gap-3 hover:bg-blue-500/20 transition-all group cursor-pointer text-left w-full"
                >
                  <Globe class="w-4 h-4 text-blue-400 shrink-0" />
                  <div class="flex-1 min-w-0">
                    <p class="text-[10px] text-blue-300 font-black uppercase tracking-wider">Network IP</p>
                    <p class="text-white text-xs font-mono font-bold truncate">{url}</p>
                  </div>
                  {#if copiedIp === url}
                    <Check class="w-4 h-4 text-green-400 shrink-0" />
                  {:else}
                    <Copy class="w-4 h-4 text-slate-500 group-hover:text-white transition-colors shrink-0" />
                  {/if}
                </button>
              {/each}
              
              {#if localIps.length === 0}
                <div class="p-3 rounded-xl bg-blue-500/10 border border-blue-500/20 flex items-center gap-3">
                  <Globe class="w-4 h-4 text-blue-400" />
                  <div>
                    <p class="text-[10px] text-blue-300 font-black uppercase tracking-wider">Local</p>
                    <p class="text-white text-xs font-mono font-bold">{getServerUrl('localhost')}</p>
                  </div>
                </div>
              {/if}
            </div>
          </div>
          
          <div class="p-3 rounded-xl bg-purple-500/10 border border-purple-500/20 flex items-center gap-3">
            <Network class="w-4 h-4 text-purple-400" />
            <div>
              <p class="text-[10px] text-purple-300 font-black uppercase tracking-wider">Binding</p>
              <p class="text-white text-xs font-bold">0.0.0.0:{port} (all interfaces)</p>
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

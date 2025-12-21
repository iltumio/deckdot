<script>
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import * as Card from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Alert from "$lib/components/ui/alert";
  import { Settings as SettingsIcon, Shield, Hash, Lock, Save, CheckCircle2, AlertCircle, Network, RefreshCw, Copy, Check, Eye, EyeOff } from "lucide-svelte";

  let port = $state(null);
  let authCode = $state('');
  let loading = $state(false);
  let settingsLoaded = $state(false);
  let regenerating = $state(false);
  let message = $state('');
  let messageType = $state('');
  let copied = $state(false);
  let showCode = $state(false);

  onMount(async () => {
    try {
      const settings = await invoke('get_settings');
      port = settings.port;
      authCode = settings.auth_code;
      settingsLoaded = true;
    } catch (error) {
      showMessage('Failed to load settings: ' + error, 'error');
    }
  });

  async function saveSettings() {
    loading = true;
    message = '';
    
    try {
      await invoke('save_settings', {
        settings: {
          port: parseInt(port.toString()),
          auth_code: authCode
        }
      });
      showMessage('Configuration updated successfully', 'success');
    } catch (error) {
      showMessage('Save failed: ' + error, 'error');
    } finally {
      loading = false;
    }
  }

  async function regenerateCode() {
    regenerating = true;
    try {
      const newCode = await invoke('regenerate_auth_code');
      authCode = newCode;
      showMessage('New access code generated', 'success');
    } catch (error) {
      showMessage('Failed to regenerate code: ' + error, 'error');
    } finally {
      regenerating = false;
    }
  }

  async function copyCode() {
    try {
      await navigator.clipboard.writeText(authCode);
      copied = true;
      setTimeout(() => { copied = false; }, 2000);
    } catch (error) {
      showMessage('Failed to copy', 'error');
    }
  }

  function showMessage(msg, type) {
    message = msg;
    messageType = type;
    setTimeout(() => {
      message = '';
    }, 3000);
  }

  function getMaskedCode(code) {
    if (!code) return '';
    return '•'.repeat(code.length);
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
        <Alert.Title>{messageType === 'error' ? 'Update Error' : 'Config Saved'}</Alert.Title>
        <Alert.Description>{message}</Alert.Description>
      </Alert.Root>
    </div>
  {/if}

  <Card.Root class="glass border-white/10 overflow-hidden shadow-2xl">
    <div class="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-purple-500 via-blue-500 to-cyan-500 opacity-50"></div>
    <Card.Header>
      <Card.Title class="text-2xl font-bold flex items-center gap-2 text-white">
        <SettingsIcon class="w-6 h-6 text-purple-400" />
        Configuration
      </Card.Title>
      <Card.Description class="text-slate-400">Security and network preferences</Card.Description>
    </Card.Header>
    
    <Card.Content class="space-y-8 pt-4">
      <div class="space-y-4">
        <h3 class="text-[11px] font-black text-blue-400 uppercase tracking-[0.2em] flex items-center gap-2">
          <Network class="w-4 h-4" />
          Network Settings
        </h3>
        <div class="grid gap-2">
          <Label for="port" class="text-slate-300 font-bold">Server Port</Label>
          <div class="relative">
            <Hash class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-500" />
            <Input
              id="port"
              type="number"
              bind:value={port}
              min="1"
              max="65535"
              disabled={!settingsLoaded}
              class="bg-white/5 border-white/10 text-white pl-10 h-12 rounded-xl focus:ring-blue-500 focus:border-blue-500 font-bold"
              placeholder={settingsLoaded ? '' : 'Loading...'}
            />
          </div>
          <p class="text-xs text-slate-500 font-medium">The port used for both management UI and phone access.</p>
        </div>
      </div>

      <div class="space-y-4">
        <h3 class="text-[11px] font-black text-purple-400 uppercase tracking-[0.2em] flex items-center gap-2">
          <Shield class="w-4 h-4" />
          Access Code
        </h3>
        
        <div class="p-4 rounded-xl bg-gradient-to-br from-purple-500/10 to-blue-500/10 border border-purple-500/20">
          <div class="flex items-center justify-between mb-3">
            <Label class="text-slate-300 font-bold text-sm">Current Code</Label>
            <div class="flex items-center gap-1">
              <Button
                variant="ghost"
                size="sm"
                onclick={() => showCode = !showCode}
                class="h-8 w-8 p-0 text-slate-400 hover:text-white hover:bg-white/10"
              >
                {#if showCode}
                  <EyeOff class="w-4 h-4" />
                {:else}
                  <Eye class="w-4 h-4" />
                {/if}
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onclick={copyCode}
                class="h-8 w-8 p-0 text-slate-400 hover:text-white hover:bg-white/10"
              >
                {#if copied}
                  <Check class="w-4 h-4 text-green-400" />
                {:else}
                  <Copy class="w-4 h-4" />
                {/if}
              </Button>
            </div>
          </div>
          
          <div class="flex items-center gap-3">
            <div class="flex-1 bg-black/30 rounded-xl p-4 border border-white/10">
              <code class="text-2xl font-mono font-bold tracking-[0.3em] text-white select-all">
                {showCode ? authCode : getMaskedCode(authCode)}
              </code>
            </div>
            <Button
              onclick={regenerateCode}
              disabled={regenerating}
              variant="outline"
              class="h-14 px-4 border-purple-500/30 bg-purple-500/10 hover:bg-purple-500/20 text-purple-300 hover:text-white rounded-xl"
            >
              <RefreshCw class={`w-5 h-5 ${regenerating ? 'animate-spin' : ''}`} />
            </Button>
          </div>
          <p class="text-xs text-slate-500 font-medium mt-3">
            This code is required to access the remote control from your phone.
          </p>
        </div>

        <div class="grid gap-2">
          <Label for="customCode" class="text-slate-300 font-bold">Set Custom Code</Label>
          <div class="relative">
            <Lock class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-500" />
            <Input
              id="customCode"
              type="text"
              bind:value={authCode}
              class="bg-white/5 border-white/10 text-white pl-10 h-12 rounded-xl focus:ring-blue-500 focus:border-blue-500 font-bold font-mono uppercase tracking-widest"
              placeholder="Enter custom code"
              maxlength="20"
            />
          </div>
          <p class="text-xs text-slate-500 font-medium">Leave as-is to use the auto-generated code, or enter your own.</p>
        </div>
      </div>
    </Card.Content>

    <Card.Footer class="bg-white/5 border-t border-white/10 mt-6 py-6 flex justify-end">
      <Button 
        onclick={saveSettings} 
        disabled={loading}
        class="bg-blue-600 hover:bg-blue-700 text-white px-8 h-11 rounded-xl shadow-lg shadow-blue-600/20 flex items-center gap-2"
      >
        <Save class="w-4 h-4" />
        {loading ? 'Applying...' : 'Apply Changes'}
      </Button>
    </Card.Footer>
  </Card.Root>

  <div class="p-6 glass rounded-2xl border-white/10 bg-purple-600/5">
    <div class="flex items-start gap-4">
      <Shield class="w-6 h-6 text-purple-400 mt-1" />
      <div class="space-y-2">
        <h4 class="font-bold text-white">How It Works</h4>
        <p class="text-sm text-slate-400 leading-relaxed font-medium">
          The access code is used to authenticate remote connections. Share it only with 
          devices you trust. You can regenerate it anytime to revoke previous access.
        </p>
      </div>
    </div>
  </div>
</div>

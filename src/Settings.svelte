<script>
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import * as Card from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Alert from "$lib/components/ui/alert";
  import { Settings as SettingsIcon, Shield, Hash, User, Lock, Save, CheckCircle2, AlertCircle, Network } from "lucide-svelte";

  let port = 8080;
  let username = 'admin';
  let password = '';
  let loading = false;
  let message = '';
  let messageType = '';

  onMount(async () => {
    try {
      const settings = await invoke('get_settings');
      port = settings.port;
      username = settings.username;
      password = settings.password;
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
          username,
          password
        }
      });
      showMessage('Configuration updated successfully', 'success');
    } catch (error) {
      showMessage('Save failed: ' + error, 'error');
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
</script>

<div class="space-y-6">
  {#if message}
    <div class="animate-in fade-in zoom-in duration-300">
      <Alert.Root variant={messageType === 'error' ? 'destructive' : 'default'} class="glass border-white/10 text-white">
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
              class="bg-white/5 border-white/10 text-white pl-10 h-12 rounded-xl focus:ring-blue-500 focus:border-blue-500 font-bold"
              placeholder="8080"
            />
          </div>
          <p class="text-xs text-slate-500 font-medium">The port used for both management UI and phone access.</p>
        </div>
      </div>

      <div class="space-y-4">
        <h3 class="text-[11px] font-black text-purple-400 uppercase tracking-[0.2em] flex items-center gap-2">
          <Shield class="w-4 h-4" />
          Authentication
        </h3>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div class="grid gap-2">
            <Label for="username" class="text-slate-300 font-bold">Username</Label>
            <div class="relative">
              <User class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-500" />
              <Input
                id="username"
                type="text"
                bind:value={username}
                class="bg-white/5 border-white/10 text-white pl-10 h-12 rounded-xl focus:ring-blue-500 focus:border-blue-500 font-bold"
                placeholder="admin"
              />
            </div>
          </div>
          <div class="grid gap-2">
            <Label for="password" class="text-slate-300 font-bold">Password</Label>
            <div class="relative">
              <Lock class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-500" />
              <Input
                id="password"
                type="password"
                bind:value={password}
                class="bg-white/5 border-white/10 text-white pl-10 h-12 rounded-xl focus:ring-blue-500 focus:border-blue-500 font-bold"
                placeholder="Secure password"
              />
            </div>
          </div>
        </div>
        <p class="text-xs text-slate-500 font-medium">Used for Basic Auth when connecting from a remote device.</p>
      </div>
    </Card.Content>

    <Card.Footer class="bg-white/5 border-t border-white/10 mt-6 py-6 flex justify-end">
      <Button 
        on:click={saveSettings} 
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
        <h4 class="font-bold text-white">Security Tip</h4>
        <p class="text-sm text-slate-400 leading-relaxed font-medium">
          Always use a strong password when exposing the server to your local network. 
          The server uses industry-standard Basic Authentication for all incoming command requests.
        </p>
      </div>
    </div>
  </div>
</div>

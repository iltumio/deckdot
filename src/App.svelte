<script>
  import { mount } from 'svelte';
  import Settings from './Settings.svelte';
  import Commands from './Commands.svelte';
  import ServerControl from './ServerControl.svelte';
  import * as Tabs from "$lib/components/ui/tabs";
  import { Settings as SettingsIcon, Terminal, Activity, Monitor } from "lucide-svelte";

  let activeTab = 'server';
</script>

<main class="min-h-screen p-4 md:p-8 flex flex-col items-center">
  <div class="w-full max-w-4xl">
    <header class="mb-8 flex flex-col md:flex-row items-center justify-between gap-4 glass p-4 rounded-2xl border-white/10 bg-white/5">
      <div class="flex items-center gap-4">
        <div class="inline-flex items-center justify-center p-2.5 rounded-xl bg-blue-500/10 border border-blue-500/20">
          <Monitor class="w-6 h-6 text-blue-400" />
        </div>
        <div class="text-left">
          <h1 class="text-2xl font-bold tracking-tight text-white">
            DECK<span class="text-blue-400">.</span>
          </h1>
          <p class="text-slate-400 text-xs font-bold uppercase tracking-widest">Remote PC Controller</p>
        </div>
      </div>
      
      <div class="hidden md:flex items-center gap-2 px-3 py-1 rounded-full bg-white/5 border border-white/10">
        <div class="w-2 h-2 rounded-full bg-green-500 animate-pulse"></div>
        <span class="text-[10px] font-black text-slate-300 uppercase tracking-widest">System Ready</span>
      </div>
    </header>

    <Tabs.Root bind:value={activeTab} class="w-full">
      <Tabs.List class="w-full glass mb-8 grid grid-cols-3 p-1.5 rounded-xl bg-white/5 border-white/10 h-auto">
        <Tabs.Trigger value="server" class="rounded-lg data-[state=active]:bg-gradient-to-r data-[state=active]:from-blue-600 data-[state=active]:to-blue-500 data-[state=active]:text-white data-[state=active]:shadow-lg data-[state=inactive]:text-slate-400 data-[state=inactive]:hover:text-white data-[state=inactive]:hover:bg-white/5 transition-all duration-300 flex items-center justify-center gap-2 py-3 h-full font-bold">
          <Activity class="w-4 h-4" />
          <span class="hidden md:inline">Server</span>
        </Tabs.Trigger>
        <Tabs.Trigger value="settings" class="rounded-lg data-[state=active]:bg-gradient-to-r data-[state=active]:from-purple-600 data-[state=active]:to-purple-500 data-[state=active]:text-white data-[state=active]:shadow-lg data-[state=inactive]:text-slate-400 data-[state=inactive]:hover:text-white data-[state=inactive]:hover:bg-white/5 transition-all duration-300 flex items-center justify-center gap-2 py-3 h-full font-bold">
          <SettingsIcon class="w-4 h-4" />
          <span class="hidden md:inline">Settings</span>
        </Tabs.Trigger>
        <Tabs.Trigger value="commands" class="rounded-lg data-[state=active]:bg-gradient-to-r data-[state=active]:from-cyan-600 data-[state=active]:to-cyan-500 data-[state=active]:text-white data-[state=active]:shadow-lg data-[state=inactive]:text-slate-400 data-[state=inactive]:hover:text-white data-[state=inactive]:hover:bg-white/5 transition-all duration-300 flex items-center justify-center gap-2 py-3 h-full font-bold">
          <Terminal class="w-4 h-4" />
          <span class="hidden md:inline">Commands</span>
        </Tabs.Trigger>
      </Tabs.List>

      <Tabs.Content value="server" class="mt-0 focus-visible:outline-none focus-visible:ring-0">
        <div class="animate-in fade-in slide-in-from-bottom-4 duration-500">
          <ServerControl />
        </div>
      </Tabs.Content>

      <Tabs.Content value="settings" class="mt-0 focus-visible:outline-none focus-visible:ring-0">
        <div class="animate-in fade-in slide-in-from-bottom-4 duration-500">
          <Settings />
        </div>
      </Tabs.Content>

      <Tabs.Content value="commands" class="mt-0 focus-visible:outline-none focus-visible:ring-0">
        <div class="animate-in fade-in slide-in-from-bottom-4 duration-500">
          <Commands />
        </div>
      </Tabs.Content>
    </Tabs.Root>
  </div>

  <footer class="mt-auto py-8 text-slate-500 text-sm">
    &copy; 2024 DECK Controller &bull; Built with Tauri & Svelte
  </footer>
</main>

<style>
  :global(body) {
    overflow-x: hidden;
  }
</style>

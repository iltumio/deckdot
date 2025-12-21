<script>
  import { onMount } from "svelte";
  import * as Card from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import {
    Lock,
    Zap,
    CheckCircle,
    XCircle,
    Loader2,
    Monitor,
    Grid3x3,
    RefreshCw,
  } from "lucide-svelte";

  let authCode = $state("");
  let executing = $state(null);
  let statusMessage = $state(null);
  let authValid = $state(null);
  let commands = $state([]);
  let loading = $state(true);
  let isError = $state(false);
  let isFetching = $state(false);

  // Column selection - null means use responsive defaults
  let selectedColumns = $state(null);
  const columnOptions = [2, 4, 6, 8];

  // Manual fetch function for commands
  async function fetchCommands() {
    console.log("Fetching commands with code:", authCode);
    isFetching = true;
    loading = commands.length === 0;
    isError = false;
    
    try {
      const response = await fetch(
        "/api/commands?code=" + encodeURIComponent(authCode)
      );
      if (response.status === 401) {
        if (authCode) authValid = false;
        commands = [];
        return;
      }
      if (!response.ok) {
        throw new Error("Failed to fetch commands");
      }
      if (authCode) authValid = true;
      commands = await response.json();
    } catch (error) {
      console.error("Failed to fetch commands:", error);
      isError = true;
    } finally {
      loading = false;
      isFetching = false;
    }
  }

  // Execute command function
  async function doExecuteCommand(commandId) {
    try {
      const response = await fetch(
        "/execute?code=" + encodeURIComponent(authCode),
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ id: commandId }),
        }
      );

      if (response.status === 401) {
        authValid = false;
        throw new Error("Invalid access code");
      }

      authValid = true;
      const data = await response.json();

      if (data.success) {
        showStatus("Executed!", "success");
      } else {
        showStatus(data.message || "Failed", "error");
      }
    } catch (error) {
      showStatus(error.message || "Connection error", "error");
    } finally {
      executing = null;
    }
  }

  onMount(async () => {
    // Check for code in URL
    const urlParams = new URLSearchParams(window.location.search);
    const codeFromUrl = urlParams.get("code");

    if (codeFromUrl) {
      authCode = codeFromUrl;
      localStorage.setItem("deck_code", codeFromUrl);
    } else {
      authCode = localStorage.getItem("deck_code") || "";
    }

    // Load column preference
    const savedColumns = localStorage.getItem("deck_columns");
    if (savedColumns) {
      const parsed = parseInt(savedColumns);
      if (columnOptions.includes(parsed)) {
        selectedColumns = parsed;
      }
    }
    
    // Fetch commands
    await fetchCommands();
  });

  function setColumns(cols) {
    if (selectedColumns === cols) {
      // Toggle off - back to responsive defaults
      selectedColumns = null;
      localStorage.removeItem("deck_columns");
    } else {
      selectedColumns = cols;
      localStorage.setItem("deck_columns", cols.toString());
    }
  }

  // Compute grid class based on selection
  function getGridClass() {
    if (selectedColumns === null) {
      return "grid-cols-2 sm:grid-cols-4 lg:grid-cols-6";
    }
    switch (selectedColumns) {
      case 2:
        return "grid-cols-2";
      case 4:
        return "grid-cols-4";
      case 6:
        return "grid-cols-6";
      case 8:
        return "grid-cols-8";
      default:
        return "grid-cols-2 sm:grid-cols-4 lg:grid-cols-6";
    }
  }

  function handleCodeInput(e) {
    authCode = e.target.value.toUpperCase();
    localStorage.setItem("deck_code", authCode);
    authValid = null; // Reset validation on change
  }

  function executeCommand(id) {
    if (!authCode) {
      showStatus("Enter access code first", "error");
      return;
    }

    executing = id;
    doExecuteCommand(id);
  }

  function refreshCommands() {
    fetchCommands();
  }

  function showStatus(message, type) {
    statusMessage = { message, type };
    setTimeout(() => {
      statusMessage = null;
    }, 2500);
  }
</script>

<div class="min-h-dvh flex flex-col safe-top safe-bottom">
  <!-- Header -->
  <header class="px-5 pt-6 pb-4">
    <div class="flex items-center justify-center gap-3">
      <div class="p-2 rounded-xl bg-blue-500/15 border border-blue-500/20">
        <Monitor class="w-5 h-5 text-blue-400" />
      </div>
      <div class="text-center">
        <h1 class="text-2xl font-bold tracking-tight text-white">
          DECK<span class="text-blue-400">.</span>
        </h1>
        <p
          class="text-[10px] font-bold text-slate-500 uppercase tracking-widest"
        >
          Remote Controller
        </p>
      </div>
      <!-- Refresh button -->
      <button
        onclick={refreshCommands}
        disabled={isFetching}
        class="p-2 rounded-xl bg-white/5 border border-white/10 hover:bg-white/10 transition-colors disabled:opacity-50"
      >
        <RefreshCw
          class="w-4 h-4 text-slate-400 {isFetching ? 'animate-spin' : ''}"
        />
      </button>
    </div>
  </header>

  <!-- Auth Section -->
  <div class="px-5 mb-5">
    <Card.Root class="glass border-white/10">
      <Card.Content class="p-4">
        <div class="flex items-center gap-3 mb-3">
          <Lock class="w-4 h-4 text-indigo-400" />
          <span
            class="text-xs font-bold text-slate-400 uppercase tracking-wider"
            >Access Code</span
          >
          {#if authValid === true}
            <span
              class="ml-auto flex items-center gap-1 text-[10px] font-bold text-green-400 uppercase"
            >
              <CheckCircle class="w-3 h-3" /> Valid
            </span>
          {:else if authValid === false}
            <span
              class="ml-auto flex items-center gap-1 text-[10px] font-bold text-red-400 uppercase"
            >
              <XCircle class="w-3 h-3" /> Invalid
            </span>
          {/if}
        </div>
        <Input
          type="text"
          value={authCode}
          oninput={handleCodeInput}
          placeholder="Enter code"
          maxlength="20"
          autocomplete="off"
          autocapitalize="characters"
          class="bg-black/30 border-white/10 text-white text-center text-lg font-mono font-bold tracking-[0.25em] uppercase h-12 rounded-xl"
        />
      </Card.Content>
    </Card.Root>
  </div>

  <!-- Column Selector -->
  <div class="px-5 mb-4">
    <div class="flex items-center justify-center gap-2">
      <Grid3x3 class="w-4 h-4 text-slate-500" />
      <span
        class="text-xs font-bold text-slate-500 uppercase tracking-wider mr-2"
        >Columns</span
      >
      {#each columnOptions as cols}
        <button
          onclick={() => setColumns(cols)}
          class="w-9 h-9 rounded-lg text-sm font-bold transition-all {selectedColumns ===
          cols
            ? 'bg-blue-600 text-white shadow-lg shadow-blue-500/30'
            : 'bg-white/5 text-slate-400 hover:bg-white/10 hover:text-white border border-white/10'}"
        >
          {cols}
        </button>
      {/each}
    </div>
    {#if selectedColumns === null}
      <p class="text-center text-[10px] text-slate-600 mt-2">
        Auto: 2 mobile · 4 tablet · 6 desktop
      </p>
    {/if}
  </div>

  <!-- Commands Grid -->
  <div class="flex-1 px-5 pb-6 overflow-y-auto">
    {#if loading}
      <div class="flex items-center justify-center py-16">
        <Loader2 class="w-8 h-8 text-blue-400 animate-spin" />
      </div>
    {:else if isError}
      <div class="text-center py-16">
        <div
          class="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-red-800/30 border border-red-500/20 mb-4"
        >
          <XCircle class="w-8 h-8 text-red-400" />
        </div>
        <p class="text-slate-400 font-medium">Failed to load commands</p>
        <p class="text-slate-600 text-sm mt-1">
          {authCode
            ? "Check your connection or access code"
            : "Enter access code to view"}
        </p>
        <button
          onclick={refreshCommands}
          class="mt-4 px-4 py-2 rounded-xl bg-blue-600 text-white font-bold text-sm hover:bg-blue-700 transition-colors"
        >
          Try Again
        </button>
      </div>
    {:else if commands.length === 0}
      <div class="text-center py-16">
        <div
          class="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-slate-800/50 border border-white/10 mb-4"
        >
          <Zap class="w-8 h-8 text-slate-500" />
        </div>
        <p class="text-slate-400 font-medium">No commands available</p>
        <p class="text-slate-600 text-sm mt-1">
          {authCode ? "Check your access code" : "Enter access code to view"}
        </p>
      </div>
    {:else}
      <div class="grid {getGridClass()} gap-3">
        {#each commands as cmd (cmd.id)}
          <button
            onclick={() => executeCommand(cmd.id)}
            disabled={executing === cmd.id}
            class="aspect-square p-4 rounded-2xl bg-white/[0.03] border border-white/[0.06] flex flex-col items-center justify-center text-center transition-all btn-haptic hover:bg-blue-500/10 hover:border-blue-500/20 active:scale-95 disabled:opacity-60 disabled:scale-100 group relative overflow-hidden"
          >
            <!-- Background glow effect -->
            <div
              class="absolute inset-0 bg-gradient-to-br from-blue-500/0 to-cyan-500/0 group-hover:from-blue-500/10 group-hover:to-cyan-500/5 transition-all duration-300"
            ></div>

            <!-- Icon -->
            <div class="relative z-10 mb-3">
              {#if executing === cmd.id}
                <div
                  class="w-12 h-12 rounded-xl bg-blue-500/20 flex items-center justify-center"
                >
                  <Loader2 class="w-6 h-6 text-blue-400 animate-spin" />
                </div>
              {:else}
                <div
                  class="w-12 h-12 rounded-xl bg-white/5 flex items-center justify-center group-hover:bg-blue-500/20 transition-colors"
                >
                  <Zap
                    class="w-6 h-6 text-slate-400 group-hover:text-blue-400 transition-colors"
                  />
                </div>
              {/if}
            </div>

            <!-- Text -->
            <div class="relative z-10 w-full px-1">
              <p
                class="text-sm font-bold text-white truncate group-hover:text-blue-100 transition-colors"
              >
                {cmd.name}
              </p>
            </div>
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Status Toast -->
  {#if statusMessage}
    <div
      class="fixed bottom-6 left-5 right-5 safe-bottom animate-in fade-in slide-in-from-bottom-4 duration-200"
    >
      <div
        class={`p-4 rounded-2xl backdrop-blur-xl font-semibold text-center ${
          statusMessage.type === "success"
            ? "bg-green-500/90 text-white"
            : "bg-red-500/90 text-white"
        }`}
      >
        {statusMessage.message}
      </div>
    </div>
  {/if}
</div>

<style>
  .animate-in {
    animation: animate-in 0.2s ease-out;
  }

  @keyframes animate-in {
    from {
      opacity: 0;
      transform: translateY(1rem);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>

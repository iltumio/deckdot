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
  } from "lucide-svelte";

  let commands = $state([]);
  let authCode = $state("");
  let loading = $state(true);
  let executing = $state(null);
  let statusMessage = $state(null);
  let authValid = $state(null);

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

    await fetchCommands();
  });

  async function fetchCommands() {
    loading = true;
    try {
      const response = await fetch(
        "/api/commands?code=" + encodeURIComponent(authCode)
      );
      if (response.ok) {
        commands = await response.json();
        if (authCode) authValid = true;
      } else if (response.status === 401) {
        // Still fetch commands without auth for display
        commands = [];
        if (authCode) authValid = false;
      }
    } catch (error) {
      console.error("Failed to fetch commands:", error);
    } finally {
      loading = false;
    }
  }

  function handleCodeInput(e) {
    authCode = e.target.value.toUpperCase();
    localStorage.setItem("deck_code", authCode);
    authValid = null; // Reset validation on change
  }

  async function executeCommand(id) {
    if (!authCode) {
      showStatus("Enter access code first", "error");
      return;
    }

    executing = id;

    try {
      const response = await fetch(
        "/execute?code=" + encodeURIComponent(authCode),
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ id }),
        }
      );

      if (response.status === 401) {
        authValid = false;
        showStatus("Invalid access code", "error");
        return;
      }

      authValid = true;
      const data = await response.json();

      if (data.success) {
        showStatus("Executed!", "success");
      } else {
        showStatus(data.message || "Failed", "error");
      }
    } catch (error) {
      showStatus("Connection error", "error");
    } finally {
      executing = null;
    }
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

  <!-- Commands Grid -->
  <div class="flex-1 px-5 pb-6 overflow-y-auto">
    {#if loading}
      <div class="flex items-center justify-center py-16">
        <Loader2 class="w-8 h-8 text-blue-400 animate-spin" />
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
      <div class="grid gap-3">
        {#each commands as cmd (cmd.id)}
          <button
            onclick={() => executeCommand(cmd.id)}
            disabled={executing === cmd.id}
            class="w-full p-5 rounded-2xl bg-white/[0.03] border border-white/[0.06] text-left transition-all btn-haptic hover:bg-blue-500/10 hover:border-blue-500/20 disabled:opacity-60 disabled:scale-100 group"
          >
            <div class="flex items-center gap-4">
              <div class="flex-1 min-w-0">
                <p
                  class="text-lg font-semibold text-white truncate group-hover:text-blue-100"
                >
                  {cmd.name}
                </p>
                <p
                  class="text-xs font-mono text-slate-500 uppercase tracking-wide truncate"
                >
                  {cmd.id}
                </p>
              </div>
              <div class="shrink-0">
                {#if executing === cmd.id}
                  <div
                    class="w-10 h-10 rounded-xl bg-blue-500/20 flex items-center justify-center"
                  >
                    <Loader2 class="w-5 h-5 text-blue-400 animate-spin" />
                  </div>
                {:else}
                  <div
                    class="w-10 h-10 rounded-xl bg-white/5 flex items-center justify-center group-hover:bg-blue-500/20 transition-colors"
                  >
                    <Zap
                      class="w-5 h-5 text-slate-400 group-hover:text-blue-400 transition-colors"
                    />
                  </div>
                {/if}
              </div>
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

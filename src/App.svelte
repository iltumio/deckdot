<script>
  import Settings from "./Settings.svelte";
  import Commands from "./Commands.svelte";
  import ServerControl from "./ServerControl.svelte";
  import * as Tabs from "$lib/components/ui/tabs";
  import {
    Settings as SettingsIcon,
    Terminal,
    Activity,
    Monitor,
  } from "lucide-svelte";

  let activeTab = "server";
  let iconContainer;

  // Dev-only: Download icon component (tree-shaken in production)
  // @ts-ignore
  let DownloadIcon = null;
  // @ts-ignore
  if (import.meta.env.DEV) {
    import("lucide-svelte").then((m) => (DownloadIcon = m.Download));
  }

  // Dev-only: Icon download function (tree-shaken in production)
  // @ts-ignore
  const downloadIcon = import.meta.env.DEV
    ? async function () {
        if (!iconContainer) return;

        const size = 800;
        const svg = iconContainer.querySelector("svg");
        if (!svg) return;

        // Get computed styles
        const containerStyles = getComputedStyle(iconContainer);
        const bgColor = containerStyles.backgroundColor;
        const borderColor = containerStyles.borderColor;
        const borderRadius =
          (parseFloat(containerStyles.borderRadius) /
            parseFloat(containerStyles.width)) *
          size;
        const padding =
          (parseFloat(containerStyles.padding) /
            parseFloat(containerStyles.width)) *
          size;

        // Create canvas
        const canvas = document.createElement("canvas");
        canvas.width = size;
        canvas.height = size;
        const ctx = canvas.getContext("2d");

        // Calculate border width
        const borderWidth = 2 * (size / parseFloat(containerStyles.width));
        const inset = borderWidth / 2;

        // Draw rounded rectangle with fill and border in one path
        ctx.fillStyle = bgColor;
        ctx.strokeStyle = borderColor;
        ctx.lineWidth = borderWidth;
        ctx.beginPath();
        ctx.roundRect(
          inset,
          inset,
          size - borderWidth,
          size - borderWidth,
          borderRadius
        );
        ctx.fill();
        ctx.stroke();

        // Clone and prepare SVG
        const svgClone = svg.cloneNode(true);
        svgClone.setAttribute("width", size - padding * 2);
        svgClone.setAttribute("height", size - padding * 2);
        svgClone.style.color = "#60a5fa"; // text-blue-400

        // Convert SVG to data URL
        const svgData = new XMLSerializer().serializeToString(svgClone);
        const svgBlob = new Blob([svgData], {
          type: "image/svg+xml;charset=utf-8",
        });
        const svgUrl = URL.createObjectURL(svgBlob);

        // Load and draw SVG
        const img = new Image();
        img.onload = () => {
          ctx.drawImage(
            img,
            padding,
            padding,
            size - padding * 2,
            size - padding * 2
          );
          URL.revokeObjectURL(svgUrl);

          // Download
          const link = document.createElement("a");
          link.download = "icon.png";
          link.href = canvas.toDataURL("image/png");
          link.click();
        };
        img.src = svgUrl;
      }
    : null;
</script>

<main class="min-h-screen p-4 md:p-8 flex flex-col items-center">
  <div class="w-full max-w-4xl">
    <header
      class="mb-8 flex flex-col md:flex-row items-center justify-between gap-4 glass p-4 rounded-2xl border-white/10 bg-white/5"
    >
      <div class="flex items-center gap-4">
        <div class="relative">
          <div
            bind:this={iconContainer}
            class="inline-flex items-center justify-center p-2.5 rounded-xl bg-blue-500/10 border border-blue-500/20"
          >
            <Monitor class="w-6 h-6 text-blue-400" />
          </div>
          {#if DownloadIcon && downloadIcon}
            <button
              onclick={downloadIcon}
              class="absolute -bottom-1 -right-1 p-1 rounded-full bg-green-500 hover:bg-green-400 transition-colors shadow-lg"
              title="Download as 800x800 PNG"
            >
              <svelte:component
                this={DownloadIcon}
                class="w-3 h-3 text-white"
              />
            </button>
          {/if}
        </div>
        <div class="text-left">
          <h1 class="text-2xl font-bold tracking-tight text-white">
            DECK<span class="text-blue-400">.</span>
          </h1>
          <p class="text-slate-400 text-xs font-bold uppercase tracking-widest">
            Remote PC Controller
          </p>
        </div>
      </div>

      <div
        class="hidden md:flex items-center gap-2 px-3 py-1 rounded-full bg-white/5 border border-white/10"
      >
        <div class="w-2 h-2 rounded-full bg-green-500 animate-pulse"></div>
        <span
          class="text-[10px] font-black text-slate-300 uppercase tracking-widest"
          >System Ready</span
        >
      </div>
    </header>

    <Tabs.Root bind:value={activeTab} class="w-full">
      <Tabs.List
        class="w-full glass mb-8 grid grid-cols-3 p-1.5 rounded-xl bg-white/5 border-white/10 h-auto"
      >
        <Tabs.Trigger
          value="server"
          class="rounded-lg data-[state=active]:bg-gradient-to-r data-[state=active]:from-blue-600 data-[state=active]:to-blue-500 data-[state=active]:text-white data-[state=active]:shadow-lg data-[state=inactive]:text-slate-400 data-[state=inactive]:hover:text-white data-[state=inactive]:hover:bg-white/5 transition-all duration-300 flex items-center justify-center gap-2 py-3 h-full font-bold"
        >
          <Activity class="w-4 h-4" />
          <span class="hidden md:inline">Server</span>
        </Tabs.Trigger>
        <Tabs.Trigger
          value="settings"
          class="rounded-lg data-[state=active]:bg-gradient-to-r data-[state=active]:from-purple-600 data-[state=active]:to-purple-500 data-[state=active]:text-white data-[state=active]:shadow-lg data-[state=inactive]:text-slate-400 data-[state=inactive]:hover:text-white data-[state=inactive]:hover:bg-white/5 transition-all duration-300 flex items-center justify-center gap-2 py-3 h-full font-bold"
        >
          <SettingsIcon class="w-4 h-4" />
          <span class="hidden md:inline">Settings</span>
        </Tabs.Trigger>
        <Tabs.Trigger
          value="commands"
          class="rounded-lg data-[state=active]:bg-gradient-to-r data-[state=active]:from-cyan-600 data-[state=active]:to-cyan-500 data-[state=active]:text-white data-[state=active]:shadow-lg data-[state=inactive]:text-slate-400 data-[state=inactive]:hover:text-white data-[state=inactive]:hover:bg-white/5 transition-all duration-300 flex items-center justify-center gap-2 py-3 h-full font-bold"
        >
          <Terminal class="w-4 h-4" />
          <span class="hidden md:inline">Commands</span>
        </Tabs.Trigger>
      </Tabs.List>

      <Tabs.Content
        value="server"
        class="mt-0 focus-visible:outline-none focus-visible:ring-0"
      >
        <div class="animate-in fade-in slide-in-from-bottom-4 duration-500">
          <ServerControl />
        </div>
      </Tabs.Content>

      <Tabs.Content
        value="settings"
        class="mt-0 focus-visible:outline-none focus-visible:ring-0"
      >
        <div class="animate-in fade-in slide-in-from-bottom-4 duration-500">
          <Settings />
        </div>
      </Tabs.Content>

      <Tabs.Content
        value="commands"
        class="mt-0 focus-visible:outline-none focus-visible:ring-0"
      >
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

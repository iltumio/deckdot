<script>
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import * as Card from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Badge } from "$lib/components/ui/badge";
  import * as Alert from "$lib/components/ui/alert";
  import { 
    Terminal, Plus, Edit2, Trash2, Save, X, 
    Monitor, Command as CommandIcon, Tag, 
    AlertCircle, CheckCircle2, Search
  } from "lucide-svelte";

  let commands = [];
  let loading = false;
  let message = '';
  let messageType = '';
  let editingId = null;
  let showForm = false;
  let searchQuery = '';
  
  let formId = '';
  let formName = '';
  let formCommand = '';
  let formFocusApp = '';

  onMount(async () => {
    await loadCommands();
  });

  async function loadCommands() {
    try {
      commands = await invoke('get_commands');
    } catch (error) {
      showMessage('Failed to load commands: ' + error, 'error');
    }
  }

  function startAdd() {
    editingId = null;
    formId = '';
    formName = '';
    formCommand = '';
    formFocusApp = '';
    showForm = true;
  }

  function startEdit(cmd) {
    editingId = cmd.id;
    formId = cmd.id;
    formName = cmd.name;
    formCommand = cmd.command;
    formFocusApp = cmd.focus_app || '';
    showForm = true;
  }

  function cancelEdit() {
    showForm = false;
    editingId = null;
  }

  async function saveCommand() {
    if (!formId || !formName || !formCommand) {
      showMessage('ID, Name, and Command are required', 'error');
      return;
    }

    loading = true;
    
    try {
      const updatedCommands = [...commands];
      
      if (editingId) {
        const index = updatedCommands.findIndex(c => c.id === editingId);
        if (index !== -1) {
          updatedCommands[index] = {
            id: formId,
            name: formName,
            command: formCommand,
            focus_app: formFocusApp || null
          };
        }
      } else {
        if (updatedCommands.some(c => c.id === formId)) {
          showMessage('Command ID already exists', 'error');
          loading = false;
          return;
        }
        updatedCommands.push({
          id: formId,
          name: formName,
          command: formCommand,
          focus_app: formFocusApp || null
        });
      }

      await invoke('save_commands', { commandsVec: updatedCommands });
      await loadCommands();
      showMessage(editingId ? 'Command updated' : 'New command added', 'success');
      cancelEdit();
    } catch (error) {
      showMessage('Operation failed: ' + error, 'error');
    } finally {
      loading = false;
    }
  }

  async function deleteCommand(id) {
    if (!confirm('Are you sure you want to remove this command?')) {
      return;
    }

    loading = true;
    
    try {
      const updatedCommands = commands.filter(c => c.id !== id);
      await invoke('save_commands', { commandsVec: updatedCommands });
      await loadCommands();
      showMessage('Command removed', 'success');
    } catch (error) {
      showMessage('Delete failed: ' + error, 'error');
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

  $: filteredCommands = commands.filter(cmd => 
    cmd.name.toLowerCase().includes(searchQuery.toLowerCase()) || 
    cmd.id.toLowerCase().includes(searchQuery.toLowerCase())
  );
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
        <Alert.Title>{messageType === 'error' ? 'Error' : 'Success'}</Alert.Title>
        <Alert.Description>{message}</Alert.Description>
      </Alert.Root>
    </div>
  {/if}

  <div class="flex flex-col md:flex-row gap-4 items-center justify-between glass p-4 rounded-2xl border-white/10">
    <div class="relative w-full md:w-96">
      <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-blue-400" />
      <Input 
        bind:value={searchQuery}
        placeholder="Search commands..." 
        class="bg-white/5 border-white/10 text-white pl-10 h-11 rounded-xl focus:ring-blue-500 font-bold"
      />
    </div>
    <Button 
      on:click={startAdd} 
      disabled={showForm || loading}
      class="w-full md:w-auto bg-blue-600 hover:bg-blue-700 text-white h-11 px-6 rounded-xl flex items-center gap-2 font-bold shadow-lg"
    >
      <Plus class="w-4 h-4" />
      Add Command
    </Button>
  </div>

  {#if showForm}
    <Card.Root class="glass border-white/10 overflow-hidden shadow-2xl animate-in fade-in slide-in-from-top-4 duration-500">
      <div class="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-blue-500 via-cyan-500 to-green-500 opacity-50"></div>
      <Card.Header>
        <Card.Title class="text-xl font-bold flex items-center gap-2 text-white">
          <Edit2 class="w-5 h-5 text-blue-400" />
          {editingId ? 'Edit Command' : 'New Command'}
        </Card.Title>
      </Card.Header>
      
      <Card.Content class="space-y-4">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div class="grid gap-2">
            <Label for="form-id" class="text-slate-300 font-bold">Identifier (ID)</Label>
            <Input
              id="form-id"
              bind:value={formId}
              placeholder="e.g. volume_up"
              disabled={!!editingId}
              class="bg-white/5 border-white/10 text-white h-11 rounded-xl font-bold"
            />
          </div>
          <div class="grid gap-2">
            <Label for="form-name" class="text-slate-300 font-bold">Display Name</Label>
            <Input
              id="form-name"
              bind:value={formName}
              placeholder="e.g. Increase Volume"
              class="bg-white/5 border-white/10 text-white h-11 rounded-xl font-bold"
            />
          </div>
        </div>

        <div class="grid gap-2">
          <Label for="form-command" class="text-slate-300 font-bold">Shell Command</Label>
          <div class="relative">
            <Terminal class="absolute left-3 top-3 w-4 h-4 text-blue-400" />
            <textarea
              id="form-command"
              bind:value={formCommand}
              placeholder="powershell.exe -Command ..."
              class="w-full min-h-[100px] bg-white/5 border border-white/10 text-white pl-10 pt-2 rounded-xl focus:ring-2 focus:ring-blue-500 focus:outline-none font-mono text-sm font-bold"
            ></textarea>
          </div>
        </div>

        <div class="grid gap-2">
          <Label for="form-focus-app" class="text-slate-300 font-bold">Focus App Window (Optional)</Label>
          <Input
            id="form-focus-app"
            bind:value={formFocusApp}
            placeholder="e.g. Notepad"
            class="bg-white/5 border-white/10 text-white h-11 rounded-xl font-bold"
          />
        </div>
      </Card.Content>

      <Card.Footer class="flex justify-end gap-3 bg-white/5 border-t border-white/10 py-4 mt-4">
        <Button variant="ghost" on:click={cancelEdit} class="text-slate-400 hover:text-white hover:bg-white/5 h-11 px-6 rounded-xl font-bold">
          <X class="w-4 h-4 mr-2" />
          Cancel
        </Button>
        <Button on:click={saveCommand} disabled={loading} class="bg-blue-600 hover:bg-blue-700 text-white h-11 px-8 rounded-xl font-bold shadow-lg">
          <Save class="w-4 h-4 mr-2" />
          {loading ? 'Saving...' : 'Save Command'}
        </Button>
      </Card.Footer>
    </Card.Root>
  {/if}

  {#if commands.length === 0}
    <div class="p-20 text-center glass rounded-2xl border-white/10">
      <div class="p-6 inline-flex rounded-full bg-white/5 mb-4">
        <Terminal class="w-12 h-12 text-slate-500" />
      </div>
      <h3 class="text-xl font-bold text-white mb-2">No commands yet</h3>
      <p class="text-slate-400 mb-8 max-w-sm mx-auto font-medium">Configure your first remote command to start controlling your PC.</p>
      <Button on:click={startAdd} class="bg-blue-600 hover:bg-blue-700 text-white font-bold px-8 shadow-xl">
        Create Your First Command
      </Button>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      {#each filteredCommands as cmd (cmd.id)}
        <div class="group glass rounded-2xl border-white/10 p-5 hover:border-blue-500/50 transition-all duration-300 relative overflow-hidden">
          <div class="absolute -right-4 -top-4 p-8 rounded-full bg-blue-600/5 opacity-0 group-hover:opacity-100 transition-opacity duration-500">
            <CommandIcon class="w-12 h-12 text-blue-400/20" />
          </div>
          
          <div class="flex justify-between items-start mb-4 relative z-10">
            <div class="space-y-1">
              <h4 class="text-lg font-black text-white group-hover:text-blue-400 transition-colors">{cmd.name}</h4>
              <Badge variant="outline" class="text-[10px] uppercase tracking-widest border-white/20 text-slate-400 font-black px-2 bg-white/5">
                ID: {cmd.id}
              </Badge>
            </div>
            <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
              <Button size="icon" variant="ghost" on:click={() => startEdit(cmd)} class="h-8 w-8 text-slate-400 hover:text-white hover:bg-white/10">
                <Edit2 class="w-3.5 h-3.5" />
              </Button>
              <Button size="icon" variant="ghost" on:click={() => deleteCommand(cmd.id)} class="h-8 w-8 text-slate-400 hover:text-red-400 hover:bg-white/10">
                <Trash2 class="w-3.5 h-3.5" />
              </Button>
            </div>
          </div>

          <div class="space-y-3 relative z-10">
            <div class="bg-black/40 rounded-xl p-3 border border-white/10">
              <code class="text-xs text-blue-200 font-bold break-all line-clamp-2">{cmd.command}</code>
            </div>
            
            {#if cmd.focus_app}
              <div class="flex items-center gap-2 text-[10px] text-purple-300 font-black uppercase tracking-wider">
                <Monitor class="w-3 h-3" />
                Focus: <span class="text-white">{cmd.focus_app}</span>
              </div>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

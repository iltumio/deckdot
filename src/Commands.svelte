<script>
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import * as Card from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Badge } from "$lib/components/ui/badge";
  import { Select } from "$lib/components/ui/select";
  import * as Alert from "$lib/components/ui/alert";
  import { 
    Terminal, Plus, Edit2, Trash2, Save, X, 
    Monitor, Command as CommandIcon, Tag, 
    AlertCircle, CheckCircle2, Search,
    Volume2, FolderOpen, AppWindow, Keyboard, Circle
  } from "lucide-svelte";

  // Command type definitions
  const COMMAND_TYPES = [
    { value: 'shell', label: 'Shell Command', icon: Terminal, description: 'Execute any shell command' },
    { value: 'volume', label: 'Volume Control', icon: Volume2, description: 'Adjust system volume' },
    { value: 'open_directory', label: 'Open Directory', icon: FolderOpen, description: 'Open folder in file manager' },
    { value: 'focus_app', label: 'Focus Application', icon: AppWindow, description: 'Bring app to foreground' },
    { value: 'keybind', label: 'Send Keybind', icon: Keyboard, description: 'Simulate keyboard shortcut' },
  ];

  const VOLUME_DIRECTIONS = [
    { value: 'up', label: 'Volume Up' },
    { value: 'down', label: 'Volume Down' },
    { value: 'mute', label: 'Toggle Mute' },
  ];

  let commands = [];
  let runningApps = [];
  let loading = false;
  let message = '';
  let messageType = '';
  let editingId = null;
  let showForm = false;
  let searchQuery = '';
  
  // Form fields
  let formId = '';
  let formName = '';
  let formCommandType = 'shell';
  let formCommand = '';
  let formFocusApp = '';
  
  // Type-specific fields
  let formVolumeDirection = 'up';
  let formVolumeStep = 5;
  let formDirectoryPath = '';
  let formAppName = '';
  let formKeybind = '';
  
  // Keybind recording state
  let isRecordingKeybind = false;
  let keybindInputRef = null;
  
  // Delete confirmation state
  let confirmDeleteId = null;

  function startRecordingKeybind() {
    isRecordingKeybind = true;
    formKeybind = '';
    // Focus the input to capture keys
    setTimeout(() => keybindInputRef?.focus(), 50);
  }

  function handleKeybindKeydown(event) {
    if (!isRecordingKeybind) return;
    
    event.preventDefault();
    event.stopPropagation();
    
    const parts = [];
    
    // Add modifiers in order
    if (event.metaKey) parts.push('cmd');
    if (event.ctrlKey) parts.push('ctrl');
    if (event.altKey) parts.push('alt');
    if (event.shiftKey) parts.push('shift');
    
    // Get the actual key
    let key = event.key.toLowerCase();
    
    // Skip if only modifier was pressed
    if (['meta', 'control', 'alt', 'shift'].includes(key)) {
      return;
    }
    
    // Map special keys
    const keyMap = {
      ' ': 'space',
      'arrowup': 'up',
      'arrowdown': 'down',
      'arrowleft': 'left',
      'arrowright': 'right',
      'escape': 'escape',
      'enter': 'enter',
      'tab': 'tab',
      'backspace': 'backspace',
      'delete': 'delete',
    };
    
    key = keyMap[key] || key;
    parts.push(key);
    
    formKeybind = parts.join('+');
    isRecordingKeybind = false;
  }

  function stopRecordingKeybind() {
    isRecordingKeybind = false;
  }

  onMount(async () => {
    await loadCommands();
    await loadRunningApps();
  });

  async function loadCommands() {
    try {
      commands = await invoke('get_commands');
    } catch (error) {
      showMessage('Failed to load commands: ' + error, 'error');
    }
  }

  async function loadRunningApps() {
    try {
      runningApps = await invoke('get_running_applications');
    } catch (error) {
      console.error('Failed to load running apps:', error);
      runningApps = [];
    }
  }

  function resetForm() {
    formId = '';
    formName = '';
    formCommandType = 'shell';
    formCommand = '';
    formFocusApp = '';
    formVolumeDirection = 'up';
    formVolumeStep = 5;
    formDirectoryPath = '';
    formAppName = '';
    formKeybind = '';
  }

  function startAdd() {
    editingId = null;
    resetForm();
    showForm = true;
    loadRunningApps(); // Refresh running apps list
  }

  function startEdit(cmd) {
    editingId = cmd.id;
    formId = cmd.id;
    formName = cmd.name;
    formCommandType = cmd.command_type || 'shell';
    formCommand = cmd.command || '';
    formFocusApp = cmd.focus_app || '';
    formVolumeDirection = cmd.volume_direction || 'up';
    formVolumeStep = cmd.volume_step || 5;
    formDirectoryPath = cmd.directory_path || '';
    formAppName = cmd.app_name || '';
    formKeybind = cmd.keybind || '';
    showForm = true;
    loadRunningApps(); // Refresh running apps list
  }

  function cancelEdit() {
    showForm = false;
    editingId = null;
  }

  function buildCommandObject() {
    const base = {
      id: formId,
      name: formName,
      command_type: formCommandType,
    };

    switch (formCommandType) {
      case 'shell':
        return {
          ...base,
          command: formCommand,
          focus_app: formFocusApp || null,
        };
      case 'volume':
        return {
          ...base,
          volume_direction: formVolumeDirection,
          volume_step: parseInt(formVolumeStep) || 5,
        };
      case 'open_directory':
        return {
          ...base,
          directory_path: formDirectoryPath,
        };
      case 'focus_app':
        return {
          ...base,
          app_name: formAppName,
        };
      case 'keybind':
        return {
          ...base,
          keybind: formKeybind,
        };
      default:
        return base;
    }
  }

  function validateForm() {
    if (!formId || !formName) {
      return 'ID and Name are required';
    }

    switch (formCommandType) {
      case 'shell':
        if (!formCommand) return 'Shell command is required';
        break;
      case 'volume':
        if (!formVolumeDirection) return 'Volume direction is required';
        break;
      case 'open_directory':
        if (!formDirectoryPath) return 'Directory path is required';
        break;
      case 'focus_app':
        if (!formAppName) return 'Application name is required';
        break;
      case 'keybind':
        if (!formKeybind) return 'Keybind is required';
        break;
    }

    return null;
  }

  async function saveCommand() {
    const validationError = validateForm();
    if (validationError) {
      showMessage(validationError, 'error');
      return;
    }

    loading = true;
    
    try {
      const updatedCommands = [...commands];
      const newCommand = buildCommandObject();
      
      if (editingId) {
        const index = updatedCommands.findIndex(c => c.id === editingId);
        if (index !== -1) {
          updatedCommands[index] = newCommand;
        }
      } else {
        if (updatedCommands.some(c => c.id === formId)) {
          showMessage('Command ID already exists', 'error');
          loading = false;
          return;
        }
        updatedCommands.push(newCommand);
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

  function requestDeleteCommand(id) {
    confirmDeleteId = id;
  }

  function cancelDelete() {
    confirmDeleteId = null;
  }

  async function confirmDeleteCommand() {
    if (!confirmDeleteId) return;
    
    const id = confirmDeleteId;
    confirmDeleteId = null;
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

  function getCommandTypeInfo(type) {
    return COMMAND_TYPES.find(t => t.value === type) || COMMAND_TYPES[0];
  }

  function getCommandDescription(cmd) {
    const type = cmd.command_type || 'shell';
    switch (type) {
      case 'shell':
        return cmd.command;
      case 'volume':
        return `Volume ${cmd.volume_direction || 'up'} (${cmd.volume_step || 5}%)`;
      case 'open_directory':
        return cmd.directory_path;
      case 'focus_app':
        return `Focus: ${cmd.app_name}`;
      case 'keybind':
        return cmd.keybind;
      default:
        return cmd.command || '';
    }
  }

  $: filteredCommands = commands.filter(cmd => 
    cmd.name.toLowerCase().includes(searchQuery.toLowerCase()) || 
    cmd.id.toLowerCase().includes(searchQuery.toLowerCase())
  );

  $: currentTypeInfo = getCommandTypeInfo(formCommandType);
</script>

<div class="space-y-6">
  <!-- Delete Confirmation Modal -->
  {#if confirmDeleteId}
    {@const cmdToDelete = commands.find(c => c.id === confirmDeleteId)}
    <div class="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center animate-in fade-in duration-200">
      <div class="glass border border-white/10 rounded-2xl p-6 max-w-md mx-4 shadow-2xl animate-in zoom-in-95 duration-200">
        <div class="flex items-center gap-3 mb-4">
          <div class="p-2 rounded-full bg-red-500/20">
            <Trash2 class="w-5 h-5 text-red-400" />
          </div>
          <h3 class="text-lg font-bold text-white">Delete Command</h3>
        </div>
        <p class="text-slate-300 mb-6">
          Are you sure you want to delete <span class="font-bold text-white">"{cmdToDelete?.name}"</span>? This action cannot be undone.
        </p>
        <div class="flex justify-end gap-3">
          <Button variant="ghost" onclick={cancelDelete} class="text-slate-400 hover:text-white hover:bg-white/5 h-10 px-4 rounded-xl font-bold">
            Cancel
          </Button>
          <Button onclick={confirmDeleteCommand} class="bg-red-600 hover:bg-red-700 text-white h-10 px-6 rounded-xl font-bold">
            Delete
          </Button>
        </div>
      </div>
    </div>
  {/if}

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
      onclick={startAdd} 
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
        <!-- Basic Info -->
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

        <!-- Command Type Selection -->
        <div class="grid gap-2">
          <Label for="form-type" class="text-slate-300 font-bold">Command Type</Label>
          <Select
            id="form-type"
            bind:value={formCommandType}
            class="bg-white/5 border-white/10 text-white h-11 rounded-xl font-bold"
          >
            {#each COMMAND_TYPES as type}
              <option value={type.value}>{type.label}</option>
            {/each}
          </Select>
          <p class="text-xs text-slate-400">{currentTypeInfo.description}</p>
        </div>

        <!-- Type-specific fields -->
        {#if formCommandType === 'shell'}
          <div class="grid gap-2">
            <Label for="form-command" class="text-slate-300 font-bold">Shell Command</Label>
            <div class="relative">
              <Terminal class="absolute left-3 top-3 w-4 h-4 text-blue-400" />
              <textarea
                id="form-command"
                bind:value={formCommand}
                placeholder="powershell.exe -Command ... or /path/to/script"
                class="w-full min-h-[100px] bg-white/5 border border-white/10 text-white pl-10 pt-2 rounded-xl focus:ring-2 focus:ring-blue-500 focus:outline-none font-mono text-sm font-bold"
              ></textarea>
            </div>
          </div>

          <div class="grid gap-2">
            <Label for="form-focus-app" class="text-slate-300 font-bold">Focus App Window (Optional)</Label>
            <Input
              id="form-focus-app"
              bind:value={formFocusApp}
              placeholder="e.g. Notepad (Window title to focus after execution)"
              class="bg-white/5 border-white/10 text-white h-11 rounded-xl font-bold"
            />
          </div>
        {/if}

        {#if formCommandType === 'volume'}
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div class="grid gap-2">
              <Label for="form-volume-dir" class="text-slate-300 font-bold">Direction</Label>
              <Select
                id="form-volume-dir"
                bind:value={formVolumeDirection}
                class="bg-white/5 border-white/10 text-white h-11 rounded-xl font-bold"
              >
                {#each VOLUME_DIRECTIONS as dir}
                  <option value={dir.value}>{dir.label}</option>
                {/each}
              </Select>
            </div>
            <div class="grid gap-2">
              <Label for="form-volume-step" class="text-slate-300 font-bold">Step Amount (%)</Label>
              <Input
                id="form-volume-step"
                type="number"
                min="1"
                max="100"
                bind:value={formVolumeStep}
                placeholder="5"
                class="bg-white/5 border-white/10 text-white h-11 rounded-xl font-bold"
              />
            </div>
          </div>
        {/if}

        {#if formCommandType === 'open_directory'}
          <div class="grid gap-2">
            <Label for="form-dir-path" class="text-slate-300 font-bold">Directory Path</Label>
            <div class="relative">
              <FolderOpen class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-blue-400" />
              <Input
                id="form-dir-path"
                bind:value={formDirectoryPath}
                placeholder="~/Documents or /Users/name/Projects"
                class="bg-white/5 border-white/10 text-white h-11 rounded-xl font-bold pl-10"
              />
            </div>
            <p class="text-xs text-slate-400">Use ~ for home directory, e.g., ~/Downloads</p>
          </div>
        {/if}

        {#if formCommandType === 'focus_app'}
          <div class="grid gap-2">
            <Label for="form-app-name" class="text-slate-300 font-bold">Application Name</Label>
            <div class="relative">
              <AppWindow class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-blue-400" />
              <Input
                id="form-app-name"
                bind:value={formAppName}
                list="running-apps"
                placeholder="e.g. Safari, Visual Studio Code, Slack"
                class="bg-white/5 border-white/10 text-white h-11 rounded-xl font-bold pl-10"
              />
              <datalist id="running-apps">
                {#each runningApps as app}
                  <option value={app}></option>
                {/each}
              </datalist>
            </div>
            {#if runningApps.length > 0}
              <div class="flex flex-wrap gap-1.5 mt-2">
                <span class="text-xs text-slate-500">Running:</span>
                {#each runningApps.slice(0, 8) as app}
                  <button
                    type="button"
                    onclick={() => formAppName = app}
                    class="text-xs px-2 py-0.5 rounded-full bg-white/5 border border-white/10 text-slate-300 hover:bg-blue-600/20 hover:border-blue-500/50 transition-colors cursor-pointer"
                  >
                    {app}
                  </button>
                {/each}
                {#if runningApps.length > 8}
                  <span class="text-xs text-slate-500">+{runningApps.length - 8} more</span>
                {/if}
              </div>
            {/if}
          </div>
        {/if}

        {#if formCommandType === 'keybind'}
          <div class="grid gap-2">
            <Label for="form-keybind" class="text-slate-300 font-bold">Keyboard Shortcut</Label>
            <div class="flex gap-2">
              <div class="relative flex-1">
                <Keyboard class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-blue-400" />
                <Input
                  id="form-keybind"
                  bind:ref={keybindInputRef}
                  bind:value={formKeybind}
                  placeholder={isRecordingKeybind ? "Press any key combination..." : "e.g. cmd+shift+v"}
                  class="bg-white/5 border-white/10 text-white h-11 rounded-xl font-bold pl-10 {isRecordingKeybind ? 'ring-2 ring-red-500 border-red-500 animate-pulse' : ''}"
                  onkeydown={handleKeybindKeydown}
                  onblur={stopRecordingKeybind}
                  readonly={isRecordingKeybind}
                />
              </div>
              <Button 
                type="button"
                onclick={isRecordingKeybind ? stopRecordingKeybind : startRecordingKeybind}
                class="{isRecordingKeybind 
                  ? 'bg-red-600 hover:bg-red-700 text-white' 
                  : 'bg-white/10 hover:bg-white/20 text-white'} h-11 px-4 rounded-xl font-bold flex items-center gap-2"
              >
                <Circle class="w-3 h-3 {isRecordingKeybind ? 'fill-current animate-pulse' : ''}" />
                {isRecordingKeybind ? 'Recording...' : 'Record'}
              </Button>
            </div>
            <p class="text-xs text-slate-400">
              Click "Record" and press any key combination, or type manually using + to combine keys.
              <br />
              Examples: cmd+space, ctrl+shift+t, f5, escape
            </p>
          </div>
        {/if}
      </Card.Content>

      <Card.Footer class="flex justify-end gap-3 bg-white/5 border-t border-white/10 py-4 mt-4">
        <Button variant="ghost" onclick={cancelEdit} class="text-slate-400 hover:text-white hover:bg-white/5 h-11 px-6 rounded-xl font-bold">
          <X class="w-4 h-4 mr-2" />
          Cancel
        </Button>
        <Button onclick={saveCommand} disabled={loading} class="bg-blue-600 hover:bg-blue-700 text-white h-11 px-8 rounded-xl font-bold shadow-lg">
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
      <Button onclick={startAdd} class="bg-blue-600 hover:bg-blue-700 text-white font-bold px-8 shadow-xl">
        Create Your First Command
      </Button>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      {#each filteredCommands as cmd (cmd.id)}
        {@const typeInfo = getCommandTypeInfo(cmd.command_type || 'shell')}
        <div class="group glass rounded-2xl border-white/10 p-5 hover:border-blue-500/50 transition-all duration-300 relative overflow-hidden">
          <div class="absolute -right-4 -top-4 p-8 rounded-full bg-blue-600/5 opacity-0 group-hover:opacity-100 transition-opacity duration-500">
            <svelte:component this={typeInfo.icon} class="w-12 h-12 text-blue-400/20" />
          </div>
          
          <div class="flex justify-between items-start mb-4 relative z-10">
            <div class="space-y-1">
              <h4 class="text-lg font-black text-white group-hover:text-blue-400 transition-colors">{cmd.name}</h4>
              <div class="flex items-center gap-2">
                <Badge variant="outline" class="text-[10px] uppercase tracking-widest border-white/20 text-slate-400 font-black px-2 bg-white/5">
                  ID: {cmd.id}
                </Badge>
                <Badge variant="outline" class="text-[10px] uppercase tracking-widest border-blue-500/30 text-blue-400 font-black px-2 bg-blue-500/10">
                  <svelte:component this={typeInfo.icon} class="w-2.5 h-2.5 mr-1" />
                  {typeInfo.label}
                </Badge>
              </div>
            </div>
            <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
              <Button size="icon" variant="ghost" onclick={() => startEdit(cmd)} class="h-8 w-8 text-slate-400 hover:text-white hover:bg-white/10">
                <Edit2 class="w-3.5 h-3.5" />
              </Button>
              <Button size="icon" variant="ghost" onclick={() => requestDeleteCommand(cmd.id)} class="h-8 w-8 text-slate-400 hover:text-red-400 hover:bg-white/10">
                <Trash2 class="w-3.5 h-3.5" />
              </Button>
            </div>
          </div>

          <div class="space-y-3 relative z-10">
            <div class="bg-black/40 rounded-xl p-3 border border-white/10">
              <code class="text-xs text-blue-200 font-bold break-all line-clamp-2">{getCommandDescription(cmd)}</code>
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

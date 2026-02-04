<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Terminal } from 'xterm';
  import { FitAddon } from 'xterm-addon-fit';
  import 'xterm/css/xterm.css';
  import { API_URL } from '$lib/api';
  import { X, Maximize2, Minimize2 } from '@lucide/svelte';
  import { Button } from '$lib/components/ui/button';

  let { nodeId, nodeName, onClose } = $props();

  let terminalElement: HTMLDivElement;
  let terminal: Terminal;
  let fitAddon: FitAddon;
  let ws: WebSocket;
  let isMaximized = $state(false);

  onMount(() => {
    terminal = new Terminal({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: 'Menlo, Monaco, "Courier New", monospace',
      theme: {
        background: '#1a1b26',
        foreground: '#a9b1d6',
      }
    });

    fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);
    terminal.open(terminalElement);
    fitAddon.fit();

    // Initialize WebSocket
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const host = API_URL.replace(/^https?:\/\//, '');
    const token = localStorage.getItem('token');
    const wsUrl = `${protocol}//${host}/api/nodes/terminal?token=${token}`;

    ws = new WebSocket(wsUrl);

    ws.onopen = () => {
      // WebSocket connection established
    };

    ws.onmessage = async (event) => {
      if (event.data instanceof Blob) {
        const text = await event.data.text();
        terminal.write(text);
      } else {
        terminal.write(event.data);
      }
    };

    terminal.onData((data) => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(data);
      }
    });

    window.addEventListener('resize', handleResize);
  });

  function handleResize() {
    if (fitAddon) fitAddon.fit();
  }

  onDestroy(() => {
    if (ws) ws.close();
    if (terminal) terminal.dispose();
    window.removeEventListener('resize', handleResize);
  });

  function toggleMaximize() {
    isMaximized = !isMaximized;
    setTimeout(handleResize, 100);
  }
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4 backdrop-blur-sm" class:p-0={isMaximized}>
  <div class="flex flex-col bg-[#1a1b26] rounded-xl border border-white/10 shadow-2xl overflow-hidden transition-all duration-300"
       class:w-full={isMaximized} class:h-full={isMaximized}
       class:w-[800px]={!isMaximized} class:h-[500px]={!isMaximized}>

    <!-- Titlebar -->
    <div class="flex items-center justify-between px-4 py-2 bg-black/20 border-b border-white/5">
      <div class="flex items-center gap-2">
        <div class="flex gap-1.5">
          <div class="w-3 h-3 rounded-full bg-red-500/80"></div>
          <div class="w-3 h-3 rounded-full bg-yellow-500/80"></div>
          <div class="w-3 h-3 rounded-full bg-green-500/80"></div>
        </div>
        <span class="text-xs font-medium text-white/70 ml-2">Terminal: {nodeName}</span>
      </div>

      <div class="flex items-center gap-1">
        <Button variant="ghost" size="icon" class="h-8 w-8 text-white/50 hover:text-white" onclick={toggleMaximize}>
          {#if isMaximized}
            <Minimize2 class="h-4 w-4" />
          {:else}
            <Maximize2 class="h-4 w-4" />
          {/if}
        </Button>
        <Button variant="ghost" size="icon" class="h-8 w-8 text-white/50 hover:text-red-400" onclick={onClose}>
          <X class="h-4 w-4" />
        </Button>
      </div>
    </div>

    <!-- Terminal Area -->
    <div bind:this={terminalElement} class="flex-1 p-2 overflow-hidden"></div>
  </div>
</div>

<style>
  :global(.xterm-viewport) {
    background-color: transparent !important;
  }
</style>

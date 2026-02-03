<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Terminal } from 'xterm';
  import { FitAddon } from 'xterm-addon-fit';
  import 'xterm/css/xterm.css';
  import { API_URL } from '$lib/api';
  import { Monitor, Maximize2, Minimize2, RefreshCw } from '@lucide/svelte';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';

  let terminalElement: HTMLDivElement;
  let terminal: Terminal;
  let fitAddon: FitAddon;
  let ws: WebSocket;
  let isConnected = $state(false);

  function initTerminal() {
    if (ws) ws.close();
    if (terminal) terminal.dispose();

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
      isConnected = true;
      terminal.write('\x1b[32mConnected to host terminal\x1b[0m\r\n');
    };

    ws.onmessage = async (event) => {
      if (event.data instanceof Blob) {
        const text = await event.data.text();
        terminal.write(text);
      } else {
        terminal.write(event.data);
      }
    };

    ws.onclose = () => {
      isConnected = false;
      terminal.write('\r\n\x1b[31mDisconnected from host terminal\x1b[0m\r\n');
    };

    terminal.onData((data) => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(data);
      }
    });
  }

  onMount(() => {
    initTerminal();
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
</script>

<div class="h-full flex flex-col space-y-6">
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-3xl font-bold tracking-tight">Host Terminal</h1>
      <p class="text-muted-foreground">
        Direct shell access to the host machine.
      </p>
    </div>
    <div class="flex items-center gap-2">
      <div class="flex items-center gap-2 px-3 py-1 bg-muted rounded-full text-xs font-medium">
        <div class="w-2 h-2 rounded-full {isConnected ? 'bg-green-500 animate-pulse' : 'bg-red-500'}"></div>
        {isConnected ? 'Connected' : 'Disconnected'}
      </div>
      <Button variant="outline" size="sm" onclick={initTerminal}>
        <RefreshCw class="h-4 w-4 mr-2" />
        Reconnect
      </Button>
    </div>
  </div>

  <Card.Root class="flex-1 min-h-0 bg-[#1a1b26] border-white/10 overflow-hidden shadow-2xl relative group">
    <div class="absolute top-2 right-4 z-10 opacity-0 group-hover:opacity-100 transition-opacity">
      <div class="flex gap-2">
         <div class="w-3 h-3 rounded-full bg-red-500/80"></div>
         <div class="w-3 h-3 rounded-full bg-yellow-500/80"></div>
         <div class="w-3 h-3 rounded-full bg-green-500/80"></div>
      </div>
    </div>
    <Card.Content class="p-4 h-full">
      <div bind:this={terminalElement} class="h-full w-full"></div>
    </Card.Content>
  </Card.Root>
</div>

<style>
  :global(.xterm-viewport) {
    background-color: transparent !important;
  }
</style>

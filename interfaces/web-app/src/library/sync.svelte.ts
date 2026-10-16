import { getSocketUrl } from "../api.ts";

class SyncEngine extends EventTarget {
  _ws: WebSocket | null = null;
  isOpen: boolean = $state(false);
  private _reconnectTimer: ReturnType<typeof setTimeout> | null = null;

  connect() {
    if (
      this._ws &&
      (this._ws.readyState === WebSocket.OPEN || this._ws.readyState === WebSocket.CONNECTING)
    ) {
      return;
    }

    if (this._reconnectTimer) {
      clearTimeout(this._reconnectTimer);
      this._reconnectTimer = null;
    }

    const socket = new WebSocket(getSocketUrl());

    socket.onopen = () => {
      this.isOpen = true;
      this.dispatchEvent(new Event("open"));
    };

    socket.onmessage = async (event: MessageEvent) => {
      try {
        const raw = event.data instanceof Blob ? await event.data.text() : event.data;
        const data = JSON.parse(raw);
        this.dispatchEvent(new CustomEvent("message", { detail: data }));
      } catch {}
    };

    socket.onclose = () => {
      this.isOpen = false;
      this._ws = null;
      this.dispatchEvent(new Event("close"));
      this.scheduleReconnect();
    };

    socket.onerror = () => {
      socket.close();
    };

    this._ws = socket;
  }

  private scheduleReconnect() {
    if (this._reconnectTimer) {
      return;
    }
    this._reconnectTimer = setTimeout(() => {
      this._reconnectTimer = null;
      this.connect();
    }, 2000);
  }

  send(payload: unknown) {
    if (this._ws && this._ws.readyState === WebSocket.OPEN) {
      this._ws.send(JSON.stringify(payload));
    }
  }
}

export const sync = new SyncEngine();

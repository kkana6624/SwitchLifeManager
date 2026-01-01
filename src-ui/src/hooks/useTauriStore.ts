import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { MonitorSharedState } from '../types';

export function useTauriStore() {
  const [state, setState] = useState<MonitorSharedState | null>(null);

  useEffect(() => {
    // Initial fetch
    invoke<MonitorSharedState>('get_snapshot')
      .then(setState)
      .catch(console.error);

    // Listen for updates
    const unlistenPromise = listen<MonitorSharedState>('state-update', (event) => {
      setState(event.payload);
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  return state;
}

import { AppShell, Burger, Group, NavLink, Text, Title, Badge } from '@mantine/core';
import { useDisclosure } from '@mantine/hooks';
import { useTauriStore } from './hooks/useTauriStore';
import { useState } from 'react';
import { Dashboard } from './features/dashboard/Dashboard';
import { Settings } from './features/settings/Settings';
import { InputTester } from './features/tester/InputTester';
import { LiveSession } from './features/live/LiveSession';
import { SessionHistory } from './features/sessions/SessionHistory';
import { History as MaintenanceHistory } from './features/history/History';

function App() {
  const [opened, { toggle }] = useDisclosure();
  const state = useTauriStore();
  const [activeTab, setActiveTab] = useState('dashboard');

  if (!state) {
    return <Text p="md">Connecting to backend...</Text>;
  }

  return (
    <AppShell
      header={{ height: 60 }}
      navbar={{ width: 250, breakpoint: 'sm', collapsed: { mobile: !opened } }}
      padding="md"
    >
      <AppShell.Header>
        <Group h="100%" px="md">
          <Burger opened={opened} onClick={toggle} hiddenFrom="sm" size="sm" />
          <Title order={3}>Switch Life Manager</Title>
          <Group ml="auto">
            <Badge color={state.is_connected ? "green" : "red"}>
              {state.is_connected ? "Connected" : "Disconnected"}
            </Badge>
            {state.is_game_running && <Badge color="blue">In Game</Badge>}
          </Group>
        </Group>
      </AppShell.Header>

      <AppShell.Navbar p="md">
        <NavLink
          label="Dashboard"
          active={activeTab === 'dashboard'}
          onClick={() => setActiveTab('dashboard')}
          variant="filled"
        />
        <NavLink
          label="Input Tester"
          active={activeTab === 'tester'}
          onClick={() => setActiveTab('tester')}
          variant="filled"
        />
        <NavLink
          label="Live Session"
          active={activeTab === 'live'}
          onClick={() => setActiveTab('live')}
          variant="filled"
        />
        <NavLink
          label="Past Sessions"
          active={activeTab === 'sessions'}
          onClick={() => setActiveTab('sessions')}
          variant="filled"
        />
        <NavLink
          label="Maintenance Log"
          active={activeTab === 'maintenance'}
          onClick={() => setActiveTab('maintenance')}
          variant="filled"
        />
        <NavLink
          label="Settings"
          active={activeTab === 'settings'}
          onClick={() => setActiveTab('settings')}
          variant="filled"
        />
      </AppShell.Navbar>

      <AppShell.Main>
        {activeTab === 'dashboard' && <Dashboard state={state} />}
        {activeTab === 'tester' && <InputTester state={state} />}
        {activeTab === 'live' && <LiveSession state={state} />}
        {activeTab === 'sessions' && <SessionHistory state={state} />}
        {/* Reusing History component for Maintenance Log */}
        {activeTab === 'maintenance' && <MaintenanceHistory state={state} />}
        {activeTab === 'settings' && <Settings state={state} />}
      </AppShell.Main>
    </AppShell>
  );
}

export default App;
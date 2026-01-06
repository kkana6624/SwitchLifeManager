import { Container, Grid, Card, Text, Select, Button, Stack, Title, Table, Group, NumberInput, Divider, Alert, Code, Switch } from '@mantine/core';
import { MonitorSharedState, AppConfig } from '../../types';
import { ORDERED_KEYS } from '../../constants';
import { invoke } from '@tauri-apps/api/core';
import { useState } from 'react';
import { KeyBindingModal } from './KeyBindingModal';

interface SettingsProps {
    state: MonitorSharedState;
}

export function Settings({ state }: SettingsProps) {
    const [learningKey, setLearningKey] = useState<string | null>(null);

    const handleConfigChange = (key: keyof AppConfig, value: any) => {
        const newConfig = { ...state.config, [key]: value };
        invoke('update_config', { config: newConfig });
    };

    const handleResetMapping = () => {
        if (confirm("Reset all key bindings to default (PhoenixWAN)?")) {
            invoke('reset_to_default_mapping');
        }
    };

    const handleBind = (key: string, button: number) => {
        invoke('set_binding', { key, button });
        setLearningKey(null);
    };

    return (
        <Container fluid>
            <Grid>
                {/* Controller Settings */}
                <Grid.Col span={{ base: 12, md: 6 }}>
                    <Card shadow="sm" padding="lg" radius="md" withBorder h="100%">
                        <Title order={4} mb="md">Controller Configuration</Title>
                        <Stack>
                            <Select
                                label="Input Method"
                                description="Restart required if changed (usually)"
                                data={[
                                    { value: 'DirectInput', label: 'DirectInput (HID) - Recommended' },
                                    { value: 'XInput', label: 'XInput (Xbox)' }
                                ]}
                                value={state.config.input_method}
                                onChange={(val) => handleConfigChange('input_method', val)}
                            />

                            <NumberInput
                                label="Controller Index"
                                description="Device ID (0 = First controller found)"
                                min={0}
                                max={16}
                                value={state.config.target_controller_index}
                                onChange={(val) => invoke('set_target_controller', { index: Number(val) })}
                            />

                            <NumberInput
                                label="Chatter Threshold (ms)"
                                description="Debounce time to ignore mechanical chatter"
                                min={1}
                                max={100}
                                value={state.config.chatter_threshold_ms}
                                onChange={(val) => handleConfigChange('chatter_threshold_ms', Number(val))}
                            />

                            <Title order={5} mt="sm">Polling Rates (ms)</Title>
                            <Group grow>
                                <NumberInput
                                    label="Connected"
                                    min={1}
                                    max={1000}
                                    value={state.config.polling_rate_ms_connected}
                                    onChange={(val) => handleConfigChange('polling_rate_ms_connected', Number(val))}
                                />
                                <NumberInput
                                    label="Disconnected"
                                    min={100}
                                    max={5000}
                                    value={state.config.polling_rate_ms_disconnected}
                                    onChange={(val) => handleConfigChange('polling_rate_ms_disconnected', Number(val))}
                                />
                            </Group>
                        </Stack>
                    </Card>
                </Grid.Col>

                {/* Key Bindings */}
                <Grid.Col span={{ base: 12, md: 6 }}>
                    <Card shadow="sm" padding="lg" radius="md" withBorder h="100%">
                        <Group justify="space-between" mb="md">
                            <Title order={4}>Key Bindings</Title>
                            <Button variant="light" color="red" size="xs" onClick={handleResetMapping}>
                                Reset Defaults
                            </Button>
                        </Group>

                        <Table striped highlightOnHover>
                            <Table.Thead>
                                <Table.Tr>
                                    <Table.Th>Key</Table.Th>
                                    <Table.Th>Assigned Button (Bitmask)</Table.Th>
                                    <Table.Th>Action</Table.Th>
                                </Table.Tr>
                            </Table.Thead>
                            <Table.Tbody>
                                {ORDERED_KEYS.map(key => {
                                    const binding = state.bindings[key];
                                    const isBound = binding !== undefined && binding !== 0;

                                    return (
                                        <Table.Tr key={key}>
                                            <Table.Td fw={500}>{key}</Table.Td>
                                            <Table.Td>
                                                {isBound ? (
                                                    <Code>{binding}</Code>
                                                ) : (
                                                    <Text c="dimmed" size="sm">Unbound</Text>
                                                )}
                                            </Table.Td>
                                            <Table.Td>
                                                <Button
                                                    size="xs"
                                                    variant="outline"
                                                    onClick={() => setLearningKey(key)}
                                                >
                                                    Set
                                                </Button>
                                            </Table.Td>
                                        </Table.Tr>
                                    );
                                })}
                            </Table.Tbody>
                        </Table>
                    </Card>
                </Grid.Col>
                {/* OBS Integration */}
                <Grid.Col span={{ base: 12 }}>
                    <Card shadow="sm" padding="lg" radius="md" withBorder>
                        <Group justify="space-between" mb="md">
                            <Title order={4}>OBS Integration</Title>
                            <Text size="sm" c={state.config.obs_enabled ? "green" : "dimmed"}>
                                {state.config.obs_enabled ? "Enabled" : "Disabled"}
                            </Text>
                        </Group>
                        <Grid>
                            <Grid.Col span={{ base: 12, md: 6 }}>
                                <Switch
                                    label="Enable OBS Server"
                                    description="Starts a local HTTP server for OBS Browser Source"
                                    checked={state.config.obs_enabled}
                                    onChange={(event: React.ChangeEvent<HTMLInputElement>) => invoke('set_obs_enabled', { enabled: event.currentTarget.checked })}
                                    mb="md"
                                />
                                <NumberInput
                                    label="Server Port"
                                    description="Port to listen on (Default: 36000)"
                                    value={state.config.obs_port}
                                    min={1024}
                                    max={65535}
                                    onChange={(val) => invoke('set_obs_port', { port: Number(val) })}
                                    disabled={!state.config.obs_enabled}
                                    mb="md"
                                />
                                <NumberInput
                                    label="Overlay Refresh Rate (ms)"
                                    description="How often OBS updates the stats (100ms - 5000ms)"
                                    value={state.config.obs_poll_interval_ms}
                                    min={100}
                                    max={5000}
                                    step={100}
                                    onChange={(val) => invoke('set_obs_poll_interval', { intervalMs: Number(val) })}
                                    mb="md"
                                />
                            </Grid.Col>
                            <Grid.Col span={{ base: 12, md: 6 }}>
                                <Text fw={500} mb={5}>Browser Source URL</Text>
                                <Alert color="blue" variant="light">
                                    <Code block>{`http://localhost:${state.config.obs_port}/`}</Code>
                                </Alert>
                                <Text size="xs" c="dimmed" mt="xs">
                                    Copy this URL into OBS Studio "Browser" source properties.
                                    Set width/height as needed (e.g. 400x600).
                                </Text>
                            </Grid.Col>
                        </Grid>
                    </Card>
                </Grid.Col>
            </Grid>

            {/* Config JSON Dump (Debug) */}
            <Divider my="xl" label="Advanced" labelPosition="center" />
            <Alert title="Current Config" color="gray" variant="light">
                <Code block>{JSON.stringify(state.config, null, 2)}</Code>
            </Alert>

            <KeyBindingModal
                opened={!!learningKey}
                onClose={() => setLearningKey(null)}
                targetKey={learningKey}
                currentRawState={state.raw_button_state}
                onBind={handleBind}
            />
        </Container>
    );
}

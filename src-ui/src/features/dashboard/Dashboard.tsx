import { Container, Grid, Card, Text, Progress, Group, Select, Button, Stack, Title, Badge, Checkbox, Paper, Modal, TextInput } from '@mantine/core';
import { MonitorSharedState, SwitchData } from '../../types';
import { SWITCH_MODELS, ORDERED_KEYS } from '../../constants';
import { invoke } from '@tauri-apps/api/core';
import { useState } from 'react';

interface DashboardProps {
    state: MonitorSharedState;
}

export function Dashboard({ state }: DashboardProps) {
    const [selectedKeys, setSelectedKeys] = useState<string[]>([]);
    const [bulkModelId, setBulkModelId] = useState<string | null>(null);

    // Date Edit State
    const [dateModalOpen, setDateModalOpen] = useState(false);
    const [editingKey, setEditingKey] = useState<string | null>(null);
    const [editingDate, setEditingDate] = useState<string>('');

    const getSwitchModel = (id: string) => {
        return SWITCH_MODELS.find(m => m.id === id) || SWITCH_MODELS.find(m => m.id === "generic_unknown")!;
    };

    const getLifeExpectancyPercentage = (presses: number, rated: number) => {
        const remaining = rated - presses;
        if (remaining <= 0) return 0;
        return (remaining / rated) * 100;
    };

    const getProgressColor = (percentage: number) => {
        if (percentage > 50) return 'green';
        if (percentage > 25) return 'yellow';
        return 'red';
    };

    const handleResetStats = (key: string) => {
        if (confirm(`Reset stats for ${key}?`)) {
            invoke('reset_stats', { key });
        }
    };

    const handleModelChange = (key: string, modelId: string) => {
        if (confirm(`Change model for ${key} to ${modelId}? Stats will be reset.`)) {
            invoke('replace_switch', { key, newModelId: modelId });
        }
    };

    const toggleSelection = (key: string) => {
        setSelectedKeys(prev =>
            prev.includes(key) ? prev.filter(k => k !== key) : [...prev, key]
        );
    };

    const toggleSelectAll = () => {
        if (selectedKeys.length === ORDERED_KEYS.length) {
            setSelectedKeys([]);
        } else {
            setSelectedKeys([...ORDERED_KEYS]);
        }
    };

    const handleBulkReset = () => {
        if (selectedKeys.length === 0) return;
        if (confirm(`Reset stats for ${selectedKeys.length} selected keys?`)) {
            selectedKeys.forEach(key => invoke('reset_stats', { key }));
            setSelectedKeys([]);
        }
    };

    const handleBulkApplyModel = () => {
        if (selectedKeys.length === 0 || !bulkModelId) return;
        if (confirm(`Change model to ${bulkModelId} for ${selectedKeys.length} keys? Stats will be reset.`)) {
            selectedKeys.forEach(key => invoke('replace_switch', { key, newModelId: bulkModelId }));
            setSelectedKeys([]);
        }
    };

    const handleOpenDateEdit = (key: string, currentDate: string | null) => {
        setEditingKey(key);
        if (currentDate) {
            // Convert to YYYY-MM-DDTHH:mm local time approximation or use UTC?
            // Input type datetime-local expects local time.
            // new Date(isoString) converts to local time object.
            // We need to format it.
            const date = new Date(currentDate);
            // Simple hack for YYYY-MM-DDTHH:mm
            // (Note: manual offset adjustment might be needed if exact precision matters, 
            // but for simple UI this often suffices or use a library)
            // Let's use simplified string manipulation for ISO-like local format
            const offsetMs = date.getTimezoneOffset() * 60 * 1000;
            const localISOTime = (new Date(date.getTime() - offsetMs)).toISOString().slice(0, 16);
            setEditingDate(localISOTime);
        } else {
            const now = new Date();
            const offsetMs = now.getTimezoneOffset() * 60 * 1000;
            const localISOTime = (new Date(now.getTime() - offsetMs)).toISOString().slice(0, 16);
            setEditingDate(localISOTime);
        }
        setDateModalOpen(true);
    };

    const handleSaveDate = () => {
        if (editingKey && editingDate) {
            const dateObj = new Date(editingDate);
            invoke('set_last_replaced_date', { key: editingKey, date: dateObj.toISOString() });
            setDateModalOpen(false);
        }
    };

    return (
        <Container fluid>
            <Group justify="space-between" mb="md">
                <Title order={4}>Switch Life Expectancy</Title>
                <Button variant="default" size="xs" onClick={toggleSelectAll}>
                    {selectedKeys.length === ORDERED_KEYS.length ? "Deselect All" : "Select All"}
                </Button>
            </Group>

            {selectedKeys.length > 0 && (
                <Paper p="md" mb="lg" bg="blue.0" withBorder>
                    <Title order={5} mb="xs">Bulk Actions ({selectedKeys.length} selected)</Title>
                    <Group align="end">
                        <Select
                            label="Change Model"
                            placeholder="Select Switch Model"
                            data={SWITCH_MODELS.map(m => ({ value: m.id, label: m.name }))}
                            value={bulkModelId}
                            onChange={setBulkModelId}
                            style={{ flexGrow: 1, maxWidth: 300 }}
                        />
                        <Button onClick={handleBulkApplyModel} disabled={!bulkModelId}>
                            Apply Model
                        </Button>
                        <Button color="red" variant="light" onClick={handleBulkReset}>
                            Reset Stats
                        </Button>
                    </Group>
                </Paper>
            )}

            {/* Session Info Section */}
            <Grid mb="lg">
                <Grid.Col span={{ base: 12, md: 6 }}>
                    <Paper shadow="xs" p="md" withBorder>
                        <Group justify="space-between" mb="xs">
                            <Title order={5}>Session Stats ({state.is_game_running ? "Live" : "Previous"})</Title>
                            {state.is_game_running && <Badge color="green" variant="dot">Running</Badge>}
                        </Group>
                        <Group grow>
                            <Stack gap={0}>
                                <Text size="xs" c="dimmed">Session Presses</Text>
                                <Text fw={700} size="lg">
                                    {ORDERED_KEYS.reduce((acc, key) => acc + (state.switches[key]?.stats.last_session_presses || 0), 0).toLocaleString()}
                                </Text>
                            </Stack>
                            <Stack gap={0}>
                                <Text size="xs" c="dimmed">Session Chatters</Text>
                                <Text fw={700} size="lg" c="red">
                                    {ORDERED_KEYS.reduce((acc, key) => acc + (state.switches[key]?.stats.last_session_chatters || 0), 0).toLocaleString()}
                                </Text>
                            </Stack>
                        </Group>
                    </Paper>
                </Grid.Col>

                <Grid.Col span={{ base: 12, md: 6 }}>
                    <Paper shadow="xs" p="md" withBorder>
                        <Title order={5} mb="xs">Recent Sessions (Last 3)</Title>
                        {state.recent_sessions && state.recent_sessions.length > 0 ? (
                            <Stack gap="xs">
                                {state.recent_sessions.slice().reverse().map((session, idx) => (
                                    <Group key={idx} justify="space-between">
                                        <Text size="sm">{new Date(session.start_time).toLocaleString()}</Text>
                                        <Badge variant="outline">{session.duration_secs}s</Badge>
                                    </Group>
                                ))}
                            </Stack>
                        ) : (
                            <Text size="sm" c="dimmed">No recent sessions recorded.</Text>
                        )}
                    </Paper>
                </Grid.Col>
            </Grid>

            {/* Switch Grid */}
            <Grid>
                {ORDERED_KEYS.map(key => {
                    const switchData = state.switches[key] || {
                        switch_model_id: "generic_unknown",
                        stats: { total_presses: 0, total_chatters: 0 }
                    } as SwitchData;

                    const model = getSwitchModel(switchData.switch_model_id);
                    const percentage = getLifeExpectancyPercentage(switchData.stats.total_presses, model.rated_lifespan_presses);
                    const color = getProgressColor(percentage);
                    const isSelected = selectedKeys.includes(key);

                    // Calculations
                    const totalEvents = switchData.stats.total_presses + switchData.stats.total_chatters;
                    const chatterRate = totalEvents > 0
                        ? (switchData.stats.total_chatters / totalEvents) * 100
                        : 0;
                    const isHighChatter = chatterRate > 0.5; // Warning threshold > 0.5%

                    return (
                        <Grid.Col key={key} span={{ base: 12, md: 6, lg: 4 }}>
                            <Card
                                shadow="sm"
                                padding="lg"
                                radius="md"
                                withBorder
                                style={{ borderColor: isSelected ? 'var(--mantine-color-blue-5)' : undefined, borderWidth: isSelected ? 2 : 1 }}
                            >
                                <Group justify="space-between" mb="xs">
                                    <Group>
                                        <Checkbox
                                            checked={isSelected}
                                            onChange={() => toggleSelection(key)}
                                        />
                                        <Text fw={700}>{key}</Text>
                                    </Group>
                                    <Group gap="xs">
                                        {isHighChatter && (
                                            <Badge color="red" variant="filled">Warning: Chatter</Badge>
                                        )}
                                        <Badge color={color} variant="light">
                                            {percentage.toFixed(1)}% Life
                                        </Badge>
                                    </Group>
                                </Group>

                                <Text size="sm" c="dimmed" mb="xs">
                                    Model: {model.name}
                                </Text>

                                <Progress
                                    value={percentage}
                                    color={color}
                                    size="xl"
                                    radius="xl"
                                    mb="md"
                                />

                                <Group grow mb="md">
                                    <Stack gap={0}>
                                        <Text size="xs" c="dimmed">Presses</Text>
                                        <Text fw={500}>{switchData.stats.total_presses.toLocaleString()}</Text>
                                    </Stack>
                                    <Stack gap={0}>
                                        <Text size="xs" c="dimmed">Chatters</Text>
                                        <Text fw={500} c={isHighChatter ? 'red' : undefined}>
                                            {switchData.stats.total_chatters.toLocaleString()} ({chatterRate.toFixed(2)}%)
                                        </Text>
                                    </Stack>
                                    <Stack gap={0}>
                                        <Text size="xs" c="dimmed">Session</Text>
                                        <Text fw={500}>
                                            {switchData.stats.last_session_presses.toLocaleString()}
                                        </Text>
                                    </Stack>
                                </Group>

                                <Stack gap={0} mb="md">
                                    <Text size="xs" c="dimmed">Last Replaced</Text>
                                    <Group justify="space-between">
                                        <Text size="sm">
                                            {switchData.last_replaced_at ? new Date(switchData.last_replaced_at).toLocaleDateString() : 'Never'}
                                        </Text>
                                        <Button variant="subtle" size="compact-xs" onClick={() => handleOpenDateEdit(key, switchData.last_replaced_at)}>
                                            Edit
                                        </Button>
                                    </Group>
                                </Stack>

                                <Group>
                                    <Select
                                        size="xs"
                                        data={SWITCH_MODELS.map(m => ({ value: m.id, label: m.name }))}
                                        value={switchData.switch_model_id}
                                        onChange={(val: string | null) => val && handleModelChange(key, val)}
                                        style={{ flexGrow: 1 }}
                                    />
                                    <Button size="xs" color="red" variant="light" onClick={() => handleResetStats(key)}>
                                        Reset
                                    </Button>
                                </Group>
                            </Card>
                        </Grid.Col>
                    );
                })}
            </Grid >

            <Modal opened={dateModalOpen} onClose={() => setDateModalOpen(false)} title={`Set Last Replaced Date: ${editingKey}`}>
                <Stack>
                    <TextInput
                        type="datetime-local"
                        label="Replacement Date"
                        value={editingDate}
                        onChange={(e: React.ChangeEvent<HTMLInputElement>) => setEditingDate(e.currentTarget.value)}
                    />
                    <Group justify="flex-end">
                        <Button variant="default" onClick={() => setDateModalOpen(false)}>Cancel</Button>
                        <Button onClick={handleSaveDate}>Save</Button>
                    </Group>
                </Stack>
            </Modal>
        </Container >
    );
}
import { Container, Grid, Card, Text, Progress, Group, Select, Button, Stack, Title, Badge, Checkbox, Paper } from '@mantine/core';
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
                                    <Badge color={color} variant="light">
                                        {percentage.toFixed(1)}% Life
                                    </Badge>
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
                                        <Text fw={500}>{switchData.stats.total_presses.toLocaleString()} / {model.rated_lifespan_presses.toLocaleString()}</Text>
                                    </Stack>
                                    <Stack gap={0}>
                                        <Text size="xs" c="dimmed">Chatters</Text>
                                        <Text fw={500}>{switchData.stats.total_chatters.toLocaleString()}</Text>
                                    </Stack>
                                </Group>

                                <Group>
                                    <Select 
                                        size="xs"
                                        data={SWITCH_MODELS.map(m => ({ value: m.id, label: m.name }))}
                                        value={switchData.switch_model_id}
                                        onChange={(val) => val && handleModelChange(key, val)}
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
            </Grid>
        </Container>
    );
}

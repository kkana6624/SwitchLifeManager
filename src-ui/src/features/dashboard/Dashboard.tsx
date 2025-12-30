import { Container, Grid, Card, Text, Progress, Group, Select, Button, Stack, Title, Badge } from '@mantine/core';
import { MonitorSharedState, SwitchData } from '../../types';
import { SWITCH_MODELS, ORDERED_KEYS } from '../../constants';
import { invoke } from '@tauri-apps/api/core';

interface DashboardProps {
    state: MonitorSharedState;
}

export function Dashboard({ state }: DashboardProps) {
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

    return (
        <Container fluid>
            <Title order={4} mb="md">Switch Life Expectancy</Title>
            <Grid>
                {ORDERED_KEYS.map(key => {
                    const switchData = state.switches[key] || {
                        switch_model_id: "generic_unknown",
                        stats: { total_presses: 0, total_chatters: 0 }
                    } as SwitchData;
                    
                    const model = getSwitchModel(switchData.switch_model_id);
                    const percentage = getLifeExpectancyPercentage(switchData.stats.total_presses, model.rated_lifespan_presses);
                    const color = getProgressColor(percentage);

                    return (
                        <Grid.Col key={key} span={{ base: 12, md: 6, lg: 4 }}>
                            <Card shadow="sm" padding="lg" radius="md" withBorder>
                                <Group justify="space-between" mb="xs">
                                    <Text fw={700}>{key}</Text>
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
                                    <Button size="xs" color="red" variant="subtle" onClick={() => handleResetStats(key)}>
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
